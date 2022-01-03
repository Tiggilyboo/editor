use render::colour::ColourRGBA;
use eddy::styles::ToRgbaFloat32;
use crate::view::{
    ThemeStyleMap,
    ThemeSettings,
};

#[derive(Debug)]
pub struct ViewResources {
    pub background: ColourRGBA,
    pub foreground: ColourRGBA,
    pub caret: ColourRGBA,
    pub selection: ColourRGBA,
    pub selection_bg: ColourRGBA,
    pub gutter: ColourRGBA,
    pub gutter_bg: ColourRGBA,
}

impl Default for ViewResources {
    fn default() -> Self {
        Self {
            background: [0.1, 0.1, 0.1, 1.0], 
            foreground: [0.9, 0.9, 0.9, 1.0],
            caret: [1.0, 1.0, 1.0, 1.0],
            selection_bg: [1.0, 1.0, 1.0, 0.3],
            selection: [0.1, 0.1, 0.1, 1.0],
            gutter: [0.7, 0.7, 0.7, 1.0],
            gutter_bg: [0.2, 0.2, 0.2, 1.0],
        }
    }
}

impl ViewResources {
    pub fn update_theme(&mut self, style_map: &ThemeStyleMap, theme_settings: &ThemeSettings) {
        let default = style_map.get_default_style();
        
        self.foreground = if let Some(fg) = theme_settings.foreground {
            fg.to_rgba_f32array()
        } else {
            default.fg_color.unwrap().to_rgba_f32array()
        };
        self.background = if let Some(bg) = theme_settings.background {
            bg.to_rgba_f32array()
        } else {
            default.bg_color.unwrap().to_rgba_f32array()
        };
        self.caret = if let Some(cr) = theme_settings.caret {
            cr.to_rgba_f32array()
        } else {
            self.foreground
        };
        self.selection = if let Some(sl) = theme_settings.selection_foreground {
            sl.to_rgba_f32array()
        } else {
            self.background
        };
        self.selection_bg = if let Some(sl) = theme_settings.selection {
            sl.to_rgba_f32array()
        } else {
            self.foreground
        };
        self.gutter_bg = if let Some(gt) = theme_settings.gutter {
            gt.to_rgba_f32array()
        } else {
            self.background
        };
        self.gutter = if let Some(gt) = theme_settings.gutter_foreground {
            gt.to_rgba_f32array()
        } else {
            self.foreground
        };
    }

    pub fn from(style_map: &ThemeStyleMap) -> Self {
        let settings = style_map.get_theme_settings();
        let mut resources = Self::default();

        resources.update_theme(style_map, settings);

        resources
    }
}
