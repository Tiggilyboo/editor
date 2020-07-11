use crate::render::Renderer;
use super::text::TextWidget;

#[derive(Clone)]
pub enum WidgetKind {
    Text(TextWidget),
    Button,
}

pub trait Widget {
    fn index(&self) -> usize;
    fn position(&self) -> [f32; 2];
    fn queue_draw(&self, renderer: &mut Renderer);
    fn dirty(&self) -> bool;
}

