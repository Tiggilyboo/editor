use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

use super::widget::{
    Widget,
    hash_widget,
};
use super::colour::ColourRGBA;
use crate::render::Renderer;

use glyph_brush::{
    Section,
    OwnedSection,
    Text,
    Layout,
};
use crate::editor::linecache::Line;
use rpc::{
    Style,
    theme::ToRgbaFloat32,
};

pub struct TextWidget {
    index: usize,
    dirty: bool,
    section: OwnedSection,
    cursor: Vec<usize>,
}

impl Hash for TextWidget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state); 
        self.cursor.hash(state);
    }
}

impl TextWidget {
    pub fn from_line(index: usize, line: &Line, scale: f32, colour: ColourRGBA, styles: &HashMap<usize, Style>) -> Self {
        let text = line.text().trim_end_matches(|c| c == '\r' || c == '\n');
        let section = Section::default()
            .with_layout(Layout::default_single_line());

        let mut texts: Vec<Text> = vec!();
        if line.styles().len() > 0 {
            for style_span in line.styles().iter() {
                let start = style_span.range.start;
                let end = if style_span.range.end > text.len() {
                    text.len()
                } else {
                    style_span.range.end
                };
                if start > end {
                    continue;
                }
                let content = &text[start..end];
                if let Some(style) = styles.get(&style_span.style_id) {
                    let colour = if let Some(fg) = &style.fg {
                        fg.to_rgba_f32array()
                    } else {
                        colour
                    };
                    
                    texts.push(Text::new(content)
                        .with_color(colour)
                        .with_scale(scale)
                    );
                }
            }
        } else {
            texts.push(Text::new(text)
                .with_color(colour)
                .with_scale(scale)
            );
        }

        Self {
            index,
            dirty: true,
            cursor: line.cursor().to_owned(),
            section: section.with_text(texts).to_owned(),
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
