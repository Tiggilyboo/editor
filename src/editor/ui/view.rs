use std::ops::Range;
use std::sync::{
    Mutex,
    Weak,
};

use glyph_brush::{
    OwnedSection,
    Section,
    Layout,
    Text,
};

use crate::events::binding::{
    Action,
    Mode,
    Motion,
};
use crate::editor::linecache::LineCache;
use serde_json::{
    json,
    Value,
};

use crate::render::Renderer;
use crate::editor::rpc::{
    Core,
    Config,
    Theme,
    EditViewCommands,
};
use super::{
    widget::Widget,
    text::TextWidget,
    primitive::PrimitiveWidget,
};

type Method = String;
type Params = Value;
type ColourRGBA = [f32; 4];

struct Resources {
    fg: ColourRGBA,
    bg: ColourRGBA,
    sel: ColourRGBA,
    cursor: ColourRGBA,
    gutter_fg: ColourRGBA,
    gutter_bg: ColourRGBA,
    scale: f32,
    line_gap: f32,
}

pub struct EditView {
    index: usize,
    dirty: bool,
    view_id: Option<String>,
    filename: Option<String>,
    line_cache: LineCache,
    scroll_offset: f32,
    viewport: Range<usize>,
    core: Weak<Mutex<Core>>,
    pending: Vec<(Method, Params)>,
    config: Option<Config>,
    theme: Option<Theme>,
    size: [f32; 2],
    resources: Resources,
    gutter: PrimitiveWidget,
    background: PrimitiveWidget,
    mode: Mode,
    show_line_numbers: bool,
}

const TOP_PAD: f32 = 6.0;
const LEFT_PAD: f32 = 6.0;

#[inline]
fn rgba8_to_rgba32(colour_u8: [u8; 4]) -> [f32; 4] {
    [
        colour_u8[0] as f32 / 255.0,
        colour_u8[1] as f32 / 255.0,
        colour_u8[2] as f32 / 255.0,
        colour_u8[3] as f32 / 255.0,
    ]
}

impl Widget for EditView {
    fn index(&self) -> usize {
        self.index 
    }

    fn position(&self) -> [f32; 2] {
        [TOP_PAD, LEFT_PAD]
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        let text_ctx = renderer.get_text_context().clone();

        let first_line = self.y_to_line(0.0);
        let last_line = std::cmp::min(self.y_to_line(self.size[1]) + 1, self.line_cache.height());
        
        // Figure out the maximum width of the line number
        let scale = self.resources.scale;
        let gutter_width = if self.show_line_numbers {
            LEFT_PAD + LEFT_PAD 
                + text_ctx.borrow().get_text_width(last_line.to_string().clone().as_str())
        } else {
            0.0
        };
        let x0 = LEFT_PAD + gutter_width;
        let mut y = self.line_to_content_y(first_line) - self.scroll_offset;

        let line_gap = self.resources.line_gap;

        self.background.queue_draw(renderer);
        self.gutter.set_width(gutter_width);
        self.gutter.queue_draw(renderer);
    
        // Selection start index, background = 0, gutter = 1
        let mut s_ix = 2;
        for line_num in first_line..last_line {
            if let Some(ref mut text_widget) = &mut self.get_line(line_num) {
                let line_content = text_widget.get_section().to_borrowed().text[0].text;
                let line_len = line_content.len();

                // Selections
                for selection in self.line_cache.get_selections(line_num).iter() {
                    if selection.start_col == selection.end_col
                    || selection.start_col >= line_len 
                    || selection.end_col >= line_len {
                        continue;
                    }
                    let sel_content = &line_content[selection.start_col..selection.end_col];
                    let sel_x0 = text_ctx.borrow().get_text_width(&line_content[..selection.start_col]);
                    let width = text_ctx.borrow().get_text_width(sel_content);

                    let mut selection = PrimitiveWidget::new(s_ix, [x0 + sel_x0, y, 0.2], [width, line_gap], self.resources.sel);

                    selection.queue_draw(renderer);
                    s_ix += 1;
                }

                // Line body
                text_widget.set_position(x0, y);
                text_widget.queue_draw(renderer);

                // Cursors
                let cursors = text_widget.get_cursor();
                for offset in cursors {
                    let section = &text_widget.get_section().to_borrowed();
                    let pos = text_ctx.borrow_mut()
                        .get_cursor_position(section, offset, scale); 

                    let mut offside = create_offside_section("\u{2588}", self.resources.cursor, scale);
                    offside.screen_position = pos;
                    text_ctx.borrow_mut()
                        .queue_text(&offside.to_borrowed());
                }

                // Line numbers
                if self.show_line_numbers {
                    let content = (line_num + 1).to_string();
                    let left_offset = text_ctx.borrow().get_text_width(content.as_str());

                    let mut offside = create_offside_section(
                        content.clone().as_str(), self.resources.gutter_fg, scale);
                    offside.screen_position = (gutter_width - left_offset - LEFT_PAD, y);
                    text_ctx.borrow_mut()
                        .queue_text(&offside.to_borrowed());
                }
                
                text_widget.set_dirty(true);
            }
            y += line_gap;
        }
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}

#[inline]
fn create_offside_section(content: &str, colour: [f32; 4], scale: f32) -> OwnedSection {
    Section::default()
        .add_text(Text::new(content)
                  .with_scale(scale)
                  .with_color(colour))
        .with_layout(Layout::default_single_line())
        .with_bounds((f32::INFINITY, scale))
        .to_owned()
}

const BLANK: ColourRGBA = [0.0, 0.0, 0.0, 0.0];
impl Resources {
    fn new(scale: f32, line_gap: f32) -> Self {
        Self {
            fg: BLANK,
            bg: BLANK,
            sel: BLANK,
            cursor: BLANK,
            gutter_bg: BLANK,
            gutter_fg: BLANK,
            line_gap,
            scale,
        }
    }
}

impl EditView {
    pub fn new(index: usize, scale: f32) -> Self {
        let size = [0.0, 0.0]; 
        let resources = Resources::new(scale, scale + 3.0);
        let background = PrimitiveWidget::new(0, [0.0, 0.0, 0.01], size, resources.bg);
        let gutter = PrimitiveWidget::new(1, [0.0, 0.0, 0.1], [scale, size[1]], resources.gutter_bg);

        Self {
            index,
            size,
            dirty: true,
            view_id: None,
            config: None,
            theme: None,
            filename: None,
            line_cache: LineCache::new(),
            scroll_offset: 0.0,
            viewport: 0..0,
            core: Default::default(),
            pending: Default::default(),
            show_line_numbers: false,
            mode: Mode::Normal,
            resources,
            background,
            gutter,
        }
    }

    fn get_line(&self, line_num: usize) -> Option<TextWidget> {
        self.line_cache
            .get_line(line_num)
            .map(|line| {
                let resources = &self.resources;
                TextWidget::from_line(line_num, &line, resources.scale, resources.fg)
            })
    }

    fn apply_update(&mut self, update: &Value) {
        self.line_cache.apply_update(update);
        self.constrain_scroll();
        self.dirty = true;
    }

    pub fn resize(&mut self, size: [f32; 2]) {
        self.size = size;
        self.gutter.set_height(size[1]);
        self.background.set_size(size);
        self.dirty = true;

        let (w, h) = (size[0], size[1]);
        self.send_edit_cmd("resize", &json!({ "width": w, "height": h }));
    }

    pub fn go_to_line(&mut self, line: usize) {
        self.send_edit_cmd("insert", &json!({ "line": line }));
    }

    pub fn char(&mut self, ch: char) {
        if ch as u32 >= 0x20 {
            let params = json!({"chars": ch.to_string()});
            self.send_edit_cmd("insert", &params);
        }
    }

    fn send_notification(&mut self, method: &str, params: &Value) {
        let core = self.core.upgrade();
        if core.is_some() && self.view_id.is_some() {
            let core = core.unwrap();
            core.lock().unwrap().send_notification(method, params);
            println!("fe->core: {}", json!({
                method: params,
            }));
        } else {
            println!("queueing pending method: {}", method);
            self.pending.push((method.to_owned(), params.clone()));
        }
    }

    fn send_edit_cmd(&mut self, method: &str, params: &Value) {
        let core = self.core.upgrade();
        if core.is_some() && self.view_id.is_some() {
            let view_id = &self.view_id.clone().unwrap();
            let edit_params = json!({
                "method": method,
                "params": params,
                "view_id": view_id,
            });

            let core = core.unwrap();
            core.lock().unwrap().send_notification("edit", &edit_params);
        } else {
            println!("queueing pending method: {}", method);
            self.pending.push((method.to_owned(), params.clone()));
        }
    }
    
    fn send_action(&mut self, method: &str) {
        self.send_edit_cmd(method, &json!([]));
    }

    fn set_view(&mut self, view_id: String) {
        self.view_id = Some(view_id.to_string());
        self.viewport = 0..0;
        self.update_viewport();

        let pending = std::mem::replace(&mut self.pending, Vec::new());
        for notification in pending {
            let (method, params) = notification;
            self.send_edit_cmd(&method, &params);
        }
    }

    fn config_changed(&mut self, config: Config) {
        println!("config_changed: {:?}", config);
        if config.font_size.is_some() {
            self.resources.scale = config.font_size.unwrap();
            self.resources.line_gap = config.font_size.unwrap() * 1.03;
            self.dirty = true;
        }

        self.config = Some(config);
    }

    fn show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
        self.dirty = true;
    }

    fn theme_changed(&mut self, theme: Theme) {
        if let Some(col) = theme.foreground {
            self.resources.fg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        }
        if let Some(col) = theme.background {
            self.resources.bg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
            self.background.set_colour(rgba8_to_rgba32([col.r, col.g, col.b, col.a]));
        }
        if let Some(col) = theme.caret {
            self.resources.cursor = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        }
        if let Some(col) = theme.selection {
            self.resources.sel = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        } else {
            self.resources.sel = self.resources.cursor;
        }
        if let Some(col) = theme.gutter {
            self.resources.gutter_bg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        } else {
            self.resources.gutter_bg = self.resources.bg;
        }
        self.gutter.set_colour(self.resources.gutter_bg);

        if let Some(col) = theme.gutter_foreground {
            self.resources.gutter_fg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        } else {
            self.resources.gutter_fg = self.resources.fg;
        }

        self.theme = Some(theme);
        self.dirty = true;
    }

    fn set_theme(&mut self, theme_name: &str) {
        self.send_notification("set_theme", &json!({ "theme_name": theme_name }));
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn poke(&mut self, command: EditViewCommands) -> bool {
        match command {
            EditViewCommands::ViewId(view_id) => self.set_view(view_id),
            EditViewCommands::Core(core) => self.core = core.clone(),
            EditViewCommands::ApplyUpdate(update) => self.apply_update(&update),
            EditViewCommands::ScrollTo(line) => self.scroll_to(line),
            EditViewCommands::Resize(size) => self.resize(size),
            EditViewCommands::ConfigChanged(config) => self.config_changed(config),
            EditViewCommands::ThemeChanged(theme) => self.theme_changed(theme),
            EditViewCommands::Action(action) => match action {
                    Action::ReceiveChar(ch) => self.char(ch),
                    Action::SetMode(mode) => self.set_mode(mode),
                    Action::ShowLineNumbers(_) => self.show_line_numbers(!self.show_line_numbers),
                    Action::SetTheme(theme) => self.set_theme(theme.as_str()),
                    Action::Back => self.send_action("delete_backward"),
                    Action::Delete => self.send_action("delete_forward"),
                    Action::Undo => self.send_action("undo"),
                    Action::Redo => self.send_action("redo"),
                    Action::AddCursorAbove => self.send_action("add_selection_above"),
                    Action::AddCursorBelow => self.send_action("add_selection_below"),
                    Action::ClearSelection => self.send_action("collapse_selections"),
                    Action::SingleSelection => self.send_action("cancel_operation"),
                    Action::SelectAll => self.send_action("select_all"),
                    Action::NewLine => self.send_action("insert_newline"),
                    Action::Copy => self.send_action("yank"),
                    Action::ScrollPageUp => self.send_action("scroll_page_up"),
                    Action::ScrollPageDown => self.send_action("scroll_page_down"),
                    Action::Motion(motion) => match motion {
                        Motion::Up => self.send_action("move_up"),
                        Motion::Down => self.send_action("move_down"),
                        Motion::Left => self.send_action("move_left"),
                        Motion::Right => self.send_action("move_right"),
                        Motion::First => self.send_action("move_to_left_end_of_line"),
                        Motion::Last => self.send_action("move_to_right_end_of_line"),
                        Motion::WordLeft => self.send_action("move_word_left"),
                        Motion::WordRight => self.send_action("move_word_right"),
                        _ => return false,
                    },
                    Action::MotionSelect(motion) => match motion {
                        Motion::Up => self.send_action("move_up_and_modify_selection"),
                        Motion::Down => self.send_action("move_down_and_modify_selection"),
                        Motion::Left => self.send_action("move_left_and_modify_selection"),
                        Motion::Right => self.send_action("move_right_and_modify_selection"),
                        Motion::First => self.send_action("move_to_left_end_of_line_and_modify_selection"),
                        Motion::Last => self.send_action("move_to_right_end_of_line_and_modify_selection"),
                        Motion::WordLeft => self.send_action("move_word_left_and_modify_selection"),
                        Motion::WordRight => self.send_action("move_word_right_and_modify_selection"),
                        _ => return false,
                    },
                    _ => return false,
            },
        }

        true
    }

    pub fn mouse_scroll(&mut self, delta: f32) {
        self.scroll_offset -= delta; 
        self.constrain_scroll();
        self.update_viewport();
    }

    fn constrain_scroll(&mut self) {
        if self.scroll_offset < 0.0 {
            self.scroll_offset = 0.0;
            return;
        }
        
        let max_scroll = TOP_PAD + self.resources.line_gap * (self.line_cache.height().saturating_sub(1)) as f32;
        if self.scroll_offset > max_scroll {
           self.scroll_offset = max_scroll; 
        }
    }

    fn y_to_line(&self, y: f32) -> usize {
        let mut line = (y + self.scroll_offset - TOP_PAD) / self.resources.line_gap;
        if line < 0.0 { line = 0.0; }
        let line = line.floor() as usize;

        std::cmp::min(line, self.line_cache.height())
    }

    fn update_viewport(&mut self) {
        let first_line = self.y_to_line(0.0);
        let last_line = first_line + ((self.size[1] / self.resources.line_gap).floor() as usize) + 1;
        let viewport = first_line..last_line;

        if viewport != self.viewport {
            self.viewport = viewport;
            self.send_edit_cmd("scroll", &json!([first_line, last_line]));
        }
    }

    #[inline]
    fn line_to_content_y(&self, line_num: usize) -> f32 {
        TOP_PAD + (line_num as f32) * self.resources.line_gap
    }

    pub fn scroll_to(&mut self, line_num: usize) {
        let y = self.line_to_content_y(line_num);
        let bottom_slop = 20.0;
        if y < self.scroll_offset {
            self.scroll_offset = y;
            self.dirty = true;
        } else if y > self.scroll_offset + self.size[1] - bottom_slop {
            self.scroll_offset = y - (self.size[1] - bottom_slop);
            self.dirty = true;
        }
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

