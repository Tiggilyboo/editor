use super::widget::Widget;
use crate::render::Renderer;
use crate::render::primitive::Primitive;

pub struct PrimitiveWidget {
    index: usize,
    position: [f32; 2],
    depth: f32,
    size: [f32; 2],
    colour: [f32; 4],
    dirty: bool,
}

impl PrimitiveWidget {
    pub fn new(index: usize, position: [f32; 3], size: [f32; 2], colour: [f32; 4]) -> Self {
        Self {
            index,
            position: [position[0], position[1]],
            depth: position[2],
            size,
            colour,
            dirty: true,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position[0] = x;
        self.position[1] = y;
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
        self.dirty = true;
    }

    pub fn set_height(&mut self, height: f32) {
        self.size[1] = height;
        self.dirty = true;
    }
    pub fn set_width(&mut self, width: f32) {
        self.size[0] = width;
        self.dirty = true;
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn set_colour(&mut self, colour: [f32; 4]) {
        self.colour = colour;
    }
}

impl Widget for PrimitiveWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        renderer.get_primitive_context().borrow_mut()
            .queue_primitive(self.index, Primitive {
                top_left: self.position,
                bottom_right: self.size,
                depth: self.depth,
                colour: self.colour,
            });
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}