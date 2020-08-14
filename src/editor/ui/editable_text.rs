use std::hash::{
    Hasher,
    Hash,
};
use glyph_brush::{
    OwnedSection,
    Section,
    Text,
    Layout,
    ab_glyph::PxScale,
};

use super::{
    colour::ColourRGBA,
    widget::{
        Widget,
        hash_widget,
    },
    view::Resources,
};
use crate::rpc::{
    Action,
    Motion,
    Quantity,
};
use crate::render::Renderer;

pub const CURSOR_TEXT: &str = "\u{2588}";

pub struct EditableTextWidget {
    index: usize,
    dirty: bool,
    focused: bool,
    section: OwnedSection,
    cursor: OwnedSection,
    cursor_pos: usize,
}

impl Widget for EditableTextWidget {
    fn index(&self) -> usize {
        self.index
    }
    fn size(&self) -> [f32; 2] {
        [self.section.bounds.0, self.section.bounds.1]
    }
    fn position(&self) -> [f32; 2] {
        [self.section.screen_position.0, self.section.screen_position.1]
    }
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        renderer.get_text_context().borrow_mut()
            .queue_text(&self.section.to_borrowed());

        if self.focused {
            let mut text = self.text();
            if self.cursor_pos < text.len() {
                text = text[..self.cursor_pos].to_string();
            }
            let text_width = if text.len() > 0 {
                renderer.get_text_context().borrow().get_text_width(&text)
            } else {
                0.0
            };
            let text_pos = self.position();
            self.cursor.screen_position = (text_pos[0] + text_width, text_pos[1]);

            renderer.get_text_context().borrow_mut()
                .queue_text(&self.cursor.to_borrowed());
        }
    }
}

impl Hash for EditableTextWidget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state);
        self.cursor_pos.hash(state);
    }
}

impl EditableTextWidget {
    pub fn new(index: usize, resources: &Resources) -> Self {
        let section = Section::default()
            .add_text(Text::default()
                .with_scale(resources.scale)
                .with_color(resources.fg)
                .with_z(0.1))
            .with_layout(Layout::default_single_line())
            .to_owned();
        let cursor = Section::default()
            .add_text(Text::new(CURSOR_TEXT)
                .with_scale(resources.scale)
                .with_color(resources.sel)
                .with_z(0.2))
            .with_layout(Layout::default_single_line())
            .to_owned();

        Self {
            index,
            section,
            cursor,
            cursor_pos: 0,
            focused: false,
            dirty: true,
        }
    }
    pub fn text(&self) -> String {
        self.section.text[0].text.clone()
    }
    pub fn set_text(&mut self, text: &str) {
        self.cursor_pos = text.len();
        self.section.text[0].text = text.to_string();
        self.dirty = true;
    }
    pub fn set_colours(&mut self, fg: ColourRGBA, sel: ColourRGBA) {
        self.section.text[0].extra.color = fg;
        self.cursor.text[0].extra.color = sel;
        self.dirty = true;
    }
    pub fn set_scale(&mut self, scale: f32) {
        let px_scale = PxScale::from(scale);
        self.section.text[0].scale = px_scale;
        self.cursor.text[0].scale = px_scale;
        self.dirty = true;
    }
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.section.screen_position = (x, y);
        self.cursor.screen_position = (x, y);
        self.dirty = true;
    }
    pub fn set_size(&mut self, size: [f32; 2]) {
        self.section.bounds = (size[0], size[1]);
        self.cursor.bounds = (size[1], size[1]);
        self.dirty = true;
    }
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.dirty = true;
    }
    pub fn poke(&mut self, action: Action) -> bool {
        self.dirty = true;

        match action {
            Action::Delete((motion, quantity)) => self.handle_delete(motion, quantity),
            Action::Motion((motion, quantity)) => self.move_cursor(motion, quantity),
            Action::InsertChar(ch) => self.handle_char(ch),
            _ => {
                println!("Unhandled action: {:?}", action);
                false
            },
        }
    }

    fn move_cursor(&mut self, motion: Motion, quantity: Option<Quantity>) -> bool {
        let n = match quantity.unwrap_or_default() {
            Quantity::Number(c) => c,
            _ => 1,
        };

        match motion {
            Motion::Left => {
                if self.cursor_pos > n {
                    self.cursor_pos -= n;
                } else {
                    self.cursor_pos = 0;
                }
            },
            Motion::Right => {
                if self.cursor_pos + n < self.section.text[0].text.len() {
                    self.cursor_pos += n;
                } else {
                    self.cursor_pos = self.section.text[0].text.len();
                }
            },
            Motion::First => self.cursor_pos = 0,
            Motion::Last => self.cursor_pos = self.section.text[0].text.len(),
            _ => return false,
        }

        true
    }

    fn handle_char(&mut self, ch: char) -> bool {
        if self.cursor_pos <= self.text().len() {
            self.section.text[0].text.insert(self.cursor_pos, ch);
            self.move_cursor(Motion::Right, Some(Quantity::Number(1)));
        }
        true
    }

    fn handle_delete(&mut self, motion: Motion, quantity: Option<Quantity>) -> bool {
        let q = quantity.unwrap_or(Quantity::default());
        match q {
            Quantity::All | Quantity::Line(_) => {
                self.section.text[0].text = "".to_string();
            },
            Quantity::Number(n) => {
                let text_len = self.section.text[0].text.len();
                match motion {
                    Motion::Left => {
                        println!("cursor_pos: {}, text_len: {}, n: {}", self.cursor_pos, text_len, n);
                        if text_len > n
                        && self.cursor_pos > 0
                        && self.cursor_pos + n - 1 <= text_len {
                            self.section.text[0].text.remove(self.cursor_pos - n);
                            self.move_cursor(Motion::Left, None);
                        } else if text_len == 1 {
                            self.section.text[0].text.clear();
                            self.move_cursor(Motion::Left, None); 
                        }
                    },
                    Motion::Right => {
                        if self.cursor_pos + n < text_len {
                            self.section.text[0].text.remove(self.cursor_pos + n);
                        }
                    },
                    _ => return false,
                }
            }
            _ => return false,
        };

        true
    }
}
