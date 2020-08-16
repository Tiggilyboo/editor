use std::hash::{
    Hash,
    Hasher,
};
use std::collections::HashMap;
use rpc::{
    Theme,
    Style,
    theme::ToRgbaFloat32,
};
use super::ui::colour::{
    ColourRGBA,
    BLANK, 
};

pub struct Resources {
    pub fg: ColourRGBA,
    pub bg: ColourRGBA,
    pub sel: ColourRGBA,
    pub cursor: ColourRGBA,
    pub gutter_fg: ColourRGBA,
    pub gutter_bg: ColourRGBA,
    pub scale: f32,
    pub styles: HashMap<usize, Style>,
}

impl Hash for Resources {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.bg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.sel.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.gutter_fg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.gutter_bg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.scale.to_le_bytes().hash(state);
    }
}
impl Resources {
    pub fn new(scale: f32) -> Self {
        Self {
            fg: BLANK,
            bg: BLANK,
            sel: BLANK,
            cursor: BLANK,
            gutter_bg: BLANK,
            gutter_fg: BLANK,
            scale,
            styles: HashMap::new(),
        }
    }

    #[inline]
    pub fn line_gap(&self) -> f32 {
        self.scale * 1.06
    }
    pub fn pad(&self) -> f32 {
        self.scale * 0.25
    }

    pub fn update_from_theme(&mut self, theme: Theme) {
        if let Some(col) = &theme.foreground {
            self.fg = col.to_rgba_f32array();
        }
        if let Some(col) = &theme.background {
            self.bg = col.to_rgba_f32array(); 
        }
        if let Some(col) = &theme.caret {
            self.cursor = col.to_rgba_f32array();
        }
        if let Some(col) = &theme.selection {
            self.sel = col.to_rgba_f32array();
        } else {
            self.sel = self.cursor;
        }
        if let Some(col) = &theme.gutter {
            self.gutter_bg = col.to_rgba_f32array();
        } else {
            self.gutter_bg = self.bg;
        }
        if let Some(col) = &theme.gutter_foreground {
            self.gutter_fg = col.to_rgba_f32array();
        } else {
            self.gutter_fg = self.fg;
        }
    }

}
