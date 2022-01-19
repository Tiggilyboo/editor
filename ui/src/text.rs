use crate::widget::{
    Widget,
    Drawable,
    Position,
    Size,
};
use eddy::{
    line_cache::Line,
    styles::{
        ThemeStyleMap,
        ToRgbaFloat32,
    },
};
use super::primitive::PrimitiveWidget;

use render::{
    Renderer,
    text::TextGroup,
    colour::ColourRGBA,
};

pub struct TextWidget {
    text_group: TextGroup,
    background: Option<PrimitiveWidget>,
    dirty: bool,
    pad_left: f32, 
    pad_right: f32,
}

impl TextWidget {
    pub fn new() -> Self {
        Self {
            dirty: true,
            text_group: TextGroup::new(), 
            background: None,
            pad_left: 0.0,
            pad_right: 0.0,
        } 
    }

    pub fn with_text(text: String, scale: f32, colour: ColourRGBA) -> Self {
        let mut text_group = TextGroup::new();
        text_group.push(text, scale, colour);

        Self {
            dirty: true,
            text_group, 
            background: None,
            pad_left: 0.0,
            pad_right: 0.0,
        } 
    }

    pub fn with_multiline() -> Self {
        let mut text_group = TextGroup::new();
        text_group.set_multiline(true);

        Self {
            dirty: true,
            text_group,
            background: None,
            pad_left: 0.0,
            pad_right: 0.0,
        }
    }

    pub fn with_background(background_colour: ColourRGBA, depth: f32) -> Self {
        let mut me = Self::new();
        me.set_background(background_colour, depth);

        me
    }

    pub fn set_background(&mut self, background_colour: ColourRGBA, depth: f32) {
        let background = PrimitiveWidget::new(
            self.position(), 
            self.size(),
            depth,
            background_colour);

        self.background = Some(background);
    }

    pub fn set_padding(&mut self, pad_left: f32, pad_right: f32) {
        self.pad_left = pad_left;
        self.pad_right = pad_right; 
    }

    pub fn populate(&mut self, texts: Vec<String>, scale: f32, colour: ColourRGBA) {
        self.text_group.clear();
        for t in texts.iter() {
            self.text_group.push(t.into(), scale, colour);
        }

        self.set_dirty(true);
    }

    pub fn from_line(line: &Line, scale: f32, style_map: &ThemeStyleMap) -> Self {
        let mut text_group = TextGroup::new();

        if let Some(text) = &line.text {
            let text = text.trim_end_matches(|c| c == '\r' || c == '\n');
            let default_style = style_map.get_default_style();
            let def_fg_color = default_style.fg_color.unwrap().to_rgba_f32array();
            let def_sel_color = if let Some(sel_color) = style_map.get_theme_settings().selection {
                sel_color.to_rgba_f32array()
            } else {
                def_fg_color    
            };

            if line.styles.len() > 2 {
                println!("line style: {:?}", line.styles);
                let mut ix = 0;
                for triple in line.styles.chunks(3) {
                    let start = ix + triple[0];
                    let mut end = start + triple[1];
                    let style_id = triple[2];

                    let text_len = text.len();
                    if end > text_len {
                        end = text_len;
                    }

                    // Determine the selection colour
                    let fg_color = if style_id == 0 {
                        def_sel_color
                    } else {
                        if let Some(style) = style_map.get(style_id) {
                            if let Some(fg) = style.fg_color {
                                fg.to_rgba_f32array()
                            } else {
                                def_fg_color
                            }
                        } else {
                            def_fg_color
                        }
                    };

                    // Draw starting portions of the line
                    if start > 0 {
                        let beginning = &text[.. start as usize];
                        if beginning.len() > 0 {
                            text_group.push(beginning.into(), scale, def_fg_color);
                        }
                    }

                    // style selection
                    let content = &text[start as usize .. end as usize];
                    text_group.push(content.into(), scale, fg_color);

                    // Draw end portion of the line
                    if end > start && end != text_len {
                        let tail = &text[end as usize ..];
                        if tail.len() > 0 {
                            text_group.push(tail.into(), scale, def_fg_color);
                        }
                    }

                    ix = end;
                }
            } else {
                text_group.push(text.into(), scale, def_fg_color);
            }
        }
        
        Self {
            text_group, 
            dirty: true,
            background: None,
            pad_left: 0.0,
            pad_right: 0.0,
        } 
    }
    

    pub fn set_scale(&mut self, scale: f32) {
        self.text_group.set_scale(scale);
    }

    pub fn set_text(&mut self, text: String, scale: f32, colour: ColourRGBA) {
        self.text_group.set_text(text, scale, colour);
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.set_linewrap_width(width);
        if let Some(bg) = &mut self.background {
            bg.set_size(self.pad_left + width + self.pad_right, height);
        }
    }

    pub fn set_linewrap_width(&mut self, width: f32) {
        self.text_group.set_linewrap_width(width);
        self.dirty = true;
    }

}

impl Widget for TextWidget {
    fn position(&self) -> Position {
        self.text_group.screen_position().into()
    }
    
    fn set_position(&mut self, x: f32, y: f32) {
        self.text_group.set_screen_position(x + self.pad_left, y);
        if let Some(bg) = &mut self.background {
            bg.set_position(x, y);
        }
        self.dirty = true;
    }

    fn size(&self) -> Size {
        let mut size: Size = self.text_group.bounds().into();
        size.x += self.pad_left + self.pad_right;

        size
    }
}
impl Drawable for TextWidget {
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        if let Some(bg) = &self.background {
            bg.queue_draw(renderer);
        }
        renderer
            .get_text_renderer().borrow()
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
