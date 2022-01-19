use render::Renderer;

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub x: f32,
    pub y: f32,
}

pub type Position = Size;

pub trait Drawable { 
    fn dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn queue_draw(&self, renderer: &mut Renderer);
}

pub trait Widget {
    fn size(&self) -> Size;
    fn position(&self) -> Position;
    fn set_position(&mut self, x: f32, y: f32);
}

pub trait DrawableWidget: Widget + Drawable {
}

impl From<(f32, f32)> for Size {
    fn from(tuple: (f32, f32)) -> Size {
        Position {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

