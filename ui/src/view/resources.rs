use render::colour::ColourRGBA;
use eddy::styles::ToRgbaFloat32;
use crate::view::ThemeStyleMap;

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
    pub fn from(style_map: &ThemeStyleMap) -> Self {
        let default = style_map.get_default_style();
        let settings = style_map.get_theme_settings();

        println!("Theme settings: {:?}", settings);

        let foreground = if let Some(fg) = settings.foreground {
            fg.to_rgba_f32array()
        } else {
            default.fg_color.unwrap().to_rgba_f32array()
        };
        let background = if let Some(bg) = settings.background {
            bg.to_rgba_f32array()
        } else {
            default.bg_color.unwrap().to_rgba_f32array()
        };
        let caret = if let Some(cr) = settings.caret {
            cr.to_rgba_f32array()
        } else {
            foreground
        };
        let selection = if let Some(sl) = settings.selection_foreground {
            sl.to_rgba_f32array()
        } else {
            background
        };
        let selection_bg = if let Some(sl) = settings.selection {
            sl.to_rgba_f32array()
        } else {
            foreground
        };
        let gutter_bg = if let Some(gt) = settings.gutter {
            gt.to_rgba_f32array()
        } else {
            background
        };
        let gutter = if let Some(gt) = settings.gutter_foreground {
            gt.to_rgba_f32array()
        } else {
            foreground
        };

        Self {
            foreground,
            background,
            caret,
            selection,
            selection_bg,
            gutter,
            gutter_bg
        }
    }
}
