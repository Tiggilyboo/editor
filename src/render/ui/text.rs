use super::widget::Widget;
use crate::render::Renderer;

#[derive(Debug, Clone)]
pub struct TextWidget {
    index: usize,
    content: String,
    font_size: f32,
    position: [f32; 2],
    size: [f32; 2],
    colour: [f32; 4],
}

impl TextWidget {
    pub fn new(index: usize, content: String, position: [f32; 2], font_size: f32, colour: [f32; 4]) -> Self {
        Self {
            index,
            position,
            content,
            colour,
            font_size,
            size: [0.0, font_size],
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = String::from(content);
    }
}

impl Widget for TextWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position 
    }

    fn draw(&self, renderer: &mut Renderer) {
        renderer.queue_text(
            self.position,
            self.colour,
            self.font_size,
            self.content.as_str()
        );
    }
}
