use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
};

use render::{
    Renderer,
    colour::ColourRGBA,
    text::FontBounds,
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
use super::primitive::PrimitiveWidget;

pub const CURSOR_TEXT: &str = "\u{2588}";

pub struct ViewResources {
    pub bg_colour: ColourRGBA,
    pub fg_colour: ColourRGBA,
}

pub struct ViewWidget {
    view_id: ViewId,
    size: Size,
    position: Position,
    first_line: usize,
    height: usize,
    filepath: Option<String>,
    line_widgets: WidgetTree<TextWidget>,
    background: PrimitiveWidget,
    cursor_widgets: Vec<TextWidget>,
    resources: Arc<Mutex<ViewResources>>,
    font_bounds: Arc<Mutex<FontBounds>>,
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

    fn set_dirty(&mut self, dirty: bool) {
        self.line_widgets.set_dirty(dirty);
        self.dirty = dirty;
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        self.background.queue_draw(renderer);
        self.line_widgets.queue_draw(renderer);

        for cw in self.cursor_widgets.iter() {
            cw.queue_draw(renderer);
        }
    }
}

impl ViewWidget {
    pub fn new(
        view_id: ViewId, 
        filepath: Option<String>, 
        resources: Arc<Mutex<ViewResources>>, 
        font_bounds: Arc<Mutex<FontBounds>>,
) -> Self {
        let line_widgets = WidgetTree::<TextWidget>::new();
        let bg_colour = resources.lock().unwrap().bg_colour;
        let background = PrimitiveWidget::new(Position::default(), Size::default(), 0.0, bg_colour);

        Self {
            view_id,
            filepath,
            line_widgets,
            background,
            resources,
            font_bounds,
            cursor_widgets: Vec::new(),
            size: Size::default(),
            position: Position::default(),
            first_line: 0,
            height: 0,
            dirty: true,
        }
    }

    pub fn view_id(&self) -> ViewId {
        self.view_id
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.size.x = width;
        self.size.y = height;
        self.set_dirty(true);
    }

    fn calculate_cursors(&mut self, line_cache: &LineCache) {
        let colour = self.resources.lock().unwrap().fg_colour;
        let cursor_selections = line_cache.get_line_selections(0);

        self.height = line_cache.height();
        self.cursor_widgets.clear();

        if let Ok(font_bounds) = self.font_bounds.try_lock() {
            let scale = font_bounds.get_scale();

            for selection in cursor_selections.iter() {
                if let Some(selected_line) = line_cache.get_line(selection.line_num) {
                    if let Some(selected_text) = &selected_line.text {
                        let start = selection.start_col;
                        let end = selection.end_col;
                        assert!(start <= end);

                        let selected_text = if start == end {
                            &selected_text[..end]
                        } else {
                            &selected_text[start..end]
                        };
                        let x = font_bounds.get_text_width(selected_text);
                        let y = selection.line_num as f32 * scale;

                        let mut cursor_widget = TextWidget::new(CURSOR_TEXT.into(), scale, colour);
                        cursor_widget.set_position(x, y);
                        println!("cursor position (selection: {}): {}, {}", selected_text, x, y);

                        self.cursor_widgets.push(cursor_widget);
                    }
                }
            }
        }
    }

    pub fn populate(&mut self, line_cache: &LineCache, styles: Arc<Mutex<HashMap<usize, Style>>>) {
        let scale = self.font_bounds.lock().unwrap().get_scale();
        let colour = self.resources.lock().unwrap().fg_colour;

        if let Ok(styles) = styles.try_lock() {
            for ix in 0..line_cache.height() {
                if let Some(line) = line_cache.get_line(ix) {
                    let line_widget = TextWidget::from_line(&line, scale, colour, &styles);
                    self.line_widgets.insert(ix, line_widget);
                }
            }
        }

        self.calculate_cursors(line_cache);

        self.dirty = true;
    }

    pub fn measure_text(&self, text: String) -> f32 {
        self.font_bounds.lock().unwrap().get_text_width(&text) 
    }

    pub fn scroll(&mut self, line: usize, col: usize) {
        self.first_line = line;
        
        let scale = self.font_bounds.lock().unwrap().get_scale();

        for ix in self.first_line..self.height {
            let y = self.position.y + (ix as f32 * scale);
            if let Some(line_widget) = self.line_widgets.get_mut(ix) {
                line_widget.set_position(self.position.x, y);
            }
        }
    }
}

impl Default for ViewResources {
    fn default() -> Self {
        Self {
            bg_colour: [0.3, 0.3, 0.3, 1.0], 
            fg_colour: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
