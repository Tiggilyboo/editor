use std::ops::Range;
use std::sync::{
    Mutex,
    Weak,
};
use std::mem;

use winit::event::{
    VirtualKeyCode,
    ModifiersState,
};

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

struct EditView {
    index: usize,
    dirty: bool,
    view_id: Option<String>,
    line_cache: LineCache,
    scroll_offset: f32,
    viewport: Range<usize>,
    core: Weak<Mutex<Core>>,
    pending: Vec<(Method, Params)>,
    resources: Option<Resources>,
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
            
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}

impl EditView {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            dirty: false,
            view_id: None,
            line_cache: LineCache::new(),
            resources: None,
            scroll_offset: 0.0,
            viewport: 0..0,
            core: Default::default(),
            pending: Default::default(),
        }
    }

    fn create_resources(&mut self, scale: f32) -> Resources {
        Resources {
            fg: [0.9, 0.9, 0.9, 1.0],
            bg: [0.1, 0.1, 0.1, 1.0],
            sel: [0.3, 0.3, 0.3, 1.0],
            scale,
        }
    }

    pub fn rebuild_resources(&mut self) {
        self.resources = None;
    }

    pub fn clear_line_cache(&mut self) {
        self.line_cache = LineCache::new();
    }

    fn get_line(&self, line_num: usize) -> Option<TextWidget> {
        self.line_cache
            .get_line(line_num)
            .map(|line| {
                let resources = &self.resources.as_ref().unwrap();
                TextWidget::from_line(line_num, &line, resources.scale, resources.fg)
            })
    }

    fn apply_update(&mut self, update: &Value) {
        self.line_cache.apply_update(update);
        self.constrain_scroll();
    }

    pub fn char(&mut self, ch: u32, _mods: u32) {
        if let Some(c) = ::std::char::from_u32(ch) {
            if ch >= 0x20 {
                let params = json!({"chars": c.to_string()});
                self.send_edit_cmd("insert", &params);
            }
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
            VirtualKeyCode::DOWN => {
                if mods.ctrl() {
                    self.scroll_offset += LINE_SPACE;
                    self.constrain_scroll();
                    self.update_viewport();
                    self.dirty = true;
                } else {
                    let action = if mods.ctrl() || mods.alt() {
                        
                    }
                    s(mods, "move_down", "move_down_and_modify_selection")
                }
            },
        }

        true
    }

    
}

fn s<'a>(mods: ModifiersState, normal: &'a str, shifted: &'a str) -> &'a str {
    if mods.shift() { shifted } else { normal }
}
