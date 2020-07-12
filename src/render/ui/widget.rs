use crate::render::Renderer;
use super::text::TextWidget;
use crate::render::ui::view::EditView;

pub enum WidgetKind {
    Text(TextWidget),
    View(EditView),
}

pub trait Widget {
    fn index(&self) -> usize;
    fn position(&self) -> [f32; 2];
    fn queue_draw(&self, renderer: &mut Renderer);
    fn dirty(&self) -> bool;
}

