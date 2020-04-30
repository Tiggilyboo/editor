use winit::event_loop::EventLoop;
use winit::event::VirtualKeyCode;

pub mod state;
use state::InputState;
use super::editor::EditorState;

use super::render::ui::widget::WidgetKind;

pub enum EditorEvent {
    OpenWidget(WidgetKind), 
}

pub type EditorEventLoop = EventLoop<EditorEvent>;

pub fn create_event_loop() -> EditorEventLoop {
    EventLoop::<EditorEvent>::with_user_event()
}

pub fn handle_input(
    editor_state: &mut EditorState,
    input_state: &InputState,
    window_dimensions: [f32; 2],
) {
    let delta_time = editor_state.time_elapsed().as_secs_f32();

    if input_state.keycode.is_some() {
        match input_state.keycode.unwrap() {
            VirtualKeyCode::W 
                | VirtualKeyCode::Up => editor_state.translate_camera((0.0, 1.0), delta_time),
            VirtualKeyCode::A 
                | VirtualKeyCode::Left => editor_state.translate_camera((-1.0, 0.0), delta_time),
            VirtualKeyCode::S
                | VirtualKeyCode::Down => editor_state.translate_camera((0.0, -1.0), delta_time),
            VirtualKeyCode::D 
                | VirtualKeyCode::Right => editor_state.translate_camera((1.0, 0.0), delta_time),
            VirtualKeyCode::F1 => editor_state.toggle_info(),
            _ => (),
        }
    }
    if input_state.mouse.line_scroll.1 != 0f32 {
        editor_state.zoom(input_state.mouse.line_scroll.1);
    }
    if input_state.mouse.position != (0.0, 0.0) {
        editor_state.camera_direction(input_state.mouse.delta, delta_time);
    }
}
