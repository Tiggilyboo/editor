use std::collections::HashMap;
use ab_glyph::*;
use glyph_brush::*;
use crate::unicode::*;


// Keeps track of the font metrics for caching glyph metrics
// Probably a much cleaner way of doing this...
pub struct FontContext {
    font: FontArc,
    bounds: HashMap<char, Rect>,
}

impl FontContext {
    pub fn from(font: FontArc) -> Self {
        let font = font.clone();
        let mut bounds: HashMap<char, Rect> = HashMap::new();
        let included_categories = [
            LETTER_LOWERCASED,
            LETTER_UPPERCASE,
            PUNCTUATION_CLOSE,
            PUNCTUATION_DASH,
            PUNCTUATION_FINAL_QUOTE,
            PUNCTUATION_INITIAL_QUOTE,
            PUNCTUATION_CONNECTOR,
            PUNCTUATION_OPEN,
            NUMBER_DECIMAL_DIGIT,
            NUMBER_LETTER,
            SYMBOL_CURRENCY,
            SYMBOL_MATH,
        ];

        for category in included_categories.iter() {
            for ch in category.iter() {
                let glyph_id = font.glyph_id(*ch);
                bounds.insert(*ch, font.glyph_bounds(&glyph_id.with_scale(1.0)));
            }
        }

        Self {
            font,
            bounds,
        }
    }

    pub fn get_char_bounds(&self, ch: char) -> &Rect {
        self.bounds.get(&ch)
            .expect("could not find character in FontContext")
    }
}

