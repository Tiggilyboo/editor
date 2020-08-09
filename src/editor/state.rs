use std::sync::{
    Arc,
    Mutex,
};
use std::collections::HashMap;

use winit::event::ModifiersState;
use winit::event_loop::EventLoopProxy;

use super::ui::{
    view::EditView,
};
use rpc::{ 
    Mode,
    Action,
    ActionTarget,
};
use super::commands::EditViewCommands;
use crate::events::{
    EditorEvent,
    state::InputState,
    mapper_winit::map_scancode,
    binding::{
        Key,
        KeyBinding,
        MouseBinding,
        default_mouse_bindings,
        default_key_bindings,
    },
};
use super::plugins::{
    PluginId,
    PluginState,
};

pub type ViewId = String;

pub struct EditorState {
    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>, 
    themes: Vec<String>,
    languages: Vec<String>, 
    plugins: HashMap<PluginId, PluginState>, 
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
    event_proxy: Arc<EventLoopProxy<EditorEvent>>,
}

impl EditorState {
    pub fn new(event_proxy: Arc<EventLoopProxy<EditorEvent>>) -> Self {
        Self {
            focused: Default::default(),
            views: HashMap::new(),
            plugins: HashMap::new(),
            themes: vec![],
            languages: vec![],
            mouse_bindings: default_mouse_bindings(),
            key_bindings: default_key_bindings(),
            event_proxy,
        }
    }
    
    pub fn get_focused_view(&mut self) -> &mut EditView {
        let view_id = self.focused.clone()
            .expect("no focused EditView");

        self.views.get_mut(&view_id)
            .expect("Focused EditView not found in views")
    }

    pub fn set_available_themes(&mut self, themes: Vec<String>) {
        self.themes = themes;
    }
    pub fn set_available_languages(&mut self, languages: Vec<String>) {
        self.languages = languages;
    }
    pub fn set_available_plugins(&mut self, plugins: Vec<PluginState>) {
        for plugin in plugins.iter() {
            let name = plugin.name.clone();
            self.plugins.insert(name, plugin.clone());
        }
    }
    pub fn get_plugin(&self, plugin_id: PluginId) -> Option<PluginState> {
        if let Some(plugin) = self.plugins.get(&plugin_id) {
            Some(plugin.clone())
        } else {
            None
        }
    }

    pub fn process_keyboard_input(&self, mode: Mode, modifiers: ModifiersState, key: Key) -> Option<(Action, ActionTarget)> {
        let kc = match key {
            Key::KeyCode(virtual_keycode) => Some(virtual_keycode),
            Key::ScanCode(scancode) => map_scancode(scancode),
        };
        if kc.is_none() {
            return None;
        }
        if self.focused.is_none() {
            return None;
        }

        for binding in self.key_bindings.iter() {
            if binding.is_triggered_by(mode, modifiers, &Key::KeyCode(kc.unwrap())) {
                return Some((
                    binding.get_action(),
                    binding.get_target(),
                ));
            }
        }

        None
    }

    pub fn update_from_input(&mut self, input: Arc<Mutex<InputState>>) -> bool {
        if let Ok(ref input) = input.clone().try_lock() {
            let should_keydown = input.key.is_some() 
                || input.modifiers.ctrl() || input.modifiers.shift() || input.modifiers.alt();
            let should_mouse = input.mouse.button.is_some()
                || input.mouse.line_scroll.1 != 0.0;

            let mut handled = false;
            if self.focused.is_none() { 
                return false;
            }

            let mut command: Option<EditViewCommands> = None;
            let mut target: Option<ActionTarget> = None;
            if let edit_view = self.get_focused_view() {
                let mode = edit_view.mode();

                if should_keydown && input.key.is_some() {
                    if let Some((action, action_target)) 
                        = &self.process_keyboard_input(mode, input.modifiers, input.key.unwrap()) {
                            command = Some(EditViewCommands::Action(action.clone()));
                            target = Some(action_target.clone());
                    }
                }
            }
            if let edit_view = self.get_focused_view() {
                if command.is_some() && target.is_some() {
                    match target.unwrap() {
                        ActionTarget::EventLoop => {
                            match command.unwrap() {
                                EditViewCommands::Action(action) => {
                                    self.event_proxy.send_event(EditorEvent::Action(action));
                                },
                            }
                        },
                        _ => {
                            edit_view.poke_target(command.unwrap(), target.unwrap());
                        }
                    }
                    handled = true;
                }
                if should_mouse {
                    if input.mouse.line_scroll.1 != 0.0 {
                        edit_view.mouse_scroll(input.mouse.line_scroll.1);
                        handled = true;
                    }
                }
                // If focus changed, force dirty
                if input.window_focus_changed {
                    edit_view.set_dirty(true);
                    handled = true;
                }
            }

            handled
        } else {
            println!("unable to lock input in update_from_input");
            false
        }
    }
}
