use super::widget::Widget;
use crate::render::Renderer;
use crate::render::primitive::Primitive;

pub struct PrimitiveWidget {
    index: usize,
    position: [f32; 2],
    size: [f32, 2],
    dirty: bool,
}

impl PrimitiveWidget {
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position[0] = x;
        self.position[1] = y;
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

impl Widget for PrimitiveWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer.get_primitive_context().borrow_mut()
            .queue_primitive(Primitive {
                top_left: self.position,
                bottom_right: self.size,
                depth: 0.0,
                colour: self.colour,
            });
    }
}
