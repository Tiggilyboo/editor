pub mod state;
mod mapper_winit;

use winit::event_loop::EventLoop;
use winit::event::VirtualKeyCode;
use mapper_winit::input_into_string;

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
) {
    let delta_time = editor_state.time_elapsed().as_secs_f32();

    if input_state.keycode.is_some() {
        match input_state.keycode.unwrap() {
            VirtualKeyCode::F1 => {
                editor_state.toggle_info();
            },
            _ => {
                let input_string = input_into_string(input_state.modifiers, input_state.keycode);
                if input_string.is_some() {
                    let widget = editor_state.widgets.iter_mut()
                        .next().unwrap();

                    match widget {
                        WidgetKind::Text(text_widget) => {
                            let mut content = String::from(text_widget.content());
                            content.push_str(input_string.unwrap().as_str());
                            println!("content is now: {}", content);

                            text_widget.set_content(content.as_str());
                            text_widget.set_dirty(true);
                        },
                        _ => (),
                    }
                    


                }
            },
        }
    }
}
