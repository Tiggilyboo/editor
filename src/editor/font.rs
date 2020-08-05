use std::collections::HashMap;
use ab_glyph::*;
use glyph_brush::*;
use crate::unicode::*;

// Keeps track of the font metrics for caching glyph metrics
// Probably a much cleaner way of doing this...
pub struct FontContext {
    font: FontArc,
    bounds: HashMap<char, Rect>,
    font_size: f32,
}

fn initialise(font: FontArc) -> HashMap<char, Rect> {
    let mut bounds: HashMap<char, Rect> = HashMap::new();
    let included_categories = [
        LETTER_LOWERCASED,
        LETTER_UPPERCASE,
        LETTER_MODIFIER,
        LETTER_OTHER,
        SEPARATOR_SPACE,
        PUNCTUATION_CLOSE,
        PUNCTUATION_DASH,
        PUNCTUATION_FINAL_QUOTE,
        PUNCTUATION_INITIAL_QUOTE,
        PUNCTUATION_CONNECTOR,
        PUNCTUATION_OPEN,
        PUNCTUATION_OTHER,
        NUMBER_DECIMAL_DIGIT,
        NUMBER_LETTER,
        SYMBOL_MODIFIER,
        SYMBOL_CURRENCY,
        SYMBOL_MATH,
        SYMBOL_OTHER,
        MARK_ENCLOSING,
        MARK_NONSPACING,
    ];

    for category in included_categories.iter() {
        for ch in category.iter() {
            let glyph_id = font.glyph_id(*ch);
            bounds.insert(*ch, font.glyph_bounds(&glyph_id.with_scale(1.0)));
        }
    }

    bounds
}

impl FontContext {
    pub fn from(font: FontArc, font_size: f32) -> Self {
        Self {
            font: font.clone(),
            bounds: initialise(font.clone()),
            font_size,
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
            println!("could not find character '{}' in FontContext", ch);
            unreachable!()
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.font_size
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.font_size = scale;
    }
}

