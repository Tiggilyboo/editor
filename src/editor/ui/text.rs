use super::widget::Widget;
use crate::render::Renderer;

use glyph_brush::{
    Layout,
    Section,
    OwnedSection,
    Text,
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

fn create_section(line: &Line, scale: f32, colour: [f32; 4]) -> OwnedSection {
    let text = line.text();
    let trimmed_text = text.trim_end_matches(|c| c == '\r' || c == '\n');

    Section::default()
        .add_text(Text::new(trimmed_text)
                  .with_scale(scale)
                  .with_color(colour))
        .with_bounds((f32::INFINITY, scale))
        .with_layout(Layout::default_single_line())
        .to_owned()
}

impl TextWidget {
    pub fn from_line(index: usize, line: &Line, scale: f32, colour: [f32; 4]) -> Self {
        Self {
            index,
            dirty: true,
            section: create_section(line, scale, colour),
            cursor: line.cursor().to_owned(),
            styles: line.styles().to_vec(),
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
        renderer.get_text_context().borrow_mut()
            .queue_text(&self.section.to_borrowed());
    }
}
