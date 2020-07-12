use std::cell::RefCell;

use widget::WidgetKind;
use view::EditView;
use super::Renderer;
use super::text::TextContext;
use widget::Widget;
use crate::editor::EditorState;

pub mod widget;
pub mod text;
pub mod view;

pub fn create_initial_ui_state(screen_size: [f32; 2]) -> Vec<WidgetKind> {
    #[inline]
    fn white() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }

    vec![
        WidgetKind::View(EditView::new(0, screen_size, 20.0)),
    ]
}

pub fn update_ui(editor_state: &mut EditorState, renderer: &mut Renderer) {
    let mut requires_redraw = false;

    for w in editor_state.widgets.iter_mut() {
        match w {
            WidgetKind::Text(text_widget) => {
                if text_widget.dirty() {
                    text_widget.queue_draw(renderer);
                    requires_redraw = true;
                }
            },
            WidgetKind::View(view) => {
                if view.dirty() {
                    view.queue_draw(renderer);
                    requires_redraw = true;
                }
            },
        }
    }

    if requires_redraw {
        renderer.request_redraw();
    }
}
