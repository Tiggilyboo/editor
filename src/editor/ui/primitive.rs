use std::hash::{
    Hasher,
    Hash,
};
use super::widget::{
    hash_widget,
    Widget,
};
use super::colour::ColourRGBA;
use crate::render::Renderer;
use crate::render::primitive::Primitive;

pub struct PrimitiveWidget {
    index: usize,
    position: [f32; 2],
    depth: f32,
    size: [f32; 2],
    colour: ColourRGBA,
    dirty: bool,
}

impl PrimitiveWidget {
    pub fn new(index: usize, position: [f32; 3], size: [f32; 2], colour: ColourRGBA) -> Self {
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
        self.dirty = true;
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

    pub fn set_colour(&mut self, colour: ColourRGBA) {
        self.colour = colour;
        self.dirty = true;
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }

    pub fn colour(&self) -> ColourRGBA {
        self.colour
    }
}

impl Hash for PrimitiveWidget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state);
    }
}

impl Widget for PrimitiveWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        renderer
            .get_primitive_context().borrow_mut()
            .queue_primitive(self.index, Primitive {
                top_left: self.position,
                bottom_right: [
                    self.position[0] + self.size[0],
                    self.position[1] + self.size[1],
                ],
                depth: self.depth,
                colour: self.colour,
            });
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}
