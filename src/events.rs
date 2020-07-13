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
                let should_keydown = input_state.keycode.is_some();
                let should_mouse = input_state.mouse.button.is_some()
                    || input_state.mouse.line_scroll.1 != 0.0;

                if !should_keydown && !should_mouse {
                    println!("!keydown && !mouse");
                    return;
                }

                if let Some(widget_view) = &mut editor_state.widgets.iter_mut().filter_map(|w| {
                    match w {
                        WidgetKind::View(view) => Some(view),
                        _ => None,
                    }
                }).next() {
                    if should_keydown {
                        if let Some(input_string) = input_into_string(input_state.modifiers, input_state.keycode) {
                            let ch = input_string.chars().next().unwrap();
                            widget_view.char(ch);
                        } else {
                            widget_view.keydown(input_state.keycode.unwrap(), input_state.modifiers);
                        }
                    }
                    if should_mouse {
                        widget_view.mouse_scroll(input_state.mouse.line_scroll.1);
                    }
                }
            },
        }
    }
}
