use std::sync::{
    Arc,
    Mutex,
};
use std::collections::HashMap;
use super::ui::{
    view::EditView,
};
use crate::events::{
    state::InputState,
    mapper_winit::map_input_into_string,
};
use super::rpc::Theme;

pub type ViewId = String;

pub struct EditorState {
    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>, 
    pub available_themes: Option<Vec<String>>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            focused: Default::default(),
            views: HashMap::new(),
            available_themes: None,
        }
    }
    
    pub fn get_focused_view(&mut self) -> &mut EditView {
        let view_id = self.focused.clone()
            .expect("no focused EditView");

        self.views.get_mut(&view_id)
            .expect("Focused EditView not found in views")
    }

    

    pub fn set_available_themes(&mut self, themes: Vec<String>) {
        println!("set available themes: {:?}", themes);
        self.available_themes = Some(themes);
    }

    pub fn get_available_themes(&self) -> Option<&Vec<String>> {
        self.available_themes.as_ref()
    }

    pub fn update_from_input(&mut self, input: Arc<Mutex<InputState>>) -> bool {
        if let Ok(ref input) = input.clone().try_lock() {
            let should_keydown = input.keycode.is_some() 
                || input.modifiers.ctrl() || input.modifiers.shift() || input.modifiers.alt();
            let should_mouse = input.mouse.button.is_some()
                || input.mouse.line_scroll.1 != 0.0;

            let mut handled = false;
            if self.focused.is_some() { 
                let edit_view = self.get_focused_view();

                if should_keydown {
                    if let Some(input_string) = map_input_into_string(input.modifiers, input.keycode) {
                        let ch = input_string.chars().next().unwrap();
                        handled = edit_view.char(ch);
                    } else if input.keycode.is_some(){
                        handled = edit_view.keydown(input.keycode.unwrap(), input.modifiers);
                    }
                }
                if should_mouse {
                    if input.mouse.line_scroll.1 != 0.0 {
                        edit_view.mouse_scroll(input.mouse.line_scroll.1);
                        handled = true;
                    }
                }
            };

            handled
        } else {
            println!("unable to lock input in update_from_input");
            false
        }
    }
}
