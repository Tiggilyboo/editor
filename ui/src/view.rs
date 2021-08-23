use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
};

use render::{
    Renderer,
    colour::BLACK,
};
use eddy::{
    ViewId,
    line_cache::LineCache,
    styles::Style,
};
use super::widget::{
    Widget,
    Size,
    Position,
};
use super::tree::WidgetTree;
use super::text::TextWidget;

pub struct ViewWidget {
    view_id: ViewId,
    size: Size,
    position: Position,
    filepath: Option<String>,
    widgets: WidgetTree,
    dirty: bool,
}

impl Widget for ViewWidget {
    fn position(&self) -> Position {
        self.position
    }
    fn size(&self) -> Size {
        self.size
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        self.widgets.queue_draw(renderer);
    }
}

impl ViewWidget {
    pub fn new(view_id: ViewId, filepath: Option<String>) -> Self {
        let widgets = WidgetTree::new();

        Self {
            view_id,
            size: Size::default(),
            position: Position::default(),
            filepath,
            widgets,
            dirty: true,
        }
    }

    pub fn populate(&mut self, line_cache: &LineCache, styles: Arc<Mutex<HashMap<isize, Style>>>) {
        let styles = styles.clone();

        if let Ok(styles) = styles.try_lock() {
            for ix in 0..line_cache.height() {
                if let Some(line) = line_cache.get_line(ix) {
                    let line_widget = TextWidget::from_line(&line, 1.0, BLACK, &styles);

                    self.widgets.push(Box::new(line_widget));
                }
            }
        }

        self.dirty = true;
    }
}
