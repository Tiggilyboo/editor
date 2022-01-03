pub mod settings;
pub mod resources;
mod gutter;
mod status;

pub use resources::ViewResources;

use std::collections::BTreeMap;
use std::sync::{
    Arc,
    Mutex,
};
use std::ops::Range;

use settings::ViewWidgetSettings;
use gutter::GutterWidget;
use status::StatusWidget;

use render::{
    Renderer,
    text::FontBounds,
};
use eddy::{
    ViewId,
    line_cache::{
        Line,
        LineCache,
        Selection,
    },
    styles::{
        ThemeStyleMap,
        ThemeSettings,
    },
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

pub struct ViewWidget {
    view_id: ViewId,
    position: Position,
    dirty: bool,
    first_line: usize,
    last_line: usize,
    current_line: usize,
    scroll_offset: f32,
    settings: ViewWidgetSettings,
    
    // widgets
    line_widgets: WidgetTree<TextWidget>,
    background: PrimitiveWidget,
    selection_widgets: BTreeMap<usize, Vec<PrimitiveWidget>>,
    cursor_widgets: Vec<TextWidget>,
    gutter: GutterWidget,
    status: StatusWidget,

    // shared
    resources: Arc<Mutex<ViewResources>>,
    font_bounds: Arc<Mutex<FontBounds>>,
}

impl Widget for ViewWidget {
    fn position(&self) -> Position {
        self.position
    }

    fn size(&self) -> Size {
        self.background.size()
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.background.set_dirty(dirty);
        self.line_widgets.set_dirty(dirty);
        self.gutter.set_dirty(dirty);
        self.status.set_dirty(dirty);

        for cw in self.cursor_widgets.iter_mut() {
            cw.set_dirty(dirty); 
        }
        for (_, sw) in self.selection_widgets.iter_mut() {
            sw.iter_mut().for_each(|w| w.set_dirty(dirty));
        }

        self.dirty = dirty;
    }

    fn queue_draw(&self, renderer: &mut Renderer) {
        self.background.queue_draw(renderer);
        self.line_widgets.queue_draw(renderer);

        if self.settings.show_gutter {
            self.gutter.queue_draw(renderer);
        }
        if self.settings.show_status() {
            self.status.queue_draw(renderer);
        }

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
        let bg_colour = resources.lock().unwrap().background;
        let background = PrimitiveWidget::new(
            Position::default(), 
            Size::default(), 
            0.0, 
            bg_colour);

        let initial_font_scale = font_bounds.lock().unwrap().get_scale();
        let gutter_bg = resources.lock().unwrap().gutter_bg;
        let gutter_fg = resources.lock().unwrap().gutter;
        let gutter = GutterWidget::new(
            Position::default(),
            Size::default(),
            initial_font_scale,
            gutter_bg, gutter_fg);

        let mut status = StatusWidget::new(
            Position::default(),
            Size::default(),
            initial_font_scale,
            gutter_bg, gutter_fg);

        if let Some(filepath) = filepath.clone() {
            status.set_filepath(filepath);
        }

        let mut settings = ViewWidgetSettings::default();
        settings.show_filepath = filepath.is_some();

        Self {
            view_id,
            position: Position::default(),
            settings,
            line_widgets,
            selection_widgets,
            gutter,
            status,
            background,
            resources,
            font_bounds,
            cursor_widgets: Vec::new(),
            first_line: 0,
            last_line: 0,
            current_line: 0,
            scroll_offset: 0.0,
            dirty: true,
        }
    }

    pub fn view_id(&self) -> ViewId {
        self.view_id
    }

    pub fn get_scale(&self) -> f32 {
        self.font_bounds.lock().unwrap().get_scale()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        println!("resize view widget: {}, {}", width, height);
        self.background.set_size(width, height);
        self.gutter.set_height(height);
        self.scroll_to(self.first_line);
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

    pub fn update_from_resources(&mut self) {
        let bg_colour = self.resources.lock().unwrap().background;
        let current_colour = self.background.colour();

        if *current_colour != bg_colour {
            self.background.set_colour(bg_colour);
            self.background.set_dirty(true);
            self.set_dirty(true);
        }

        let gutter_bg = self.resources.lock().unwrap().gutter_bg;
        self.gutter.set_background(gutter_bg);
        self.status.set_background(gutter_bg);
    }

    fn populate_cursors(&mut self, line_cache: &LineCache, scale: f32) {
        self.cursor_widgets.clear();

        let selections = line_cache.get_selections();
        if selections.len() != 1 {
            return;
        }
        
        let caret = self.resources.lock().unwrap().caret;
        
        // Selection 0: Cursor
        let selection = selections[0];
        let line = line_cache.get_line(selection.line_num)
            .expect("selection line did not exist in cache");

        let y = self.line_to_content_y(selection.line_num);
        let x = self.gutter.size().x
            + self.measure_selection_width(line, selection);

        let mut cursor_widget = TextWidget::with_text(CURSOR_TEXT.into(), scale, caret);
        cursor_widget.set_position(x, y);
        self.cursor_widgets.push(cursor_widget);
    }

    pub fn populate(&mut self, line_cache: &LineCache, style_map: Arc<Mutex<ThemeStyleMap>>) {
        let scale = self.get_scale();
        let foreground = self.resources.lock().unwrap().foreground;
        let gutter_width = self.gutter.size().x;

        let mut gutter_pos_y = None;

        self.line_widgets.clear();

        if let Ok(style_map) = style_map.lock() {
            for ix in self.first_line..self.last_line {
                if let Some(line) = line_cache.get_line(ix) {
                    let y = self.line_to_content_y(ix);
                    let mut line_widget = TextWidget::from_line(&line, scale, &style_map);
                    line_widget.set_position(gutter_width, y);

                    self.line_widgets.insert(ix, line_widget);

                    if gutter_pos_y.is_none() {
                        gutter_pos_y = Some(y);
                    }
                }
            }
        } else {
            panic!("unable to lock style map in view");
        }

        if self.settings.show_gutter {
            // measure what the largest line number is and adjust the gutter width as required
            let largest_item_width = self.measure_text(self.last_line.to_string());

            self.gutter.set_position((0.0, gutter_pos_y.unwrap_or(0.0)).into());
            self.gutter.set_width(largest_item_width);
            self.gutter.update(self.first_line, self.last_line, scale, foreground);
        }

        if self.settings.show_status() {
            let h = self.size().y;
            if self.status.position().y != h - scale {
                self.status.set_position(0.0, h - scale);
                self.status.populate();
            }
        }

        self.populate_cursors(line_cache, scale);
        self.set_dirty(true);
    }

    pub fn measure_text(&self, text: String) -> f32 {
        self.font_bounds.lock().unwrap().get_text_width(&text) 
    }

    pub fn scroll_to(&mut self, line: usize) {
        let scale = self.get_scale();
        let h = self.size().y;
        let y = self.position.y + (line as f32 * scale);
        let inv_scroll = -self.scroll_offset;

        if line == 0 {
            self.scroll_offset = 0.0;
        } else if y < inv_scroll + (scale / 3.0) {
            // for scrolling up
            self.scroll_offset = -y + scale;
        } else if y > inv_scroll + h - scale - scale {
            // for scrolling down
            self.scroll_offset = -y + h - scale - scale;

            if self.settings.show_status() {
                self.scroll_offset -= scale;
            }
        }
        self.current_line = line;

        self.set_dirty(true);
        self.update_viewport();
    }

    fn line_to_content_y(&self, line: usize) -> f32 {
        self.position.y 
            + self.scroll_offset 
            + (line as f32 * self.get_scale()) 
    }

    fn y_to_line(&self, y: f32) -> usize {
        let scale = self.get_scale();
        let mut line = ((y - self.position.y - self.scroll_offset) / scale).floor();
        if line < 0.0 {
            line = 0.0;
        }

        line as usize
    }

    pub fn status(&mut self) -> &mut StatusWidget {
        &mut self.status
    }

    pub fn update_viewport(&mut self) {
        let height = self.size().y;
        let scale = self.get_scale();
        let first = self.y_to_line(self.position.y);

        // -1 line for status if visible
        let mut view_height = (height / scale).ceil() as i32;
        if self.settings.show_status() {
            if view_height > 0 {
                view_height -= 1;
            }
        }

        self.first_line = first;
        self.last_line = first + view_height as usize;
    }

    pub fn get_viewport(&self) -> Range<usize> {
        self.first_line..self.last_line 
    }
}

