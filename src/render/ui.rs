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
