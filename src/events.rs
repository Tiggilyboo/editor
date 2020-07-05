use winit::event_loop::EventLoop;
use winit::event::VirtualKeyCode;

pub mod state;
use state::InputState;
use super::editor::EditorState;

use super::render;

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
    renderer: &render::Renderer,
) {
    let delta_time = editor_state.time_elapsed().as_secs_f32();

    if input_state.keycode.is_some() {
        match input_state.keycode.unwrap() {
            VirtualKeyCode::F1 => {
                editor_state.toggle_info()
            },
            VirtualKeyCode::F2 => {
                println!("Saving text cache texture to file...");
                renderer.write_cache_to_file();
            },
            _ => (),
        }
    }
}
