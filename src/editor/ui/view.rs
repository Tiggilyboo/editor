use std::ops::Range;
use std::sync::{
    Mutex,
    Weak,
};

use winit::event::{
    VirtualKeyCode,
    ModifiersState,
};
use glyph_brush::{
    OwnedSection,
    Section,
    Layout,
    Text,
};

use crate::editor::rpc::Core;
use crate::editor::linecache::LineCache;
use serde_json::{
    json,
    Value,
};

use crate::render::Renderer;
use crate::editor::rpc::{
    Config,
    Theme,
};
use super::{
    text::TextWidget,
    widget::Widget,
};

type Method = String;
type Params = Value;
type ColourRGBA = [f32; 4];

pub enum EditViewCommands {
    ViewId(String),
    ApplyUpdate(Value),
    ScrollTo(usize),
    Core(Weak<Mutex<Core>>),
    Resize([f32; 2]),
    ConfigChanged(Config),
    ThemeChanged(Theme),
    SetTheme(String),
    MeasureWidth((u64, Vec<Value>)),
    Undo,
    Redo,
    UpperCase,
    LowerCase,
    AddCursorAbove,
    AddCursorBelow,
    SingleSelection,
    SelectAll,
}

struct Resources {
    fg: ColourRGBA,
    bg: ColourRGBA,
    sel: ColourRGBA,
    scale: f32,
}

pub struct EditView {
    index: usize,
    dirty: bool,
    view_id: Option<String>,
    line_cache: LineCache,
    scroll_offset: f32,
    viewport: Range<usize>,
    core: Weak<Mutex<Core>>,
    pending: Vec<(Method, Params)>,
    config: Option<Config>,
    theme: Option<Theme>,
    size: [f32; 2],
    resources: Resources,
}

const TOP_PAD: f32 = 6.0;
const LEFT_PAD: f32 = 6.0;
const LINE_SPACE: f32 = 17.0;

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

    fn queue_draw(&self, renderer: &mut Renderer) {
        let first_line = self.y_to_line(0.0);
        let last_line = std::cmp::min(self.y_to_line(self.size[1]) + 1, self.line_cache.height());
        
        let x0 = LEFT_PAD;
        let mut y = self.line_to_content_y(first_line) - self.scroll_offset;

        let text_ctx = renderer.get_text_context().clone();

        for line_num in first_line..last_line {
            if let Some(ref mut text_widget) = &mut self.get_line(line_num) {
                text_widget.set_position(x0, y);
                text_widget.queue_draw(renderer);

                let cursors = text_widget.get_cursor();
                for offset in cursors {
                    let section = &text_widget.get_section().to_borrowed();
                    let pos = text_ctx.borrow_mut()
                        .get_cursor_position(section, offset, self.resources.scale); 

                    let mut offside = create_offside_section(self.resources.sel, self.resources.scale);
                    offside.screen_position = pos;
                    renderer.get_text_context().borrow_mut()
                        .queue_text(&offside.to_borrowed());
                }
                
                text_widget.set_dirty(true);
            }
            y += LINE_SPACE;
        }
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}

fn create_offside_section(colour: [f32; 4], scale: f32) -> OwnedSection {
    Section::default()
        .add_text(Text::new("\u{2588}")
                  .with_scale(scale)
                  .with_color(colour))
        .with_layout(Layout::default_single_line())
        .with_bounds((f32::INFINITY, scale))
        .to_owned()
}

impl EditView {
    pub fn new(index: usize, size: [f32; 2], scale: f32) -> Self {
        let resources = Resources {
            fg: [0.9, 0.9, 0.9, 1.0],
            bg: [0.1, 0.1, 0.1, 1.0],
            sel: [0.3, 0.3, 0.3, 0.7],
            scale,
        };
        Self {
            index,
            size,
            dirty: false,
            view_id: None,
            config: None,
            theme: None,
            line_cache: LineCache::new(),
            scroll_offset: 0.0,
            viewport: 0..0,
            core: Default::default(),
            pending: Default::default(),
            resources,
        }
    }

    pub fn clear_line_cache(&mut self) {
        self.line_cache = LineCache::new();
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
        self.dirty = true;

        let (w, h) = (size[0], size[1]);
        self.send_edit_cmd("resize", &json!({ "width": w, "height": h }));
    }

    pub fn go_to_line(&mut self, line: usize) {
        self.send_edit_cmd("insert", &json!({ "line": line }));
    }

    pub fn char(&mut self, ch: char) -> bool {
        if ch as u32 >= 0x20 {
            let params = json!({"chars": ch.to_string()});
            self.send_edit_cmd("insert", &params);

            true
        } else {
            false
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
            println!("fe->core: {}", json!({
                "method": method,
                "params": params,
                "view_id": view_id,
            }));
        } else {
            println!("queueing pending method: {}", method);
            self.pending.push((method.to_owned(), params.clone()));
        }
    }
    
    fn send_action(&mut self, method: &str) {
        self.send_edit_cmd(method, &json!([]));
    }

    pub fn keydown(&mut self, keycode: VirtualKeyCode, mods: ModifiersState) -> bool {
        match keycode {
            VirtualKeyCode::Return => self.send_action("insert_newline"),
            VirtualKeyCode::Tab => {
                let action = if mods.shift() {
                    "outdent"
                } else {
                    "insert_tab"
                };
                self.send_action(action);
            },
            VirtualKeyCode::Up => {
                if mods.ctrl() {
                    self.scroll_offset -= LINE_SPACE;
                    self.constrain_scroll();
                    self.update_viewport();
                } else {
                    let action = if mods.ctrl() || mods.alt() {
                        "add_selection_above"
                    } else {
                        s(mods, "move_up", "move_up_and_modify_selection")
                    };

                    self.send_action(action);
                }
            },
            VirtualKeyCode::Down => {
                if mods.ctrl() {
                    self.scroll_offset += LINE_SPACE;
                    self.constrain_scroll();
                    self.update_viewport();
                } else {
                    let action = if mods.ctrl() || mods.alt() {
                        "add_selection_below"                        
                    } else {
                        s(mods, "move_down", "move_down_and_modify_selection")
                    };

                    self.send_action(action);
                }
            },
            VirtualKeyCode::Left => {
                let action = if mods.ctrl() {
                    s(mods, "move_word_left", "move_word_left_and_modify_selection")
                } else {
                    s(mods, "move_left", "move_left_and_modify_selection")
                };

                self.send_action(action);
            },
            VirtualKeyCode::Right => {
                let action = if mods.ctrl() {
                    s(mods, "move_word_right", "move_word_right_and_modify_selection")
                } else {
                    s(mods, "move_right", "move_right_and_modify_selection")
                };

                self.send_action(action);
            },
            VirtualKeyCode::PageUp => {
                self.send_action(s(mods, "scroll_page_up", "page_up_and_modify_selection"));
            },
            VirtualKeyCode::PageDown => {
                self.send_action(s(mods, "scroll_page_down", "page_down_and_modify_selection"));
            },
            VirtualKeyCode::Home => {
                let action = if mods.ctrl() {
                    s(mods, "move_to_beginning_of_document", "move_to_beginning_of_document_and_modify_selection")
                } else {
                    s(mods, "move_to_left_end_of_line", "move_to_left_end_of_line_and_modify_selection")
                };
                self.send_action(action);
            },
            VirtualKeyCode::End => {
                let action = if mods.ctrl() {
                    s(mods, "move_to_end_of_document", "move_to_end_of_document_and_modify_selection")  
                } else {
                    s(mods, "move_to_right_end_of_line", "move_to_right_end_of_line_and_modify_selection")
                };
                self.send_action(action);
            },
            VirtualKeyCode::F1 => self.set_theme("Solarized (dark)"),
            VirtualKeyCode::F2 => self.set_theme("Solarized (light)"),
            VirtualKeyCode::F3 => self.set_theme("InspiredGitHub"),
            VirtualKeyCode::Back => {
                let action = if mods.ctrl() {
                    s(mods, "delete_word_backward", "delete_to_beginning_of_line")
                } else {
                    "delete_backward"
                };
                self.send_action(action);
            },
            VirtualKeyCode::Delete => {
                let action = if mods.ctrl() {
                    s(mods, "delete_word_forward", "delete_to_end_of_paragraph")
                } else {
                    "delete_forward"
                };
                
                self.send_action(action);
            },
            _ => return false,
        }

        true
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
        }

        self.config = Some(config);
    }


    fn theme_changed(&mut self, theme: Theme) {
        println!("theme_changed: {:?}", theme);
        
        if let Some(col) = theme.foreground {
            self.resources.fg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        }
        if let Some(col) = theme.background {
            self.resources.bg = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        }
        if let Some(col) = theme.caret {
            self.resources.sel = rgba8_to_rgba32([col.r, col.g, col.b, col.a]);
        }

        self.theme = Some(theme);
    }

    fn set_theme(&mut self, theme_name: &str) {
        self.send_notification("set_theme", &json!({ "theme_name": theme_name }));
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
            EditViewCommands::SetTheme(theme) => self.set_theme(theme.as_str()),
            EditViewCommands::Undo => self.send_action("undo"),
            EditViewCommands::Redo => self.send_action("redo"),
            EditViewCommands::SelectAll => self.send_action("select_all"),
            _ => {
                return false;
            },
        }

        true
    }

    pub fn mouse_scroll(&mut self, delta: f32) {
        self.scroll_offset -= delta * 0.5; 
        self.constrain_scroll();
        self.update_viewport();
    }

    fn constrain_scroll(&mut self) {
        if self.scroll_offset < 0.0 {
            self.scroll_offset = 0.0;
            return;
        }
        
        let max_scroll = TOP_PAD + LINE_SPACE * (self.line_cache.height().saturating_sub(1)) as f32;
        if self.scroll_offset > max_scroll {
           self.scroll_offset = max_scroll; 
        }
    }

    fn y_to_line(&self, y: f32) -> usize {
        let mut line = (y + self.scroll_offset - TOP_PAD) / LINE_SPACE;
        if line < 0.0 { line = 0.0; }
        let line = line.floor() as usize;

        std::cmp::min(line, self.line_cache.height())
    }

    fn update_viewport(&mut self) {
        let first_line = self.y_to_line(0.0);
        let last_line = first_line + ((self.size[1] / LINE_SPACE).floor() as usize) + 1;
        let viewport = first_line..last_line;

        if viewport != self.viewport {
            self.viewport = viewport;
            self.send_edit_cmd("scroll", &json!([first_line, last_line]));
        }
    }

    #[inline]
    fn line_to_content_y(&self, line_num: usize) -> f32 {
        TOP_PAD + (line_num as f32) * LINE_SPACE
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

fn s<'a>(mods: ModifiersState, normal: &'a str, shifted: &'a str) -> &'a str {
    if mods.shift() { shifted } else { normal }
}
