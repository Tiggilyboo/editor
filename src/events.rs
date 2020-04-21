use std::sync::Arc;
use std::cell::RefCell;

use winit::event::{
    Event,
    WindowEvent,
};
use winit::event_loop::EventLoop;
use winit::event::VirtualKeyCode;
use crate::render::Renderer;

pub mod state;
use state::InputState;
use super::editor::EditorState;

pub type EditorEventLoop = EventLoop<EditorEvent>;

#[derive(Debug)]
pub struct LineItem {
    index: usize,
    item: Option<String>,
}

#[derive(Debug)]
pub enum EditorEvent {
    LineUpdate(LineItem),
}

pub fn create_event_loop<'a>() -> EditorEventLoop {
    EventLoop::<EditorEvent>::with_user_event()
}

pub fn handle_input(
    state: &mut EditorState,
    event: WindowEvent,
) {
    match event {
        WindowEvent::KeyboardInput { .. }
        | WindowEvent::MouseInput { .. }
        | WindowEvent::MouseWheel { .. }
        | WindowEvent::ModifiersChanged(_) => {

            let input_state = InputState::from_window_event(event);

            if input_state.keycode.is_some() {
                match input_state.keycode.unwrap() {
                    VirtualKeyCode::W => state.move_camera((0.0, 1.0, 0.0)),
                    VirtualKeyCode::A => state.move_camera((1.0, 0.0, 0.0)),
                    VirtualKeyCode::S => state.move_camera((0.0, -1.0, 0.0)),
                    VirtualKeyCode::D => state.move_camera((-1.0, 0.0, 0.0)),
                    _ => (),
                }
            }
            if input_state.mouse.line_scroll.1 != 0f32 {
                state.zoom(input_state.mouse.line_scroll.1);
            }
        },
        _ => (),
    }
}
