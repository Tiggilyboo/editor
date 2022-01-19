use super::widget::{
    Size,
    Position,
    Widget,
    Drawable,
};

use render::{
    Renderer,
    primitive::Primitive,
    colour::ColourRGBA,
};

#[derive(Debug)]
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
    
    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }
}

impl Drawable for PrimitiveWidget {
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer
            .get_primitive_renderer().borrow_mut()
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

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size.x = width;
        self.size.y = height;
    }


    pub fn set_colour(&mut self, colour: ColourRGBA) {
        self.colour = colour;
    }

    pub fn colour(&self) -> &ColourRGBA {
        &self.colour
    }
}
