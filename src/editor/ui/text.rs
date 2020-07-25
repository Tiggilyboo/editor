use super::widget::Widget;
use crate::render::Renderer;

use glyph_brush::{
    Section,
    OwnedSection,
    Text,
    Layout,
};

use crate::editor::linecache::{
    Line,
    StyleSpan,
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
        Self {
            index,
            dirty: true,
            cursor: line.cursor().to_owned(),
            styles: line.styles().to_vec(),
            section: Section::default()
                .add_text(Text::new(trimmed_text)
                          .with_color(colour)
                          .with_scale(scale)
                          .with_z(0.5))
                .with_layout(Layout::default_single_line())
                .to_owned(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
       self.section.screen_position = (x, y);
       self.dirty = true;
    }

    pub fn get_section(&self) -> &OwnedSection {
        &self.section
    }

    pub fn get_cursor(&self) -> Vec<usize> {
        self.cursor.clone()
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
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

    fn size(&self) -> [f32; 2] {
        let bounds = self.section.bounds;
        [bounds.0, bounds.1]
    }
    
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        renderer.get_text_context().borrow_mut()
            .queue_text(&self.section.to_borrowed());
    }
}
