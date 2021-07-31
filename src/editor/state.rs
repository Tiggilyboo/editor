use std::sync::{
    Arc,
    Mutex,
};
use std::collections::HashMap;

use winit::event::{
    ModifiersState,
    VirtualKeyCode,
};
use winit::event_loop::EventLoopProxy;
use xi_core_lib::plugins::Command;

use super::ui::view::EditView;
use rpc::{ 
    PluginId,
    Style,
    Mode,
    Action,
    ActionTarget,
};
use super::view_commands::EditViewCommands;
use crate::events::{
    EditorEvent,
    EditorEventLoopProxy,
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
use super::plugins::PluginState;

pub type ViewId = String;
pub type BindingMap = HashMap<VirtualKeyCode, Vec<KeyBinding>>;

fn construct_bindingmap(bindings: Vec<KeyBinding>) -> BindingMap {
    let mut map = BindingMap::new();

    for b in bindings.iter() {
        let vc = match b.get_trigger() {
            Key::KeyCode(vkc) => vkc.clone(),
            _ => unimplemented!(),
        };
        if let Some(ref mut stored_bindings) = map.get_mut(&vc) {
            stored_bindings.push(b.clone());
        } else {
            map.insert(vc, vec![b.clone()]);
        }
    }

    map
}

pub struct EditorState {
    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>, 
    themes: Vec<String>,
    languages: Vec<String>, 
    styles: HashMap<usize, Style>,
    plugins: HashMap<PluginId, PluginState>, 
    key_bindings: HashMap<VirtualKeyCode, Vec<KeyBinding>>,
    mouse_bindings: Vec<MouseBinding>,
    event_proxy: EditorEventLoopProxy,
}

impl EditorState {
    pub fn new(event_proxy: EventLoopProxy<EditorEvent>) -> Self {
        Self {
            focused: Default::default(),
            views: HashMap::new(),
            plugins: HashMap::new(),
            styles: HashMap::new(),
            themes: vec![],
            languages: vec![],
            mouse_bindings: default_mouse_bindings(),
            key_bindings: construct_bindingmap(default_key_bindings()),
            event_proxy,
        }
    }


    pub fn get_event_proxy(&self) -> EditorEventLoopProxy {
        self.event_proxy.clone()
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
        for (_, view) in &mut self.views.iter_mut() {
            view.poke(EditViewCommands::SetPlugins(self.plugins.clone()));
        }
    }
    pub fn define_style(&mut self, style: Style) {
        if self.styles.contains_key(&style.id) {
            self.styles.remove(&style.id);
        }
        self.styles.insert(style.id, style);

        for(_, view) in &mut self.views.iter_mut() {
            view.poke(EditViewCommands::SetStyles(self.styles.clone()));
        }
    }
    pub fn get_styles(&self) -> HashMap<usize, Style> {
        self.styles.clone()
    }
    pub fn get_plugin(&self, plugin_id: PluginId) -> Option<PluginState> {
        if let Some(plugin) = self.plugins.get(&plugin_id) {
            Some(plugin.clone())
        } else {
            None
        }
    }
    pub fn set_plugin_commands(&mut self, plugin_id: PluginId, commands: Vec<Command>) {
        if let Some(ref mut plugin) = &mut self.plugins.get_mut(&plugin_id) {
            plugin.commands = commands;
        }
    }

    pub fn align_views_horizontally(&mut self, screen_size: [f32; 2]) {
        let view_count = self.views.len();
        let view_height = if view_count > 0 {
            screen_size[1] / view_count as f32
        } else {
            screen_size[1]
        };
        
        let mut view_top = 0.0;
        for (_, view) in self.views.iter_mut() {
            view.poke(EditViewCommands::Position([0.0, view_top]));
            view.poke(EditViewCommands::Resize([screen_size[0], view_height]));

            view_top += view_height;
        }
    }

    pub fn process_keyboard_input(&self, 
        mode: Mode, modifiers: ModifiersState, key: Key
    ) -> Option<(Vec<Action>, ActionTarget)> {
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

        if let Some(keycode_options) = self.key_bindings.get(&kc.unwrap()) {
            for binding in keycode_options.iter() {
                if binding.is_triggered_by(mode, modifiers, &Key::KeyCode(kc.unwrap())) {
                    return Some((
                        binding.get_actions(),
                        binding.get_target(),
                    ));
                }
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

            let mut actions: Vec<Action> = vec!();
            let mut target: Option<ActionTarget> = None;
            if let edit_view = self.get_focused_view() {
                let mode = edit_view.mode();

                if should_keydown && input.key.is_some() {
                    if let Some((bound_actions, action_target)) 
                        = &self.process_keyboard_input(mode, input.modifiers, input.key.unwrap()) {
                            actions = bound_actions.clone();
                            target = Some(action_target.clone());
                    }
                }
            }
            if actions.len() > 0 && target.is_some() {
                match target.unwrap() {
                    ActionTarget::EventLoop => {
                        for action in actions.iter() {
                            match self.event_proxy.send_event(EditorEvent::Action(action.clone())) {
                                Ok(_) => (),
                                Err(err) => println!("unable to send event to event_loop: {}", err),
                            }
                        }
                    },
                    ActionTarget::FocusedView | ActionTarget::StatusBar => {
                        if let edit_view = self.get_focused_view() {
                            for action in actions.iter() {
                                edit_view.poke_target(EditViewCommands::Action(action.clone()), target.unwrap());
                            }
                        }
                    }
                }
                handled = true;
            }

            if let edit_view = self.get_focused_view() {
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
