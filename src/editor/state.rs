use std::collections::HashMap;
use std::time::{
    Instant,
    Duration,
};

use crate::render::ui::{
    view::EditView,
    widget::Widget,
    widget::WidgetKind,
};

pub type ViewId = String;

pub struct EditorState {
    show_info: bool,
    time: Instant,

    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>,
    pub widgets: Vec<WidgetKind>, 
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            show_info: true,
            widgets: vec!(),
            time: Instant::now(),
            focused: Default::default(),
            views: HashMap::new(),
        }
    }
    
    pub fn time_elapsed(&self) -> Duration {
        self.time.elapsed()
    }

    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }

    pub fn add_widget(&mut self, widget: WidgetKind) {
        match widget {
            WidgetKind::Text(text_widget) => {
                self.widgets.insert(text_widget.index(), WidgetKind::Text(text_widget));
            },
            WidgetKind::View(view_widget) => {
                self.widgets.insert(view_widget.index(), WidgetKind::View(view_widget));
            },
        }
    }

    pub fn get_focused_view(&self) -> &mut EditView {
        let view_id = self.focused.clone()
            .expect("no focused EditView");

        self.views.get_mut(&view_id)
            .expect("Focused EditView not found in views")
    }

    
}
