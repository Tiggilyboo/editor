use std::collections::HashMap;
use crate::widget::{
    Widget,
    Position,
    Size,
};
use eddy::{
    line_cache::Line,
    styles::{
        Style,
        ToRgbaFloat32,
    },
};

use render::{
    Renderer,
    text::TextGroup,
    colour::ColourRGBA,
};

pub struct TextWidget {
    dirty: bool,
    cursor: Option<Vec<usize>>,
    text_group: TextGroup,
}

impl TextWidget {
    pub fn new(text: String, scale: f32, colour: ColourRGBA) -> Self {
        let mut text_group = TextGroup::new();
        text_group.push(text, scale, colour);

        Self {
            dirty: true,
            cursor: None,
            text_group, 
        } 
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.text_group.set_screen_position(x, y);
    }
    
    // TODO: Do this when recieving UpdateOp
    pub fn from_line(line: &Line, scale: f32, colour: ColourRGBA, styles: &HashMap<isize, Style>) -> Self {
        let mut text_group = TextGroup::new();

        if let Some(text) = &line.text {
            let text = text.trim_end_matches(|c| c == '\r' || c == '\n');

            let mut ix = 0;
            for triple in line.styles.chunks(3) {
                let start = ix + triple[0];
                let end = start + triple[1];
                let style_id = triple[2];

                let content = &text[start as usize .. end as usize];

                if let Some(style) = styles.get(&style_id) {
                    if let Some(fg) = style.fg_color {
                        text_group.push(content.into(), scale, fg.to_rgba_f32array());
                    }
                } else {
                    text_group.push(content.into(), scale, colour);
                }

                ix = end;
            }
        }

        Self {
            dirty: true,
            cursor: Some(line.cursors.to_owned()),
            text_group, 
        } 
    }
}

impl Widget for TextWidget {
    fn position(&self) -> Position {
        self.text_group.screen_position().into()
    }

    fn size(&self) -> Size {
        self.text_group.bounds().into()
    }
    
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer
            .get_text_context().borrow_mut()
            .queue_text(&self.text_group);
    }
}

/// Counts the number of utf-16 code units in the given string.
pub fn count_utf16(s: &str) -> usize {
    let mut utf16_count = 0;
    for &b in s.as_bytes() {
        if (b as i8) >= -0x40 { utf16_count += 1; }
        if b >= 0xf0 { utf16_count += 1; }
    }
    utf16_count
}