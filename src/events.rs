use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event::VirtualKeyCode;

pub mod state;
use state::InputState;
use super::editor::EditorState;

use super::render::ui::widget::WidgetKind;

pub type EditorEventLoop = EventLoop<EditorEvent>;

pub enum EditorEvent {
    OpenWidget(WidgetKind),
}

pub fn create_event_loop() -> EditorEventLoop {
    EventLoop::<EditorEvent>::with_user_event()
}

pub fn handle_input(
    state: &mut EditorState,
    event: WindowEvent,
    window_dimensions: [f32; 2],
) {
    let input_state = InputState::from_window_event(event);

    if input_state.keycode.is_some() {
        match input_state.keycode.unwrap() {
            VirtualKeyCode::W 
                | VirtualKeyCode::Up => state.move_camera((0.0, 1.0, 0.0)),
            VirtualKeyCode::A 
                | VirtualKeyCode::Left => state.move_camera((1.0, 0.0, 0.0)),
            VirtualKeyCode::S
                | VirtualKeyCode::Down => state.move_camera((0.0, -1.0, 0.0)),
            VirtualKeyCode::D 
                | VirtualKeyCode::Right => state.move_camera((-1.0, 0.0, 0.0)),
            VirtualKeyCode::F1 => state.toggle_info(),
            _ => (),
        }
    }
    if input_state.mouse.line_scroll.1 != 0f32 {
        state.zoom(input_state.mouse.line_scroll.1);
    }
    if input_state.mouse.position != (0.0, 0.0) {
        state.pitch(input_state.mouse.position.1 as f32 / window_dimensions[1] * 90.0f32);
    }
}
