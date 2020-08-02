use crate::render::Renderer;
use super::text::TextWidget;
use super::primitive::PrimitiveWidget;
use super::status::StatusWidget;
use super::view::EditView;

pub enum WidgetKind {
    Text(TextWidget),
    Primitive(PrimitiveWidget),
    View(EditView),
    Status(StatusWidget),
}

pub trait Widget {
    fn index(&self) -> usize;
    fn position(&self) -> [f32; 2];
    fn size(&self) -> [f32; 2];
    fn queue_draw(&mut self, renderer: &mut Renderer);
    fn dirty(&self) -> bool;
}
