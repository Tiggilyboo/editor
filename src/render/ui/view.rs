use std::ops::Range;
use std::sync::{
    Mutex,
    Weak,
};

use winit::event::{
    VirtualKeyCode,
    ModifiersState,
};

use crate::render::text::TextContext;
use crate::editor::rpc::Core;
use crate::editor::linecache::LineCache;
use serde_json::{
    json,
    Value,
};

use crate::render::Renderer;
use crate::render::ui::{
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
    Undo,
    Redo,
    UpperCase,
    LowerCase,
    Transpose,
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
    resources: Resources,
    size: [f32; 2],
}

const TOP_PAD: f32 = 6.0;
const LEFT_PAD: f32 = 6.0;
const LINE_SPACE: f32 = 17.0;

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
        for line_num in first_line..last_line {
            if let Some(text_widget) = &mut self.get_line(line_num) {
                text_widget.set_position(x0, y);
                text_widget.queue_draw(renderer);
            }
            y += LINE_SPACE;
        }
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}

impl EditView {
    pub fn new(index: usize, size: [f32; 2], scale: f32) -> Self {
        Self {
            index,
            size,
            dirty: true,
            view_id: None,
            line_cache: LineCache::new(),
            resources: Resources {
                fg: [0.9, 0.9, 0.9, 1.0],
                bg: [0.1, 0.1, 0.1, 1.0],
                sel: [0.3, 0.3, 0.3, 1.0],
                scale,
            },
            scroll_offset: 0.0,
            viewport: 0..0,
            core: Default::default(),
            pending: Default::default(),
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

    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
        self.dirty = true;
    }

    pub fn char(&mut self, ch: char) {
        if ch as u32 >= 0x20 {
            let params = json!({"chars": ch.to_string()});
            self.send_edit_cmd("insert", &params);
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
            self.pending.push((method.to_owned(), params.clone()));
        }
    }
    
    fn send_action(&mut self, method: &str) {
        self.send_edit_cmd(method, &json!([]));
    }

    pub fn keydown(&mut self, keycode: VirtualKeyCode, mods: ModifiersState) -> bool {
        match keycode {
            VirtualKeyCode::Return => self.send_action("insert_newline"),
            VirtualKeyCode::Tab => self.send_action("insert_tab"),
            VirtualKeyCode::Up => {
                if mods.ctrl() {
                    self.scroll_offset -= LINE_SPACE;
                    self.constrain_scroll();
                    self.update_viewport();
                    self.dirty = true;
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
                    self.dirty = true;
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
            VirtualKeyCode::Escape => {
                self.send_action("cancel_operation");
            },
            VirtualKeyCode::Back => {
                let action = if mods.ctrl() {
                    s(mods, "delete_word_backword", "delete_to_beginning_of_line")
                } else {
                    "delete_backword"
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

    pub fn poke(&mut self, command: EditViewCommands) -> bool {
        match command {
            EditViewCommands::ViewId(view_id) => {
                self.view_id = Some(view_id.to_string());
                self.viewport = 0..0;
                self.update_viewport();

                let pending = std::mem::replace(&mut self.pending, Vec::new());
                for notification in pending {
                    let (method, params) = notification;
                    self.send_edit_cmd(&method, &params);
                }
            },
            EditViewCommands::Core(core) => self.core = core.clone(),
            EditViewCommands::ApplyUpdate(update) => self.apply_update(&update),
            EditViewCommands::ScrollTo(line) => self.scroll_to(line),
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

    fn xy_to_line_col(&self, text_context: &TextContext, x: f32, y: f32) -> (usize, usize) {
        let line_num = self.y_to_line(y);
        let col = if let Some(text_line) = 
            &mut self.get_line(line_num)
        {
            text_line.hit_test(text_context, x, y)
        } else {
            0
        };

        (line_num, col)
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
        } else if y > self.scroll_offset + self.size[1] - bottom_slop {
            self.scroll_offset = y - (self.size[1] - bottom_slop)
        }
    }

}

fn s<'a>(mods: ModifiersState, normal: &'a str, shifted: &'a str) -> &'a str {
    if mods.shift() { shifted } else { normal }
}
