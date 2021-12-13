use std::collections::HashMap;
use std::collections::BTreeMap;
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
    line_cache::{
        Line,
        LineCache,
        Selection,
    },
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
    pub cr_colour: ColourRGBA,
    pub sl_colour: ColourRGBA,
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
    selection_widgets: BTreeMap<usize, Vec<PrimitiveWidget>>,
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
        self.background.set_dirty(dirty);
        self.line_widgets.set_dirty(dirty);
        for cw in self.cursor_widgets.iter_mut() {
            cw.set_dirty(dirty); 
        }

        self.dirty = dirty;
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        self.background.queue_draw(renderer);
        self.line_widgets.queue_draw(renderer);

        for (_, sels) in self.selection_widgets.iter() {
            for sw in sels.iter() {
                sw.queue_draw(renderer); 
            }
        }

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
        let selection_widgets = BTreeMap::new();
        let bg_colour = resources.lock().unwrap().bg_colour;
        let background = PrimitiveWidget::new(
            Position::default(), 
            Size::default(), 
            0.0, 
            bg_colour);

        Self {
            view_id,
            filepath,
            line_widgets,
            selection_widgets,
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
        self.background.set_size(width, height);
        self.set_dirty(true);
    }

    fn measure_selection_width(&self, line: &Line, selection: &Selection) -> f32 {
        if let Some(text) = &line.text {
            let text_left_of_cursor = if text.len() > selection.start_col {
                &text.as_str()[..selection.start_col]
            } else {
                &text.as_str()
            };
            
            self.font_bounds.lock().unwrap()
                .get_text_width(text_left_of_cursor)

        } else {
            0.0
        }
    }

    fn populate_selections(&mut self, line_cache: &LineCache, scale: f32) {
        self.selection_widgets.clear();

        let colour = self.resources.lock().unwrap().sl_colour;
        let selections = line_cache.get_selections();
        if selections.len() == 0 {
            return;
        }

        // Skip the first, as it is always the cursor
        for sel in selections.iter().skip(1) {
            if let Some(line) = line_cache.get_line(sel.line_num) {
                let y = sel.line_num as f32 * scale;
                let x = self.measure_selection_width(line, sel);
        
                let highlight = PrimitiveWidget::new(
                    (x, y).into(), 
                    (self.position.x, scale).into(), 
                    1.0, 
                    colour);

                if let Some(widgets) = self.selection_widgets.get_mut(&sel.line_num) {
                    widgets.push(highlight);
                } else {
                    self.selection_widgets.insert(sel.line_num, vec![highlight]);
                }
            }
        }
    }

    fn populate_cursors(&mut self, line_cache: &LineCache, scale: f32) {
        let colour = self.resources.lock().unwrap().cr_colour;

        self.cursor_widgets.clear();

        let selections = line_cache.get_selections();
        if selections.len() == 0 {
            return;
        }
        
        // Selection 0: Cursor
        let selection = selections[0];
        let line = line_cache.get_line(selection.line_num);
        if let Some(line) = line {

            let mut cursor_widget = TextWidget::new(CURSOR_TEXT.into(), scale, colour);
            let y = selection.line_num as f32 * scale;
            let x = self.measure_selection_width(line, selection);

            cursor_widget.set_position(x, y);         
            self.cursor_widgets.push(cursor_widget);
        } else {
            panic!("Got selection without line number in cache!");
        }
    }

    pub fn populate(&mut self, line_cache: &LineCache, styles: Arc<Mutex<HashMap<usize, Style>>>) {
        let scale = self.font_bounds.lock().unwrap().get_scale();
        let colour = self.resources.lock().unwrap().fg_colour;
        
        self.height = line_cache.height();

        if let Ok(styles) = styles.try_lock() {
            for ix in self.first_line..self.height {
                if let Some(line) = line_cache.get_line(ix) {
                    let line_widget = TextWidget::from_line(&line, scale, colour, &styles);
                    self.line_widgets.insert(ix, line_widget);
                }
            }
        }

        self.populate_selections(line_cache, scale);
        self.populate_cursors(line_cache, scale);
        self.set_dirty(true);
    }

    pub fn measure_text(&self, text: String) -> f32 {
        self.font_bounds.lock().unwrap().get_text_width(&text) 
    }

    // TODO: if col > width of screen, move the difference
    pub fn scroll(&mut self, line: usize, col: usize) {
        self.first_line = line;
        
        let scale = self.font_bounds.lock().unwrap().get_scale();

        for ix in self.first_line..self.height {
            let y = self.position.y + (ix as f32 * scale);
            if let Some(line_widget) = self.line_widgets.get_mut(ix) {
                line_widget.set_position(self.position.x, y);
                line_widget.set_dirty(true);
            }
            if let Some(selections_on_line) = self.selection_widgets.get_mut(&ix) {
                for sel_w in selections_on_line.iter_mut() {
                    sel_w.set_position(self.position.x, y);
                    sel_w.set_dirty(true);
                }
            }
        }
    }
}

impl Default for ViewResources {
    fn default() -> Self {
        Self {
            bg_colour: [0.1, 0.1, 0.1, 1.0], 
            fg_colour: [0.9, 0.9, 0.9, 1.0],
            cr_colour: [1.0, 1.0, 1.0, 1.0],
            sl_colour: [1.0, 1.0, 1.0, 0.3],
        }
    }
}
