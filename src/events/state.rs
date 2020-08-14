
use winit::event::{
    ModifiersState,
    WindowEvent,
    ElementState,
    MouseButton,
    MouseScrollDelta,
};
use super::mapper_winit::map_scancode;
use super::binding::Key;

#[derive(Debug)]
pub struct MouseState {
    pub state: Option<ElementState>,
    pub button: Option<MouseButton>,
    pub line_scroll: (f32, f32),
    pub position: (f32, f32),
    pub delta: (f32, f32),
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            state: None,
            button: None,
            line_scroll: (0.0, 0.0),
            position: (0.0, 0.0),
            delta: (0.0, 0.0),
        }
    }
}

impl MouseState {
    // update_via_window_event returns true when the state has changed
    pub fn update_via_window_event(&mut self, input: WindowEvent, window_dimensions: [f32; 2]) -> bool {
        match input {
            WindowEvent::MouseInput { state, button, .. } => {
                let change = self.state != Some(state) || self.button != Some(button);

                self.state = Some(state);
                self.button = Some(button);
                change
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(x, y), .. } => {
                let change = self.line_scroll != (x, y);
                self.line_scroll = (x, y);
                change
            },
            WindowEvent::CursorMoved { position, .. } => {
                let (half_x, half_y) = (window_dimensions[0] / 2.0, window_dimensions[1] / 2.0); 
                let (x, y) = (position.x as f32, position.y as f32);
                let x = half_x - x;
                let y = half_y - y;

                self.delta.0 = self.position.0 - x;
                self.delta.1 = self.position.1 - y;

                let _change = self.position.0 != x || self.position.1 != y;
                self.position.0 = x;
                self.position.1 = y;
                //change
                false
            },
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct InputState {
    pub key: Option<Key>,
    pub modifiers: ModifiersState,
    pub mouse: MouseState,
    pub window_focused: bool,
    pub window_focus_changed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key: None,
            modifiers: ModifiersState::default(),
            mouse: MouseState::default(),
            window_focused: false,
            window_focus_changed: false,
        }
    }

    // update the input state with passed events
    // returns whether the state has changed or not
    pub fn update(&mut self, event: WindowEvent, window_dimensions: [f32; 2]) -> bool { 
        let old_key = self.key.clone();
        let old_mods = self.modifiers;

        self.key = match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed {
                    // Check the scancode override (Different between operating systems, X11,
                    // Wayland, etc)
                    if let Some(mapped_scancode) = map_scancode(input.scancode) {
                        Some(Key::KeyCode(mapped_scancode))
                    } else if input.virtual_keycode.is_some() {
                        Some(Key::KeyCode(input.virtual_keycode.unwrap()))
                    } else {
                        println!("scancode not mapped: {}", input.scancode);
                        Some(Key::ScanCode(input.scancode))
                    }
                } else if input.state == ElementState::Released {
                    None 
                } else {
                    old_key
                }
            },
            _ => old_key,
        };
        self.modifiers = match event {
            WindowEvent::ModifiersChanged(mods) => {
                mods
            },
            _ => old_mods,
        };

        match event {
            WindowEvent::Focused(focus) => {
                self.window_focused = focus;
                self.window_focus_changed = true;
            },
            _ => {
                self.window_focus_changed = false;
            },
        };

        let mouse_changed = self.mouse.update_via_window_event(event, window_dimensions); 

        // Change detected?
        old_key.is_none() && self.key.is_some()
            || old_key.is_some() && self.key.is_some()
            || old_mods != self.modifiers
            || mouse_changed
            || self.window_focus_changed
    }
}


