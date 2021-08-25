use super::widget::{
    Size,
    Position,
    Widget,
};

use render::{
    Renderer,
    primitive::Primitive,
    colour::ColourRGBA,
};


pub struct PrimitiveWidget {
    position: Position,
    size: Size,
    colour: ColourRGBA,
    depth: f32,
    dirty: bool,
}

impl Widget for PrimitiveWidget {
    fn position(&self) -> Position {
        self.position
    }
    
    fn size(&self) -> Size {
        self.size
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer
            .get_primitive_context().borrow_mut()
            .queue_primitive(Primitive {
                top_left: [self.position.x, self.position.y],
                bottom_right: [
                    self.position.x + self.size.x,
                    self.position.y + self.size.y,
                ],
                depth: self.depth,
                colour: self.colour,
            });
    }
}

impl PrimitiveWidget {
    pub fn new(position: Position, size: Size, depth: f32, colour: ColourRGBA) -> Self {
        Self {
            position,
            size,
            depth,
            colour,
            dirty: true,
        }
    }
}
