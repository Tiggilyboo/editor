
use winit::event::{
    VirtualKeyCode,
    ModifiersState,
    WindowEvent,
    ElementState,
    MouseButton,
    MouseScrollDelta,
};

#[derive(Debug)]
pub struct MouseState {
    pub state: Option<ElementState>,
    pub button: Option<MouseButton>,
    pub line_scroll: (f32, f32),
    pub position: (f64, f64),
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            state: None,
            button: None,
            line_scroll: (0.0, 0.0),
            position: (0.0, 0.0),
        }
    }
}

impl MouseState {
    pub fn from_window_event(input: WindowEvent) -> Self {
        let mut mouse = MouseState::default();

        match input {
            WindowEvent::MouseInput { state, button, .. } => {
                mouse.state = Some(state);
                mouse.button = Some(button);
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(x, y), .. } => {
                mouse.line_scroll = (x, y);
            },
            _ => return MouseState::default(),
        }

        mouse
    }
}

#[derive(Debug)]
pub struct InputState {
    pub keycode: Option<VirtualKeyCode>,
    pub modifiers: ModifiersState,
    pub mouse: MouseState,
}

impl InputState {
    pub fn from_window_event(event: WindowEvent) -> Self {
        Self {
            keycode: match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        input.virtual_keycode
                    } else {
                        None
                    }
                },
                _ => None,
            },
            modifiers: match event {
                WindowEvent::ModifiersChanged(mods) => {
                    mods
                },
                _ => ModifiersState::empty(),
            },
            mouse: MouseState::from_window_event(event), 
        }
    }
}


