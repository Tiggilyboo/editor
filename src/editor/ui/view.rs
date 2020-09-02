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
use log::error;

use serde_json::{
    json,
    Value,
};
use glyph_brush::{
    OwnedSection,
    Section,
    Layout,
    Text,
};

use rpc::{ 
    Action,
    ActionTarget,
    GestureType,
    SelectionGranularity,
    PluginAction,
    PluginId,
    Quantity,
    Query,
    Mode,
    Motion,
    Config,
    Theme,
    Style,
};
use crate::render::Renderer;
use crate::editor::{
    plugins::{
        PluginState,
    },
    linecache::LineCache,
    editor_rpc::Core,
    commands::EditViewCommands,
    view_resources::Resources,
};
use crate::events::{
    EditorEventLoopProxy,
    EditorEvent,
};
use super::{
    colour::{
        BLACK,
    },
    widget::{
        Widget,
        hash_widget,
    },
    text::TextWidget,
    editable_text::CURSOR_TEXT,
    primitive::PrimitiveWidget,
    status::{
        StatusWidget,
        Status,
    },
    find_replace::FindWidget,
};


type Method = String;
type Params = Value;

pub struct EditView {
    index: usize,
    size: [f32; 2],
    position: [f32; 2],
    dirty: bool,
    focused: bool,
    view_id: Option<String>,
    filepath: Option<String>,
    line_cache: LineCache,
    scroll_offset: f32,
    viewport: Range<usize>,
    core: Weak<Mutex<Core>>,
    event_proxy: Option<EditorEventLoopProxy>,
    pending: Vec<(Method, Params)>,
    config: Option<Config>,
    theme: Option<Theme>,
    language: Option<String>,
    resources: Resources,
    gutter: PrimitiveWidget,
    background: PrimitiveWidget,
    status_bar: StatusWidget,
    find_replace: FindWidget,
    plugins: HashMap<PluginId, PluginState>,
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
        self.find_replace.hash(state);
        self.current_line.hash(state);
        self.show_line_numbers.hash(state);
    }
}

impl Widget for EditView {
    fn index(&self) -> usize {
        self.index 
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        let text_ctx = renderer.get_text_context().clone();

        let line_gap = self.resources.line_gap();
        let pad = self.resources.pad();
        let drawable_height = self.drawable_text_height();
        let first_line = self.y_to_line(self.position[1]);
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
        let x0 = self.position[0] + pad + gutter_width;
        let mut y = self.position[1] + self.line_to_content_y(first_line) - self.scroll_offset;

        // Background & Gutter
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
                if self.focused {
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
                }

                // Line body
                text_widget.set_position(x0, y);
                text_widget.queue_draw(renderer);

                // Cursors
                if self.focused {
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
        self.dirty || self.status_bar.dirty()
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
        let find_replace = FindWidget::new(3, &resources);

        let pad = resources.scale / 4.0;
        let position = [pad, pad];

        Self {
            index,
            size,
            position,
            focused: true,
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
            event_proxy: None,
            plugins: HashMap::new(),
            filepath: filename,
            status_bar,
            find_replace,
            resources,
            background,
            gutter,
        }
    }

    fn get_line(&self, line_num: usize) -> Option<TextWidget> {
        let resources = &self.resources;
        self.line_cache
            .get_line(line_num)
            .map(|line| {
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
        self.status_bar.set_position(self.position[0], height); 
        self.dirty = true;

        let (w, h) = (size[0], self.drawable_text_height());
        self.send_edit_cmd("resize", &json!({ "width": w, "height": h }));
        self.update_viewport();
    }
    fn set_position(&mut self, x: f32, y: f32) {
        self.position = [x, y];
        self.status_bar.set_position(x, y + self.size[1] - self.status_bar.size()[1]);
        self.background.set_position(x, y);
        self.gutter.set_position(x, y);
        self.dirty = true;
    }
    pub fn set_focused(&mut self, focused: bool) {
        if self.focused != focused {
            self.dirty = true;
            self.status_bar.set_dirty(true);
        }
        self.status_bar.set_focused(focused);
        self.focused = focused;
    }

    fn go_to_line(&mut self, line: usize) {
        self.send_edit_cmd("goto_line", &json!({ "line": line }));
    }

    fn gesture(&mut self, line: usize, col: usize, gesture_type: GestureType) {
        self.send_edit_cmd("gesture", &json!({
            "line": line,
            "col": col,
            "ty": gesture_type,
        }));
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
            if !core.lock().unwrap().send_notification(method, params) {
                self.pending.push((method.to_owned(), params.clone()));  
            } else {
                println!("fe->core: {}", json!({
                    method: params,
                }));
            }
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
            if !core.lock().unwrap().send_notification("edit", &edit_params) {
                self.pending.push(("edit".to_string(), edit_params));
            }

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

    fn save_to_file(&mut self, filename: Option<String>) {
        let filename = if filename.is_some() { 
            filename
        } else {
            self.filepath.clone()
        };
        if self.view_id.is_some() && filename.is_some() {
            self.send_notification("save", &json!({
                "view_id": self.view_id,
                "file_path": filename,
            }));
        } else {
            error!("Unable to save to file: {:?}", filename); 
        }
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
        self.resources.update_from_theme(theme.clone());
        self.gutter.set_colour(self.resources.gutter_bg);
        self.background.set_colour(self.resources.bg.clone());
        self.status_bar.set_colours(
            self.resources.gutter_bg.clone(), 
            self.resources.fg.clone(),
            self.resources.cursor.clone(),
            BLACK);
        self.status_bar.set_scale(self.resources.scale);

        self.dirty = true;
        self.theme = Some(theme);
    }

    fn set_theme(&mut self, theme_name: &str) {
        self.send_notification("set_theme", &json!({ "theme_name": theme_name }));
        self.update_viewport(); 
    }

    fn set_language(&mut self, language: &str) {
        self.send_notification("set_language", &json!({ "language": language }));
    }

    fn set_styles(&mut self, styles: HashMap<usize, Style>) {
        self.resources.styles = styles;
    }
    fn set_plugins(&mut self, plugins: HashMap<PluginId, PluginState>) {
        self.plugins = plugins;
    }

    fn plugin_changed(&mut self, plugin: PluginState) {
        if !self.plugins.contains_key(&plugin.name) {
            self.plugins.insert(plugin.name.clone(), plugin);
        }
    }
    fn plugin_stopped(&mut self, plugin_id: PluginId) {
        if let Some(ref mut stopped_plugin) = &mut self.plugins.get_mut(&plugin_id) {
            stopped_plugin.active = false;
        }
    }
    fn queries_changed(&mut self, queries: Vec<Query>) {
        self.find_replace.set_queries(queries);
    }

    fn close_view(&mut self) {
        if let Some(view_id) = self.view_id.clone() {
            self.send_notification("close_view", &json!({ "view_id": view_id }));
            if let Some(proxy) = &self.event_proxy {
                match proxy.send_event(EditorEvent::Action(Action::Close)) {
                    Ok(_) => (),
                    Err(err) => panic!(err),
                }
            }
        }
    }
    fn open_file(&mut self, filename: Option<String>) {
        let filename = if filename.is_some() {
            filename
        } else {
            self.filepath.clone()
        };
        if let Some(proxy) = &self.event_proxy {
            match proxy.send_event(EditorEvent::Action(Action::Open(filename.clone()))) {
                Ok(_) => {},
                Err(err) => panic!(err),
            }
        }
    }
    fn split_view(&self, filename: Option<String>) {
        if let Some(proxy) = &self.event_proxy {
            match proxy.send_event(EditorEvent::Action(Action::Split(filename))) {
                Ok(_) => (),
                Err(err) => panic!(err),
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.status_bar.set_mode(mode);
        self.dirty = true;
    }
    pub fn mode(&self) -> Mode {
        self.status_bar.mode()
    }
    fn mode_selection_granularity(&self) -> SelectionGranularity {
        match self.mode() {
            Mode::SelectBlock => SelectionGranularity::Point,
            Mode::Select | Mode::SelectLine => SelectionGranularity::Line,
            _ => panic!("unhandled selection granularity for mode"),
        }
    }

    pub fn poke_target(&mut self, command: EditViewCommands, target: ActionTarget) -> bool {
        match target {
            ActionTarget::FocusedView => self.poke(command),
            ActionTarget::StatusBar => match command {
                EditViewCommands::Action(action) => self.status_bar.poke(Box::new(action)),
                _ => return false,
            },
            ActionTarget::EventLoop => return false,
        }
    }

    fn execute_motion(&mut self) -> Vec<Action> {
        let motion_text = self.status_bar.get_command();
        self.status_bar.set_command_text("");
        self.set_mode(Mode::Normal);

        let mut actions: Vec<Action> = vec!();
        if let Ok(number) = motion_text.parse::<usize>() {
            actions.push(Action::Motion((Motion::Middle, Some(Quantity::Line(number)))));
        }

        actions
    }

    pub fn execute_command(&mut self) -> Vec<Action> {
        let command_text = self.status_bar.get_command();
        self.status_bar.set_command_text("");
        self.set_mode(Mode::Normal);

        let mut actions: Vec<Action> = vec!(); 
        let args: Vec<String> = command_text.split(" ").map(|a| a.to_string()).collect();

        let filename = if args.len() > 1 {
            Some(args[1].clone())
        } else {
            self.filepath.clone()
        };

        // TODO: Abstract and make this not crap
        match args[0].as_str() {
            "e" => actions.push(Action::Open(filename)),
            "w" => actions.push(Action::Save(filename)),
            "q" => actions.push(Action::Close),
            "wq" => actions.extend(vec![Action::Save(filename), Action::Close]),
            "sp" => actions.push(Action::Split(filename)),
            "plug" => {
                if args.len() < 3 {
                    println!("usage: plug [start|stop] <plugin_name>");
                } else {
                    let plugin_id = PluginId::from(args[2].clone());
                    match args[1].as_str() {
                        "start" => actions.push(Action::Plugin(PluginAction::Start(plugin_id))),
                        "stop" => actions.push(Action::Plugin(PluginAction::Stop(plugin_id))),
                        _ => println!("args: {:?}", args),
                    }
                }
            },
            _ => {},
        }

        if actions.len() == 0 {
            println!("No command found: '{}'", command_text.clone());
        }

        actions
    }

    fn handle_plugin_action(&mut self, plugin_action: PluginAction) {
        let view_id = self.view_id.clone().unwrap();

        match plugin_action {
            PluginAction::Start(plugin_name) => {
                self.send_notification("plugin", &json!({
                    "method": "start",
                    "params": json!({
                        "view_id": view_id,
                        "plugin_name": plugin_name,
                    }),
                }));
            },
            PluginAction::Stop(plugin_name) => {
                self.send_notification("plugin", &json!({
                    "method": "start",
                    "params": json!({
                        "view_id": view_id,
                        "plugin_name": plugin_name,
                    }),
                }));
            },
        }
    }
    fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::Open(filename) => self.open_file(filename),
            Action::Split(filename) => self.split_view(filename),
            Action::Save(filename) => self.save_to_file(filename),
            Action::InsertChar(ch) => self.send_char(ch),
            Action::SetMode(mode) => self.set_mode(mode),
            Action::SetTheme(theme) => self.set_theme(theme.as_str()),
            Action::SetLanguage(language) => self.set_language(language.as_str()),
            Action::Plugin(plugin_action) => self.handle_plugin_action(plugin_action),
            Action::Close => self.close_view(),
            Action::ToggleLineNumbers => self.show_line_numbers(!self.show_line_numbers),
            Action::Undo => self.send_action("undo"),
            Action::Redo => self.send_action("redo"),
            Action::ClearSelection => self.send_action("collapse_selections"),
            Action::SingleSelection => self.send_action("cancel_operation"),
            Action::NewLine => self.send_action("insert_newline"),
            Action::Cut => self.send_action("yank"),
            Action::Copy => self.send_action("copy"),
            Action::Paste => self.send_action("paste"),
            Action::Indent => self.send_action("indent"),
            Action::Outdent => self.send_action("outdent"),
            Action::InsertTab => self.send_action("insert_tab"),
            Action::DuplicateLine => self.send_action("duplicate_line"),
            Action::IncreaseFontSize => self.increase_font_size(),
            Action::DecreaseFontSize => self.decrease_font_size(),
            Action::Execute => match self.mode() {
                Mode::Command => {
                    self.execute_command().iter()
                        .filter(|a| match a { Action::Execute => false, _ => true })
                        .for_each(|a| { 
                            self.poke(EditViewCommands::Action(a.clone())); 
                        });
                },
                Mode::Motion => {
                    self.execute_motion().iter()
                        .filter(|a| match a { Action::Execute => false, _ => true })
                        .for_each(|a| {
                            self.poke(EditViewCommands::Action(a.clone()));
                        });
                },
                _ => return false,
            },
            Action::Motion((motion, quantity)) => match quantity.unwrap_or_default() {
                Quantity::Number(n) => for _ in 0..n { match motion {
                    Motion::Up => self.send_action("move_up"),
                    Motion::Down => self.send_action("move_down"),
                    Motion::Left => self.send_action("move_left"),
                    Motion::Right => self.send_action("move_right"),
                    Motion::First => self.send_action("move_to_left_end_of_line"),
                    Motion::FirstOccupied => self.send_action("move_to_left_end_of_line"), // TODO: inaccurate
                    Motion::Last => self.send_action("move_to_right_end_of_line"),
                    Motion::High => self.go_to_line(self.viewport.start),
                    Motion::Low => self.go_to_line(self.viewport.end),
                    Motion::Middle => {
                        let mut middle = self.viewport.start 
                            + (self.viewport.end - self.viewport.start) / 2;
                        if middle > self.line_cache.height() {
                            middle = self.line_cache.height();
                        }
                        self.go_to_line(middle);
                    },
                    _ => return false,
                } },
                Quantity::Page(n) => for _ in 0..n { match motion {
                    Motion::Up => self.send_action("scroll_page_up"),
                    Motion::Down => self.send_action("scroll_page_down"),
                    _ => return false,
                } },
                Quantity::Word(n) => for _ in 0..n { match motion {
                    Motion::Left => self.send_action("move_word_left"),
                    Motion::Right => self.send_action("move_word_right"),
                    _ => return false,
                } },
                Quantity::Line(n) => match motion {
                    Motion::First => self.go_to_line(0),
                    Motion::Last => self.go_to_line(self.line_cache.height()),
                    Motion::High => self.go_to_line(self.viewport.start),
                    Motion::Low => self.go_to_line(self.viewport.end),
                    Motion::Middle => {
                        let mut middle = n + self.viewport.start 
                            + (self.viewport.end - self.viewport.start) / 2;
                        if middle > self.line_cache.height() {
                            middle = self.line_cache.height();
                        }
                        self.go_to_line(middle);
                    },
                    _ => return false,
                },
                _ => return false,
            },
            Action::Select((motion, quantity)) => match quantity.unwrap_or_default() {
                Quantity::All => self.send_action("select_all"),
                Quantity::Line(n) => {
                    let last = if self.line_cache.height() > self.current_line + n {
                        self.current_line + n
                    } else {
                        self.line_cache.height()
                    };
                    self.send_action("move_to_left_end_of_line");
                    self.send_action("move_to_right_end_of_line_and_modify_selection");
                    for _ in self.current_line..last {
                        self.send_action("move_down_and_modify_selection");
                    }
                },
                Quantity::Number(n) => for _ in 0..n { match motion {
                    Motion::Up => self.send_action("move_up_and_modify_selection"),
                    Motion::Down => self.send_action("move_down_and_modify_selection"),
                    Motion::Left => self.send_action("move_left_and_modify_selection"),
                    Motion::Right => self.send_action("move_right_and_modify_selection"),
                    Motion::First => self.send_action("move_to_left_end_of_line_and_modify_selection"),
                    Motion::FirstOccupied => self.send_action("move_to_left_end_of_line_and_modify_selection"),
                    Motion::Last => self.send_action("move_to_right_end_of_line_and_modify_selection"),
                    Motion::High => self.gesture(self.viewport.start, 0, GestureType::SelectExtend {
                        granularity: self.mode_selection_granularity(),
                    }),
                    Motion::Low => self.gesture(self.viewport.end, 0, GestureType::SelectExtend {
                        granularity: self.mode_selection_granularity(),
                    }),
                    Motion::Middle => self.gesture(
                        self.viewport.start + (self.viewport.end - self.viewport.start) / 2, 0, 
                        GestureType::SelectExtend {
                            granularity: self.mode_selection_granularity(),
                        }),
                    _ => return false,
                } },
                Quantity::Word(n) => for _ in 0 ..n { match motion {
                    Motion::Left => self.send_action("move_word_left_and_modify_selection"),
                    Motion::Right => self.send_action("move_word_right_and_modify_selection"),
                    _ => return false,
                } },
                _ => return false,
            },
            Action::Delete((motion, quantity)) => match motion {
                Motion::Left => match quantity.unwrap_or_default() {
                    Quantity::Word(n) => for _ in 0..n {
                        self.send_action("delete_word_backward");
                    },
                    _ => self.send_action("delete_backward"),
                },
                Motion::Right => match quantity.unwrap_or_default() {
                    Quantity::Word(n) => for _ in 0..n {
                        self.send_action("delete_word_forward");
                    },
                    _ => self.send_action("delete_forward"),
                },
                Motion::Up => {
                    self.handle_action(Action::Motion((Motion::Up, None)));
                    self.handle_action(Action::Motion((Motion::First, None)));
                    self.handle_action(Action::Select((Motion::Last, None)));
                    self.handle_action(Action::Delete((Motion::Left, Some(Quantity::Number(2)))));
                },
                Motion::Down => {
                    self.handle_action(Action::Motion((Motion::Down, None)));
                    self.handle_action(Action::Motion((Motion::First, None)));
                    self.handle_action(Action::Select((Motion::Last, None)));
                    self.handle_action(Action::Delete((Motion::Left, Some(Quantity::Number(2)))));
                },
                Motion::First => self.send_action("delete_to_beginning_of_line"),
                Motion::Last => {
                    self.send_action("move_to_right_end_of_line_and_modify_selection");
                    self.send_action("delete_backward");
                },
                Motion::High => {
                    self.gesture(self.viewport.start, 0, GestureType::Select {
                        granularity: self.mode_selection_granularity(),
                        multi: false,
                    });
                    self.send_action("delete_backward");
                },
                Motion::Low => {
                    self.gesture(self.viewport.end, 0, GestureType::Select {
                        granularity: self.mode_selection_granularity(),
                        multi: false,
                    });
                    self.send_action("delete_backward");
                },
                Motion::Middle => {
                    self.gesture(
                        self.viewport.start + (self.viewport.end - self.viewport.start) / 2, 0,
                        GestureType::Select {
                            granularity: self.mode_selection_granularity(),
                            multi: false,
                    });
                    self.send_action("delete_backward");
                },
                _ => (),
            },
            Action::AddCursor(motion) => match motion {
                Motion::Up => self.send_action("add_selection_above"),
                Motion::Down => self.send_action("add_selection_below"),
                _ => (),
            },
            _ => return false,
        }

        true
    }

    pub fn poke(&mut self, command: EditViewCommands) -> bool {
        match command {
            EditViewCommands::ViewId(view_id) => self.set_view(view_id),
            EditViewCommands::Core(core) => self.core = core.clone(),
            EditViewCommands::Proxy(event_proxy) => self.event_proxy = Some(event_proxy),
            EditViewCommands::ApplyUpdate(update) => self.apply_update(&update),
            EditViewCommands::ScrollTo(line) => self.scroll_to(line),
            EditViewCommands::Resize(size) => self.resize(size),
            EditViewCommands::Position(position) => self.set_position(position[0], position[1]),
            EditViewCommands::ConfigChanged(config) => self.config_changed(config),
            EditViewCommands::ThemeChanged(theme) => self.theme_changed(theme),
            EditViewCommands::LanguageChanged(language_id) => self.language_changed(language_id),
            EditViewCommands::SetStyles(styles) => self.set_styles(styles),
            EditViewCommands::SetPlugins(plugins) => self.set_plugins(plugins),
            EditViewCommands::PluginChanged(plugin) => self.plugin_changed(plugin),
            EditViewCommands::PluginStopped(plugin_id) => self.plugin_stopped(plugin_id),
            EditViewCommands::Queries(queries) => self.queries_changed(queries),
            EditViewCommands::Action(action) => return self.handle_action(action),
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
        let mut line = (y + self.scroll_offset - pad - self.position[1]) / self.resources.line_gap();
        if line < 0.0 { line = 0.0; }
        let line = line.floor() as usize;

        std::cmp::min(line, self.line_cache.height())
    }

    fn update_viewport(&mut self) {
        let first_line = self.y_to_line(self.position[1]);
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
        self.position[0] + self.resources.pad() + (line_num as f32) * self.resources.line_gap()
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
        self.status_bar.set_dirty(dirty);
        self.dirty = dirty;
    }
}

