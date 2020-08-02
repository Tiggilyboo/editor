use std::sync::{
    Arc,
    Mutex,
};
use std::collections::HashMap;

use winit::event::ModifiersState;

use super::ui::{
    view::EditView,
};
use super::rpc::{
    EditViewCommands,
    Style,
};
use crate::events::{
    state::InputState,
    mapper_winit::map_scancode,
    binding::{
        Action,
        Key,
        KeyBinding,
        MouseBinding,
        Mode,
        default_mouse_bindings,
        default_key_bindings,
    },
};

pub type ViewId = String;

pub struct EditorState {
    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>, 
    available_themes: Option<Vec<String>>,
    available_languages: Option<Vec<String>>, 
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            focused: Default::default(),
            views: HashMap::new(),
            available_themes: None,
            available_languages: None,
            mouse_bindings: default_mouse_bindings(),
            key_bindings: default_key_bindings(),
        }
    }
    
    pub fn get_focused_view(&mut self) -> &mut EditView {
        let view_id = self.focused.clone()
            .expect("no focused EditView");

        self.views.get_mut(&view_id)
            .expect("Focused EditView not found in views")
    }

    pub fn set_available_themes(&mut self, themes: Vec<String>) {
        self.available_themes = Some(themes);
    }
    pub fn set_available_languages(&mut self, languages: Vec<String>) {
        self.available_languages = Some(languages);
    }

    pub fn process_keyboard_input(&self, mode: Mode, modifiers: ModifiersState, key: Key) -> Option<Action> {
        let kc = match key {
            Key::KeyCode(virtual_keycode) => Some(virtual_keycode),
            Key::ScanCode(scancode) => map_scancode(scancode),
        };
        println!("process_keyboard_input - mode: {:?}, key: {:?}", mode, key);
        if kc.is_none() {
            return None;
        }
        if self.focused.is_none() {
            return None;
        }

        for binding in self.key_bindings.iter() {
            if binding.is_triggered_by(mode, modifiers, &Key::KeyCode(kc.unwrap())) {
                println!("action triggered - mode: {:?}, (s c a): ({}, {}, {}), input: {:?}", 
                         mode, modifiers.shift(), modifiers.ctrl(), modifiers.alt(), kc);

                return Some(binding.get_action());
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
            if let edit_view = self.get_focused_view() {
                let mode = edit_view.mode();

                if should_keydown && input.key.is_some() {
                    if let Some(action) = &self.process_keyboard_input(mode, input.modifiers, input.key.unwrap()) {
                        command = Some(EditViewCommands::Action(action.clone()));
                    }
                }
            }
            if let edit_view = self.get_focused_view() {
                if command.is_some() {
                    edit_view.poke(command.unwrap());
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
