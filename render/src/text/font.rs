use std::collections::HashMap;
use glyph_brush::ab_glyph::{ 
    Rect, 
    Font,
    FontArc,
};
use super::unicode;

pub struct FontBounds {
    bounds: HashMap<char, Rect>,
    font_size: f32,
}

impl FontBounds {
    pub fn new(font: FontArc, font_size: f32) -> Self {
        let mut bounds = HashMap::new();
        for category in unicode::default_categories().iter() {
            for ch in category.iter() {
                let glyph_id = font.glyph_id(*ch);
                bounds.insert(*ch, font.glyph_bounds(&glyph_id.with_scale(1.0)));
            }
        }
        Self {
            font_size,
            bounds,
        }
    }

    pub fn get_char_bounds(&self, ch: char) -> Rect {
        if let Some(bounds) = self.bounds.get(&ch) {
            let mut bounds = bounds.clone();
            bounds.min.x *= self.font_size;
            bounds.max.x *= self.font_size;
            bounds.min.y *= self.font_size;
            bounds.min.x *= self.font_size;
            bounds
        } else {
            println!("could not find character '{}' in FontBounds", ch);
            unreachable!();
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.font_size
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.font_size = scale;
    }
    
    pub fn get_text_width(&self, text: &str) -> f32 {
        let mut w: f32 = 0.0;
        for (_, ch) in text.char_indices() {
            match ch {
                '\n'
                | '\r' => continue,
                _ => {
                    let bounds = self.get_char_bounds(ch);
                    w += bounds.max.x;
                }
            }
        }

        w
    }
}
