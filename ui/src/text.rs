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
    text_group: TextGroup,
    dirty: bool,
}


impl TextWidget {
    pub fn new(text: String, scale: f32, colour: ColourRGBA) -> Self {
        let mut text_group = TextGroup::new();
        text_group.push(text, scale, colour);

        Self {
            dirty: true,
            text_group, 
        } 
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.text_group.set_screen_position(x, y);
        self.dirty = true;
    }

    pub fn from_line(line: &Line, scale: f32, colour: ColourRGBA, styles: &HashMap<usize, Style>) -> Self {
        let mut text_group = TextGroup::new();

        if let Some(line_num) = line.ln {
            text_group.set_screen_position(0.0, (line_num - 1) as f32 * scale);
        }

        if let Some(text) = &line.text {
            let text = text.trim_end_matches(|c| c == '\r' || c == '\n');

            if line.styles.len() > 2 {
                println!("line style: {:?}", line.styles);
                println!("Styles: {:?}", styles);
                let mut ix = 0;
                for triple in line.styles.chunks(3) {
                    let mut start = ix + triple[0];
                    let mut end = start + triple[1];
                    let style_id = triple[2];

                    let text_len = text.len();
                    if start > text_len {
                        start = text_len;
                    }
                    if end > text_len {
                        end = text_len;
                    }
                    if start > end {
                        let t = end;
                        end = start;
                        start = t;
                    }

                    let content = &text[start as usize .. end as usize];

                    if let Some(style) = styles.get(&style_id) {
                        let fg_color = if let Some(fg) = style.fg_color {
                            fg.to_rgba_f32array()
                        } else {
                            colour
                        };

                        // Draw starting portions of the line
                        if start > 0 {
                            let beginning = &text[0 .. start as usize];
                            if beginning.len() > 1 {
                                text_group.push(beginning.into(), scale, fg_color);
                            }
                        }
                        // style selection
                        text_group.push(content.into(), scale, fg_color);

                        // Draw end portion of the line
                        if end < text_len {
                            let tail = &text[end as usize .. text_len];
                            if tail.len() > 1 {
                                text_group.push(tail.into(), scale, fg_color);
                            }
                        }

                    } else {
                        text_group.push(text.into(), scale, colour);
                    }

                    ix = end;
                }
            } else {
                text_group.push(text.into(), scale, colour);
            }
        }
        
        Self {
            text_group, 
            dirty: true,
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

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        renderer
            .get_text_context().borrow()
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
