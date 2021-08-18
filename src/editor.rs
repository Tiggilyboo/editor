use std::collections::BTreeMap;
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
    width_cache::WidthCache,
    styles::ThemeStyleMap,
    Rope,
};


pub struct EditorState {
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
    mode: Mode,
    editors: BTreeMap<BufferId, RefCell<Editor>>,
    views: BTreeMap<ViewId, RefCell<View>>,
    file_manager: FileManager,
    style_map: RefCell<ThemeStyleMap>,
    width_cache: RefCell<WidthCache>,
    kill_ring: RefCell<Rope>,
    id_counter: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            key_bindings: default_key_bindings(),
            mouse_bindings: default_mouse_bindings(),
            mode: Mode::Normal,
            editors: BTreeMap::new(),
            views: BTreeMap::new(),
            file_manager: FileManager::new(),
            style_map: RefCell::new(ThemeStyleMap::new(None)),
            width_cache: RefCell::new(WidthCache::new()),
            kill_ring: RefCell::new(Rope::from("")),
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
            let editor = &self.editors[&buffer_id];
            let info = self.file_manager.get_info(buffer_id);

            EventContext {
                view_id,
                buffer_id,
                view,
                editor,
                info,
                siblings: Vec::new(),
                client: &Client{},
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

    #[inline]
    fn acquire_input_actions(&self, state: &InputState) -> Vec<Action> {
        let mut triggered_actions: Vec<Action> = Vec::new();

        if let Some(pressed_key) = state.key {
            let mut key_triggers: Vec<Action> = self.key_bindings
                .iter()
                .filter(|b| b.is_triggered_by(self.mode, state.modifiers, &pressed_key))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut key_triggers);
        }
        if let Some(mouse_button) = state.mouse.button {
            let mut mouse_triggers: Vec<Action> = self.mouse_bindings
                .iter()
                .filter(|b| b.is_triggered_by(self.mode, state.modifiers, &mouse_button))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut mouse_triggers);
        }

        triggered_actions
    }

    #[inline]
    fn process_action(&mut self, action: Action) {
        println!("Action: {:?}", action);

        let current_view_id = self.views.iter().next().unwrap().0;

        if let Some(mut ctx) = self.make_context(*current_view_id) {
            ctx.do_edit(action);
        }
    }

    pub fn process_input_actions(&mut self, state: &InputState) {
        let input_actions = self.acquire_input_actions(state);

        for action in input_actions {
            self.process_action(action);
        }
    }

    pub fn do_new_view(&mut self, path: Option<PathBuf>) {
        let view_id = self.next_view_id();
        let buffer_id = self.next_buffer_id();

        self.views.insert(view_id, RefCell::new(View::new(view_id, buffer_id)));
        self.editors.insert(buffer_id, RefCell::new(Editor::new()));
    }
}

