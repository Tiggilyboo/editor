use super::widget::Widget;
use crate::render::Renderer;

use glyph_brush::{
    Layout,
    Section,
};

use crate::editor::linecache::{
    Line,
    StyleSpan,
};
use crate::render::text::TextContext;
use glyph_brush::{
    OwnedSection,
    Text,
};

pub struct TextWidget {
    index: usize,
    dirty: bool,
    section: OwnedSection,
    cursor: Vec<usize>,
    styles: Vec<StyleSpan>,
}

impl TextWidget {
    pub fn from_line(index: usize, line: &Line, scale: f32, colour: [f32; 4]) -> Self {
        let text = line.text();
        let trimmed_text = text.trim_end_matches(|c| c == '\r' || c == '\n');
        let section = Section::default()
            .add_text(Text::new(trimmed_text)
                      .with_scale(scale)
                      .with_color(colour))
            .with_bounds((f32::INFINITY, scale))
            .with_layout(Layout::default())
            .to_owned();

        Self {
            index,
            dirty: true,
            section,
            cursor: line.cursor().to_owned(),
            styles: line.styles().to_vec(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
       self.section.screen_position = (x, y);
       self.dirty = true;
    }

    pub fn hit_test(&mut self, text_context: &TextContext, x: f32, y: f32) -> usize {
        text_context.hit_test(&self.section.to_borrowed(), x, y)
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = true;
    }
}

impl Widget for TextWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        let pos = self.section.screen_position;
        [pos.0, pos.1]
    }
    
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer.queue_text(&self.section.to_borrowed());
    }
}
