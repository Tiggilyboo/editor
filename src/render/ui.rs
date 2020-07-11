use widget::WidgetKind;
use text::TextWidget;
use super::Renderer;
use widget::Widget;
use crate::editor::EditorState;

pub mod widget;
pub mod text;

pub fn create_initial_ui_state(screen_size: [f32; 2]) -> Vec<WidgetKind> {
    fn white() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }

    vec![
        WidgetKind::Text(TextWidget::new(0, String::default(), [20.0, 20.0], 20.0, white())),
    ]
}

pub fn update_ui(editor_state: &mut EditorState, renderer: &mut Renderer) {
    let mut requires_redraw = false;
    for w in editor_state.widgets.iter_mut() {
        match w {
            WidgetKind::Text(text_widget) => {
                if text_widget.dirty() {
                    println!("update_ui: queuing text for redraw");
                    text_widget.queue_draw(renderer);
                    text_widget.set_dirty(false);
                    requires_redraw = true;
                }
            },
            _ => (),
        }
    }

    if requires_redraw {
        renderer.request_redraw();
    }
}
