use std::ops::Range;
use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

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
    ActionTarget,
    Mode,
    Motion,
};
use crate::editor::linecache::LineCache;
use serde_json::{
    json,
    Value,
    Map,
};

use crate::render::Renderer;
use crate::editor::rpc::{
    Core,
    Config,
    Theme,
    Style,
    EditViewCommands,
    theme::ToRgbaFloat32,
};
use super::{
    widget::{
        Widget,
        hash_widget,
    },
    text::TextWidget,
    primitive::PrimitiveWidget,
    status::{
        StatusWidget,
        Status,
    },
};

pub const CURSOR_TEXT: &str = "\u{2588}";

type Method = String;
type Params = Value;
type ColourRGBA = [f32; 4];

pub struct Resources {
    pub fg: ColourRGBA,
    pub bg: ColourRGBA,
    pub sel: ColourRGBA,
    pub cursor: ColourRGBA,
    pub gutter_fg: ColourRGBA,
    pub gutter_bg: ColourRGBA,
    pub scale: f32,
    styles: HashMap<usize, Style>,
}

impl Hash for Resources {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.bg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.sel.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.gutter_fg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.gutter_bg.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.scale.to_le_bytes().hash(state);
    }
}
impl Resources {
    #[inline]
    pub fn line_gap(&self) -> f32 {
        self.scale * 1.06
    }
    pub fn pad(&self) -> f32 {
        self.scale * 0.25
    }
}

pub struct EditView {
    index: usize,
    size: [f32; 2],
    dirty: bool,
    view_id: Option<String>,
    filepath: Option<String>,
    line_cache: LineCache,
    scroll_offset: f32,
    viewport: Range<usize>,
    core: Weak<Mutex<Core>>,
    pending: Vec<(Method, Params)>,
    config: Option<Config>,
    theme: Option<Theme>,
    language: Option<String>,
    resources: Resources,
    gutter: PrimitiveWidget,
    background: PrimitiveWidget,
    status_bar: StatusWidget,
    current_line: usize,
    show_line_numbers: bool,
}

impl Hash for EditView {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state); 
        self.view_id.hash(state);
        self.scroll_offset.to_le_bytes().hash(state);
        self.viewport.hash(state);
        self.resources.hash(state);
        self.gutter.hash(state);
        self.background.hash(state);
        self.status_bar.hash(state);
        self.current_line.hash(state);
        self.show_line_numbers.hash(state);
    }
}

impl Widget for EditView {
    fn index(&self) -> usize {
        self.index 
    }

    fn position(&self) -> [f32; 2] {
        let pad = self.resources.scale / 4.0;
        [pad, pad]
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        let text_ctx = renderer.get_text_context().clone();

        let line_gap = self.resources.line_gap();
        let pad = self.resources.pad();
        let drawable_height = self.drawable_text_height();
        let first_line = self.y_to_line(0.0);
        let last_line = std::cmp::min(self.y_to_line(drawable_height) + 1, self.line_cache.height());
        
        // Ensure our text context is up to date
        let scale = self.resources.scale;
        if let text_ctx = &mut text_ctx.borrow_mut() {
            if scale != text_ctx.get_font_size() {
                text_ctx.set_font_size(scale);
            }
        }

        // Figure out the maximum width of the line number
        let gutter_width = if self.show_line_numbers {
            pad + pad 
                + text_ctx.borrow().get_text_width(last_line.to_string().clone().as_str())
        } else {
            0.0
        };
        let x0 = pad + gutter_width;
        let mut y = self.line_to_content_y(first_line) - self.scroll_offset;

        self.background.queue_draw(renderer);
        self.gutter.set_width(gutter_width);
        self.gutter.queue_draw(renderer);

        // Status Bar
        let mode_width = pad + pad + text_ctx.borrow()
            .get_text_width(self.mode().to_string().clone().as_str());

        self.status_bar.set_mode_width(mode_width);
        self.status_bar.set_scale(line_gap);
        self.status_bar.queue_draw(renderer);
    
        // Selection start index, background = 0, gutter = 1, status_bar = 2, 3, 4
        let mut s_ix = 6;
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

                    let mut selection = PrimitiveWidget::new(
                        s_ix, [x0 + sel_x0, y, 0.2], [width, scale], self.resources.sel);

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
                        .get_cursor_position(section, offset); 

                    let mut offside = create_offside_section(CURSOR_TEXT, self.resources.cursor, scale);
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
                    offside.screen_position = (gutter_width - left_offset - pad, y);
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
const BLACK: ColourRGBA = [0.0, 0.0, 0.0, 1.0];
impl Resources {
    fn new(scale: f32) -> Self {
        Self {
            fg: BLANK,
            bg: BLANK,
            sel: BLANK,
            cursor: BLANK,
            gutter_bg: BLANK,
            gutter_fg: BLANK,
            scale,
            styles: HashMap::new(),
        }
    }
}

impl EditView {
    pub fn new(index: usize, scale: f32, filename: Option<String>) -> Self {
        let size = [0.0, 0.0]; 
        let resources = Resources::new(scale);
        let background = PrimitiveWidget::new(0, [0.0, 0.0, 0.01], size, resources.bg);
        let gutter = PrimitiveWidget::new(1, [0.0, 0.0, 0.1], [scale, size[1]], resources.gutter_bg);

        let status = Status {
            mode: Mode::Normal,
            filename: filename.clone(),
            line_current: 0,
            line_count: 0,
            language: None,
        };
        let status_bar = StatusWidget::new(2, status, &resources);
        println!("created status bar");

        Self {
            index,
            size,
            dirty: true,
            view_id: None,
            config: None,
            theme: None,
            language: None,
            line_cache: LineCache::new(),
            scroll_offset: 0.0,
            viewport: 0..0,
            current_line: 0,
            show_line_numbers: false,
            core: Default::default(),
            pending: Default::default(),
            filepath: filename,
            status_bar,
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
                TextWidget::from_line(line_num, &line, resources.scale, resources.fg, &resources.styles)
            })
    }

    fn apply_update(&mut self, update: &Value) {
        self.line_cache.apply_update(update);
        self.constrain_scroll();
        self.dirty = true;
    }

    fn resize(&mut self, size: [f32; 2]) {
        self.status_bar.set_size([size[0], self.resources.line_gap()]);

        let height = size[1] - self.status_bar.size()[1];
        self.size = [size[0], height];
        self.gutter.set_height(height);
        self.background.set_size([size[0], height]);
        self.status_bar.set_position(0.0, height); 
        self.dirty = true;

        let (w, h) = (size[0], self.drawable_text_height());
        self.send_edit_cmd("resize", &json!({ "width": w, "height": h }));
    }

    fn go_to_line(&mut self, line: usize) {
        self.send_edit_cmd("insert", &json!({ "line": line }));
    }

    fn send_char(&mut self, ch: char) {
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

    fn save_to_file(&mut self) {
        if self.view_id.is_some() {
            self.send_notification("save", &json!({
                "view_id": self.view_id,
                "file_path": self.filepath,
            }));
        } else {
            self.status_bar.update_command_section("Unable to save, no filepath set");
        }
    }

    fn define_style(&mut self, style: Style) {
        self.resources.styles.insert(style.id, style);
    }

    fn config_changed(&mut self, config: Config) {
        if config.font_size.is_some() {
            let old_height = self.size[1] + self.resources.line_gap();
            self.resources.scale = config.font_size.unwrap();
            self.resize([self.size[0], old_height]);
            self.dirty = true;
        }

        self.config = Some(config);
    }

    fn modify_config(&mut self, config: Config) {
        let changes = if let Some(self_config) = self.config.clone() {
            self_config.get_json_changes(config)
        } else {
            serde_json::to_value(config).unwrap()
        };
        self.send_notification("modify_user_config", &json!({
            "domain": { "user_override": self.view_id },
            "changes": Value::from(changes),
        }));
    }

    fn show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
        self.dirty = true;
    }

    fn set_font_size(&mut self, font_size: f32) {
        let config = Config {
            font_size: Some(font_size),
            ..Config::default()
        };
        self.modify_config(config);
    }
    fn increase_font_size(&mut self) {
        if let Some(config) = &mut self.config {
            let size = config.font_size.unwrap_or(self.resources.scale);
            self.set_font_size(size + 1.0);
        }
    }
    fn decrease_font_size(&mut self) {
        if let Some(config) = &mut self.config {
            let size = config.font_size.unwrap_or(self.resources.scale);
            if size >= 2.0 {
                self.set_font_size(size - 1.0);
            }
        }
    }

    fn language_changed(&mut self, language_id: String) {
        self.language = Some(language_id);
        self.status_bar.update_line_status(self.current_line, self.line_cache.height(), self.language.clone());
    }

    fn theme_changed(&mut self, theme: Theme) {
        if let Some(col) = &theme.foreground {
            self.resources.fg = col.to_rgba_f32array();
        }
        if let Some(col) = &theme.background {
            self.resources.bg = col.to_rgba_f32array(); 
            self.background.set_colour(self.resources.bg.clone());
        }
        if let Some(col) = &theme.caret {
            self.resources.cursor = col.to_rgba_f32array();
        }
        if let Some(col) = &theme.selection {
            self.resources.sel = col.to_rgba_f32array();
        } else {
            self.resources.sel = self.resources.cursor;
        }
        if let Some(col) = &theme.gutter {
            self.resources.gutter_bg = col.to_rgba_f32array();
        } else {
            self.resources.gutter_bg = self.resources.bg;
        }
        self.gutter.set_colour(self.resources.gutter_bg);
        self.status_bar.set_colours(
            self.resources.gutter_bg.clone(), 
            self.resources.fg.clone(),
            BLACK);
        self.status_bar.set_scale(self.resources.scale);

        if let Some(col) = &theme.gutter_foreground {
            self.resources.gutter_fg = col.to_rgba_f32array();
        } else {
            self.resources.gutter_fg = self.resources.fg;
        }

        self.dirty = true;
        self.theme = Some(theme);
    }

    fn set_theme(&mut self, theme_name: &str) {
        self.send_notification("set_theme", &json!({ "theme_name": theme_name }));
    }

    fn set_mode(&mut self, mode: Mode) {
        self.status_bar.set_mode(mode);
        self.dirty = true;
    }
    pub fn mode(&self) -> Mode {
        self.status_bar.mode()
    }

    pub fn poke_target(&mut self, command: EditViewCommands, target: ActionTarget) -> bool {
        match target {
            ActionTarget::FocusedView => self.poke(command),
            ActionTarget::StatusBar => match command {
                EditViewCommands::Action(action) => self.status_bar.poke(Box::new(action)),
                _ => return false,
            },
            _ => return false, 
        }
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
            EditViewCommands::LanguageChanged(language_id) => self.language_changed(language_id),
            EditViewCommands::DefineStyle(style) => self.define_style(style),
            EditViewCommands::Action(action) => match action {
                    Action::InsertChar(ch) => self.send_char(ch),
                    Action::SetMode(mode) => self.set_mode(mode),
                    Action::SetTheme(theme) => self.set_theme(theme.as_str()),
                    Action::ShowLineNumbers(_) => self.show_line_numbers(!self.show_line_numbers),
                    Action::Save => self.save_to_file(),
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
                    Action::IncreaseFontSize => self.increase_font_size(),
                    Action::DecreaseFontSize => self.decrease_font_size(),
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

    fn drawable_text_height(&self) -> f32 {
        let sb_size = self.status_bar.size();
        if sb_size[1] > self.size[1] {
            self.size[1]
        } else {
            self.size[1] - sb_size[1] 
        }
    }

    fn constrain_scroll(&mut self) {
        if self.scroll_offset < 0.0 {
            self.scroll_offset = 0.0;
            return;
        }
       
        let max_scroll = self.drawable_text_height();
        if self.scroll_offset > max_scroll {
           self.scroll_offset = max_scroll; 
        }
    }

    fn y_to_line(&self, y: f32) -> usize {
        let pad = self.resources.pad();
        let mut line = (y + self.scroll_offset - pad) / self.resources.line_gap();
        if line < 0.0 { line = 0.0; }
        let line = line.floor() as usize;

        std::cmp::min(line, self.line_cache.height())
    }

    fn update_viewport(&mut self) {
        let first_line = self.y_to_line(0.0);
        let last_line = first_line + ((self.drawable_text_height() / self.resources.line_gap()).floor() as usize) + 1;
        let viewport = first_line..last_line;

        if viewport != self.viewport {
            self.viewport = viewport;
            self.status_bar.update_line_status(self.current_line, self.line_cache.height(), self.language.clone());
            self.send_edit_cmd("scroll", &json!([first_line, last_line]));
        }
    }

    #[inline]
    fn line_to_content_y(&self, line_num: usize) -> f32 {
        self.resources.pad() + (line_num as f32) * self.resources.line_gap()
    }

    pub fn scroll_to(&mut self, line_num: usize) {
        let y = self.line_to_content_y(line_num);
        let bottom_slop = self.resources.scale / 3.0;
        if y < self.scroll_offset {
            self.scroll_offset = y;
            self.dirty = true;
        } else if y > self.scroll_offset + self.drawable_text_height() - bottom_slop {
            self.scroll_offset = y - (self.drawable_text_height() - bottom_slop);
            self.dirty = true;
        }
        self.current_line = line_num + 1;
        self.status_bar.update_line_status(self.current_line, self.line_cache.height(), self.language.clone());
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

