use render::Renderer;

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub x: f32,
    pub y: f32,
}
pub type Position = Size;

pub trait Widget {
    fn size(&self) -> Size;
    fn position(&self) -> Position;
    fn queue_draw(&self, renderer: &mut Renderer);
    fn dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
}

impl From<(f32, f32)> for Size {
    fn from(tuple: (f32, f32)) -> Size {
        Position {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

