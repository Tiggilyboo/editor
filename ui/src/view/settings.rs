
pub struct ViewWidgetSettings {
    pub show_gutter: bool,
    pub show_mode: bool,
    pub show_filepath: bool,
    pub show_line_info: bool,
}

impl Default for ViewWidgetSettings {
    fn default() -> Self {
        Self {
            show_gutter: true,
            show_mode: true,
            show_filepath: true,
            show_line_info: true,
        }
    }
}

impl ViewWidgetSettings {
    pub fn show_status(&self) -> bool {
        self.show_mode || self.show_filepath || self.show_line_info
    }
}

