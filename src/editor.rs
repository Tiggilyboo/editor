use std::collections::{
    BTreeMap,
    HashMap,
};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::cell::RefCell;

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
    },
    width_cache::WidthCache,
    styles::{ 
        ThemeStyleMap,
        Style,
    },
    line_cache::LineCache,
    Rope,
};
use ui::view::ViewWidget;

pub struct EditorState {
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
    client: Arc<Client>,
    style_map: RefCell<ThemeStyleMap>, 
    styles: Arc<Mutex<HashMap<isize, Style>>>,
    width_cache: RefCell<WidthCache>,
    kill_ring: RefCell<Rope>,
    file_manager: FileManager,
    editors: BTreeMap<BufferId, Arc<Mutex<RefCell<Editor>>>>,
    views: BTreeMap<ViewId, Arc<Mutex<RefCell<View>>>>,
    view_widgets: BTreeMap<ViewId, Arc<Mutex<RefCell<ViewWidget>>>>,
    line_caches: BTreeMap<ViewId, Arc<Mutex<RefCell<LineCache>>>>,
    focused_view_id: Option<ViewId>,
    id_counter: usize,
}

fn create_frontend_thread(
    client: Arc<Client>,
    views: Arc<BTreeMap<ViewId, Arc<Mutex<RefCell<View>>>>>,
    view_widgets: Arc<BTreeMap<ViewId, Arc<Mutex<RefCell<ViewWidget>>>>>, 
    cache: Arc<Mutex<LineCache>>, 
    styles: Arc<Mutex<HashMap<isize, Style>>>,
) {
    let client = client.clone();
    let styles = styles.clone();
    let line_cache = cache.clone();
    let view_widgets = view_widgets.clone();

    std::thread::spawn(move || {
        println!("frontend_thread started...");

        while let Ok(msg) = client.get_message_stream().lock().unwrap().recv() {
            println!("Got message: {:?}", msg);
            match msg.payload {
                Payload::BufferUpdate(update) => {
                    if let Some(view_id) = msg.view_id { 
                        if let Ok(mut line_cache) = line_cache.try_lock() {
                            line_cache.apply_update(update);
                            if let Some(view_widget) = view_widgets.get(&view_id) {
                                view_widget.lock().unwrap().borrow_mut().populate(&line_cache, styles.clone());
                            }
                        }
                    }
                },
                Payload::Command(Command::Scroll { line, col }) => {
                },
                Payload::Command(Command::Idle { token }) => {

                },
                Payload::Command(Command::ShowHover { req_id, content }) => {

                },
                Payload::Command(Command::DefineStyle { style_id, style }) => {

                },
            }
        }
        println!("frontend_thread finished.");
    });
}

impl EditorState {
    pub fn new() -> Self {
        let client = Arc::new(Client::new());
        let styles = Arc::new(Mutex::new(HashMap::new()));
        let editors = BTreeMap::<BufferId, Arc<Mutex<RefCell<Editor>>>>::new();
        let views = BTreeMap::<ViewId, Arc<Mutex<RefCell<View>>>>::new();
        let view_widgets = BTreeMap::<ViewId, Arc<Mutex<RefCell<ViewWidget>>>>::new();
        let line_caches = BTreeMap::<ViewId, Arc<Mutex<RefCell<LineCache>>>>::new();

        Self {
            client,
            styles,
            editors,
            line_caches,
            views,
            view_widgets,
            key_bindings: default_key_bindings(),
            mouse_bindings: default_mouse_bindings(),
            file_manager: FileManager::new(),
            width_cache: RefCell::new(WidthCache::new()),
            style_map: RefCell::new(ThemeStyleMap::new(None)),
            kill_ring: RefCell::new(Rope::from("")),
            focused_view_id: None,
            id_counter: 0,
        }
    }

    /// Creates an `EventContext` for the provided `ViewId`. This context
    /// holds references to the `Editor` and `View` backing this `ViewId`,
    /// as well as to sibling views, plugins, and other state necessary
    /// for handling most events.
    pub(crate) fn make_context(&self, view_id: ViewId) -> Option<EventContext> {
        if let Some(view) = self.views.get(&view_id) {
            let buffer_id = view.lock().unwrap().borrow().get_buffer_id();
            let info = self.file_manager.get_info(buffer_id);
            let editor = self.editors.get(&buffer_id).unwrap();

            Some(EventContext {
                view_id,
                view: &view,
                buffer_id,
                editor: &editor,
                info,
                siblings: Vec::new(),
                client: &self.client,
                style_map: &self.style_map,
                width_cache: &self.width_cache,
                kill_ring: &self.kill_ring,
            })
        } else {
            None
        }
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

    #[inline]
    fn process_action(&mut self, action: Action) {
        println!("Action: {:?}", action);

        if let Some(view_id) = self.focused_view_id {
            if let Some(mut ctx) = self.make_context(view_id) {
                ctx.do_edit(action);
            }
        } else {
            println!("No focused view set to process action: {:?}", action);
        }
    }

    pub fn process_input_actions(&mut self, state: &InputState) {
        if let Some(focused_view_id) = self.focused_view_id {
            if let Some(focused_view) = self.views.get(&focused_view_id) {
                let mode = focused_view.lock().unwrap().borrow().get_mode();
                let input_actions = self.acquire_input_actions(mode, state);

                for action in input_actions {
                    self.process_action(action);
                }
            } else {
                println!("No focused view set to process input state!");
            }
        }
    }

    pub fn do_new_view(&mut self, path: Option<PathBuf>) {
        let view_id = self.next_view_id();
        let buffer_id = self.next_buffer_id();
        let path_str: Option<String> = if let Some(path) = path {
            if let Ok(path) = path.into_os_string().into_string() {
                Some(path)
            } else {
                None
            }
        } else {
            None
        };

        self.editors.insert(buffer_id, Arc::new(Mutex::new(RefCell::new(Editor::new()))));
        self.views.insert(view_id, Arc::new(Mutex::new(RefCell::new(View::new(view_id, buffer_id)))));
        self.line_caches.insert(view_id, Arc::new(Mutex::new(RefCell::new(LineCache::new()))));
        self.view_widgets.insert(view_id, Arc::new(Mutex::new(RefCell::new(ViewWidget::new(view_id, path_str)))));

        self.focused_view_id = Some(view_id);
    }
}

