use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub type ColourRgba32 = u32;

#[derive(Debug, Clone, Deserialize)]
pub struct Style {
    pub id: usize,
    #[serde(rename="fg_color")]
    pub fg: Option<ColourRgba32>,
    #[serde(rename="bg_color")]
    pub bg: Option<ColourRgba32>,
    #[serde(default)]
    pub italic: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub enum UnderlineOption {
    None,
    Underline,
    StippledUnderline,
    SquigglyUnderline,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Theme {
    /// The default color for text.
    pub foreground: Option<Colour>,
    /// The default backgound color of the view.
    pub background: Option<Colour>,
    /// Colour of the caret.
    pub caret: Option<Colour>,
    /// Colour of the line the caret is in.
    /// Only used when the `higlight_line` setting is set to `true`.
    pub line_highlight: Option<Colour>,

    /// The color to use for the squiggly underline drawn under misspelled words.
    pub misspelling: Option<Colour>,
    /// The color of the border drawn around the viewport area of the minimap.
    /// Only used when the `draw_minimap_border` setting is enabled.
    pub minimap_border: Option<Colour>,
    /// A color made available for use by the theme.
    pub accent: Option<Colour>,
    /// CSS passed to popups.
    pub popup_css: Option<String>,
    /// CSS passed to phantoms.
    pub phantom_css: Option<String>,

    /// Colour of bracketed sections of text when the caret is in a bracketed section.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub bracket_contents_foreground: Option<Colour>,
    /// Controls certain options when the caret is in a bracket section.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub bracket_contents_options: Option<UnderlineOption>,
    /// Foreground color of the brackets when the caret is next to a bracket.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub brackets_foreground: Option<Colour>,
    /// Background color of the brackets when the caret is next to a bracket.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub brackets_background: Option<Colour>,
    /// Controls certain options when the caret is next to a bracket.
    /// Only applied when the match_brackets setting is set to `true`.
    pub brackets_options: Option<UnderlineOption>,

    /// Colour of tags when the caret is next to a tag.
    /// Only used when the `match_tags` setting is set to `true`.
    pub tags_foreground: Option<Colour>,
    /// Controls certain options when the caret is next to a tag.
    /// Only applied when the match_tags setting is set to `true`.
    pub tags_options: Option<UnderlineOption>,

    /// The border color for "other" matches.
    pub highlight: Option<Colour>,
    /// Background color of regions matching the current search.
    pub find_highlight: Option<Colour>,
    /// Text color of regions matching the current search.
    pub find_highlight_foreground: Option<Colour>,

    /// Background color of the gutter.
    pub gutter: Option<Colour>,
    /// Foreground color of the gutter.
    pub gutter_foreground: Option<Colour>,

    /// The background color of selected text.
    pub selection: Option<Colour>,
    /// A color that will override the scope-based text color of the selection.
    pub selection_foreground: Option<Colour>,

    /// Colour of the selection regions border.
    pub selection_border: Option<Colour>,
    /// The background color of a selection in a view that is not currently focused.
    pub inactive_selection: Option<Colour>,
    /// A color that will override the scope-based text color of the selection
    /// in a view that is not currently focused.
    pub inactive_selection_foreground: Option<Colour>,

    /// Colour of the guides displayed to indicate nesting levels.
    pub guide: Option<Colour>,
    /// Colour of the guide lined up with the caret.
    /// Only applied if the `indent_guide_options` setting is set to `draw_active`.
    pub active_guide: Option<Colour>,
    /// Colour of the current guide’s parent guide level.
    /// Only used if the `indent_guide_options` setting is set to `draw_active`.
    pub stack_guide: Option<Colour>,

    /// The color of the shadow used when a text area can be horizontally scrolled.
    pub shadow: Option<Colour>,
}

impl Colour {
    pub fn from_u32_rgba(value: ColourRgba32) -> Self {
        let rgba = value.to_le_bytes();
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    pub fn from_json(value: serde_json::Value) -> Option<Self> {
        if value.is_u64() {
            Some(Self::from_u32_rgba(value.as_u64().unwrap() as u32))
        } else if value.is_object() {
            Some(serde_json::from_value::<Colour>(value).unwrap())
        } else {
            None
        }
    }
}

pub trait ToRgbaFloat32 {
    fn to_rgba_f32array(&self) -> [f32; 4];
}

impl ToRgbaFloat32 for Colour {
    #[inline]
    fn to_rgba_f32array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

impl ToRgbaFloat32 for ColourRgba32 {
    fn to_rgba_f32array(&self) -> [f32; 4] {
        let value = self.to_le_bytes();
        [
            value[0] as f32 / 255.0,
            value[1] as f32 / 255.0,
            value[2] as f32 / 255.0,
            value[3] as f32 / 255.0,
        ]
    }
}
