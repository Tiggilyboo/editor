use crate::colour::ColourRGBA;

use glyph_brush::{
    OwnedSection,
    OwnedText,
    Layout,
    BuiltInLineBreaker,
};

pub struct TextGroup {
    section: OwnedSection,
}

impl TextGroup {
    pub fn new() -> Self {
       let section = OwnedSection::default()
           .with_layout(Layout::default_single_line());

       Self {
           section,
       }
    }

    pub fn set_multiline(&mut self, multiline: bool) {
        if multiline {
            self.section.layout = Layout::default()
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker);
        } else {
            self.section.layout = Layout::default_single_line();
        }
    }

    pub fn get_section(&self) -> &OwnedSection {
        &self.section
    }

    pub fn push(&mut self, text: String, scale: f32, colour: ColourRGBA) {
        let new_text = OwnedText::new(&text)
          .with_color(colour)
          .with_scale(scale);

        self.section.text.push(new_text);
    }

    pub fn clear(&mut self) {
        self.section.text = vec![];
    }

    pub fn screen_position(&self) -> (f32, f32) {
        self.section.screen_position
    }

    pub fn set_screen_position(&mut self, x: f32, y: f32) {
        self.section.screen_position = (x, y);
    }

    pub fn bounds(&self) -> (f32, f32) {
        self.section.bounds
    }

    pub fn set_linewrap_width(&mut self, width: f32) {
        let old_bounds = self.bounds();
        self.section.bounds = (width, old_bounds.1).into();
    }

    pub fn set_text(&mut self, text: String, scale: f32, colour: ColourRGBA) {
        self.clear();
        self.push(text, scale, colour);
    }

    pub fn set_scale(&mut self, scale: f32) {
        for t in self.section.text.iter_mut() {
            t.scale = scale.into();
        }
    }
}
