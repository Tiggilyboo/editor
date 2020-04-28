
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
    pub fn update_via_window_event(&mut self, input: WindowEvent, window_dimensions: [f32; 2]) {
        match input {
            WindowEvent::MouseInput { state, button, .. } => {
                self.state = Some(state);
                self.button = Some(button);
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(x, y), .. } => {
                self.line_scroll = (x, y);
            },
            WindowEvent::CursorMoved { position, .. } => {
                let (half_x, half_y) = (window_dimensions[0] / 2.0, window_dimensions[1] / 2.0); 
                let (x, y) = (position.x as f32, position.y as f32);
                let x = x + half_x;
                let y = y - half_y;

                self.delta.0 = self.position.0 - x;
                self.delta.1 = self.position.1 - y;

                self.position.0 = x;
                self.position.1 = y;
            },
            _ => (),
        }
    }
}

#[derive(Debug)]
pub struct InputState {
    pub keycode: Option<VirtualKeyCode>,
    pub modifiers: ModifiersState,
    pub mouse: MouseState,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keycode: None,
            modifiers: ModifiersState::default(),
            mouse: MouseState::default(),
        }
    }

    pub fn update(&mut self, event: WindowEvent, window_dimensions: [f32; 2]) { 
        self.keycode = match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed {
                    input.virtual_keycode
                } else {
                    None
                }
            },
            _ => None,
        };
        self.modifiers = match event {
            WindowEvent::ModifiersChanged(mods) => {
                mods
            },
            _ => ModifiersState::empty(),
        };
        self.mouse.update_via_window_event(event, window_dimensions); 
    }
}


