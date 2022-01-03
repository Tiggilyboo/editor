use std::iter::Iterator;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::cell::RefCell;
use std::thread::JoinHandle;

use winit::event_loop::EventLoopProxy;

use crate::events::{
    binding::{
        KeyBinding,
        MouseBinding,
        default_key_bindings,
        default_mouse_bindings,
    },
    state::InputState,
};
use eddy::{
    Action,
    Editor,
    EventContext,
    BufferId,
    Mode,
    FileManager,
    View,
    ViewId,
    Client,
    client::{
        Command, Payload,
        Request, Response,
    },
    width_cache::{
        WidthCache,
        Width,
    },
    styles::ThemeStyleMap,
    line_cache::LineCache,
    Size,
    Rope,
};
use ui::{
    widget::Widget,
    view::{
        ViewWidget,
        ViewResources,
    },
};
use render::text::FontBounds;

pub enum EditorEvent {}

pub type Threaded<T> = Arc<Mutex<T>>;

pub struct EditorState {
    proxy: EventLoopProxy<EditorEvent>, 
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
    client: Arc<Client>,
    style_map: Arc<Mutex<ThemeStyleMap>>, 
    width_cache: RefCell<WidthCache>,
    kill_ring: RefCell<Rope>,
    file_manager: FileManager,
    editors: BTreeMap<BufferId, RefCell<Editor>>,
    views: BTreeMap<ViewId, RefCell<View>>,
    view_widgets: BTreeMap<ViewId, Threaded<ViewWidget>>,
    view_resources: Threaded<ViewResources>,
    line_cache: BTreeMap<ViewId, Threaded<LineCache>>,
    font_bounds: Arc<Mutex<FontBounds>>,
    focused_view_id: Option<ViewId>,
    frontend_thread: Option<JoinHandle<()>>,
    id_counter: usize,
}

fn create_frontend_thread(
    view_id: ViewId,
    client: Arc<Client>,
    view_widget: Threaded<ViewWidget>,
    cache: Threaded<LineCache>,
    style_map: Threaded<ThemeStyleMap>,
    view_resources: Threaded<ViewResources>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        println!("frontend_thread started...");

        while let Ok(msg) = client.get_message_stream().lock().unwrap().recv() {
            match msg.payload {
                Payload::BufferUpdate(update) => {
                    if let Some(msg_view_id) = msg.view_id { 
                        if msg_view_id != view_id {
                            panic!("Message buffer payload tried to update a view with different view_id: {:?}", msg_view_id);
                        }
                        if let Ok(mut line_cache) = cache.lock() {
                            line_cache.apply_update(update);

                            if let Ok(mut view_widget) = view_widget.lock() {
                                view_widget.populate(&line_cache, style_map.clone());
                            }
                        }
                    }
                },
                Payload::Command(cmd) => match cmd {
                    Command::Scroll { line, col: _ } => {
                        if let Ok(mut view_widget) = view_widget.lock() {
                            view_widget.scroll_to(line);
                            view_widget.set_dirty(true);
                        }
                    },
                    Command::ShowHover { req_id, content } => {
                        println!("Command::ShowHover triggered: {} = {}", req_id, content);
                        unimplemented!()
                    },
                    Command::DefineStyle { style_id, style } => {
                        println!("DefineStyle: {}, {:?}", style_id, style);
                        if let Ok(mut style_map) = style_map.lock() {
                            assert!(style_id == style_map.add(&style))
                        }
                    },
                    Command::ThemeChanged { theme_name, theme_settings } => {
                        println!("ThemeChanged: {}", theme_name);
                        if let Ok(mut view_resources) = view_resources.lock() {
                            if let Ok(style_map) = style_map.lock() {
                                view_resources.update_theme(&style_map, &theme_settings);
                            }
                        }
                        if let Ok(mut view_widget) = view_widget.lock() {
                            view_widget.update_from_resources();
                            view_widget.set_dirty(true);
                        }
                    },
                    Command::StatusUpdate { mode } => {
                        if let Ok(mut view_widget) = view_widget.lock() {
                            view_widget.status().set_mode(mode);
                        }
                    },
                },
                Payload::Request(Request::MeasureText { items }) => {
                    let mut resp = Vec::new();

                    if let Ok(view_widget) = view_widget.lock() {
                        for req in items.iter() {
                            let mut measurement = Vec::new();
                            for s in req.strings.iter() {
                                let width = view_widget.measure_text(s.into());
                                measurement.push(width as Width);
                            }
                            resp.push(measurement);
                        }
                    }

                    // Push results directly, will be picked up by accompanying request sender
                    // TODO: Probably a good idea to timeout if no sender picks this up?
                    client.push_results(Response::MeasureText {
                        response: resp,
                    });
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    })
}

impl EditorState {
    pub fn new(proxy: EventLoopProxy<EditorEvent>, font_bounds: Arc<Mutex<FontBounds>>) -> Self {
        let client = Arc::new(Client::new());
        let editors = BTreeMap::<BufferId, RefCell<Editor>>::new();
        let views = BTreeMap::<ViewId, RefCell<View>>::new();
        let view_widgets = BTreeMap::<ViewId, Threaded<ViewWidget>>::new();
        let line_cache = BTreeMap::<ViewId, Threaded<LineCache>>::new();
        let style_map = ThemeStyleMap::new(None);
        let view_resources = Arc::new(Mutex::new(ViewResources::from(&style_map)));
        
        Self {
            proxy,
            client,
            editors,
            line_cache,
            font_bounds,
            views,
            view_widgets,
            view_resources,
            key_bindings: default_key_bindings(),
            mouse_bindings: default_mouse_bindings(),
            file_manager: FileManager::new(),
            width_cache: RefCell::new(WidthCache::new()),
            style_map: Arc::new(Mutex::new(style_map)),
            kill_ring: RefCell::new(Rope::from("")),
            frontend_thread: None,
            focused_view_id: None,
            id_counter: 0,
        }
    }

    /// Creates an `EventContext` for the provided `ViewId`. This context
    /// holds references to the `Editor` and `View` backing this `ViewId`,
    /// as well as to sibling views, plugins, and other state necessary
    /// for handling most events.
    pub(crate) fn make_context(&self, view_id: ViewId) -> Option<EventContext> {
        self.views.get(&view_id).map(|view| {
            let buffer_id = view.borrow().get_buffer_id();
            let info = self.file_manager.get_info(buffer_id);
            let editor = &self.editors[&buffer_id];

            EventContext {
                view_id,
                buffer_id,
                info,
                view,
                editor,
                siblings: Vec::new(),
                client: &self.client,
                style_map: &self.style_map,
                width_cache: &self.width_cache,
                kill_ring: &self.kill_ring,
            }
        })
    }

    fn next_view_id(&self) -> ViewId {
        ViewId(self.id_counter + 1)
    }

    fn next_buffer_id(&self) -> BufferId {
        BufferId(self.id_counter + 1)
    }

    fn acquire_input_actions(&self, mode: Mode, state: &InputState) -> Vec<Action> {
        let mut triggered_actions: Vec<Action> = Vec::new();

        if let Some(pressed_key) = state.key {
            let mut key_triggers: Vec<Action> = self.key_bindings
                .iter()
                .filter(|b| b.is_triggered_by(mode, state.modifiers, &pressed_key))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut key_triggers);
        }
        if let Some(mouse_button) = state.mouse.button {
            let mut mouse_triggers: Vec<Action> = self.mouse_bindings
                .iter()
                .filter(|b| b.is_triggered_by(mode, state.modifiers, &mouse_button))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut mouse_triggers);
        }

        triggered_actions
    }

    fn process_action(&self, action: Action) {
        println!("Action: {:?}", action);

        if let Some(view_id) = self.focused_view_id {
            if let Some(mut ctx) = self.make_context(view_id) {
                ctx.do_edit(action);
            }
        } else {
            panic!("No focused view set to process action: {:?}", action);
        }
    }

    pub fn process_input_actions(&mut self, state: &InputState) {
        if let Some(focused_view_id) = self.focused_view_id {
            let mode = self.views
                .get(&focused_view_id)
                .unwrap().borrow()
                .get_mode();
            let input_actions = self.acquire_input_actions(mode, state);

            for action in input_actions {
                self.process_action(action);
            }
        } else {
            println!("No focused view set to process input state!");
        }
    }

    pub fn get_dirty_views(&self) -> Vec<&Arc<Mutex<ViewWidget>>> {
        self.view_widgets
            .iter()
            .filter_map(|(_,vw)| {
                if vw.lock().unwrap().dirty() {
                    Some(vw)
                } else {
                    None
                }
            }).collect()
    }

    pub fn get_views(&self) -> Vec<&Arc<Mutex<ViewWidget>>> {
        self.view_widgets.iter().map(|(_,vw)| vw).collect()
    }

    pub fn requires_redraw(&self) -> bool {
        self.view_widgets.iter()
            .any(|(_, vw)| vw.lock().unwrap().dirty())
    }

    pub fn do_new_view(&mut self, path: Option<String>) {
        let view_id = self.next_view_id();
        let buffer_id = self.next_buffer_id();

        let editor = RefCell::new(Editor::new());
        let view = RefCell::new(View::new(view_id, buffer_id));
        let line_cache = Arc::new(Mutex::new(LineCache::new()));
        let view_resources = self.view_resources.clone();
        let font_bounds = self.font_bounds.clone();
        let view_widget = Arc::new(Mutex::new(ViewWidget::new(view_id, path.clone(), view_resources, font_bounds)));

        self.editors.insert(buffer_id, editor);
        self.views.insert(view_id, view);
        self.line_cache.insert(view_id, line_cache);
        self.view_widgets.insert(view_id, view_widget.clone());
        self.focused_view_id = Some(view_id);
        
        if let Some(path) = path {
            let path = PathBuf::from(path);
            if let Ok(text) = self.file_manager.open(&path, buffer_id) {
                if let Some(mut context) = self.make_context(view_id) {
                    context.view_init();
                    context.reload(text);
                    context.finish_init();
                }
            }
        }

        self.frontend_thread = Some(create_frontend_thread(
            view_id,
            self.client.clone(),
            self.view_widgets.get(&view_id).unwrap().clone(),
            self.line_cache.get(&view_id).unwrap().clone(),
            self.style_map.clone(),
            self.view_resources.clone(),
        ));
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        for (view_id, _) in self.views.iter() {
            if let Some(mut ctx) = self.make_context(*view_id) {
                ctx.do_edit(Action::Resize(Size { width, height }));
            }
            if let Some(view_widget) = self.view_widgets.get(&view_id) {
                if let Ok(mut view_widget) = view_widget.lock() {
                    view_widget.resize(width as f32, height as f32);
                } else {
                    panic!("unable to lock view for resize");
                }
            }
        }
    }
}

