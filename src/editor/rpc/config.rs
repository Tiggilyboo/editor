use serde::{
    Serialize,
    Deserialize,
};
use serde_json::{
    Value,
    json,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Config {
    pub font_face: Option<String>,
    pub font_size: Option<f32>,
    pub line_ending: Option<String>,
    pub plugin_search_path: Option<Vec<String>>,
    pub tab_size: Option<u64>,
    pub translate_tabs_to_spaces: Option<bool>,
    pub word_wrap: Option<bool>,
}
