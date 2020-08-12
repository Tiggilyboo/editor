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

pub struct EditableTextWidget {
    index: usize,
    dirty: bool,
    section: OwnedSection,
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
                .with_color(resources.fg))
            .with_layout(Layout::default_single_line())
            .to_owned();

        Self {
            index,
            dirty: true,
            section,
            cursor_pos: 0,
        }
    }
    pub fn text(&self) -> String {
        self.section.text[0].text.clone()
    }
    pub fn set_text(&mut self, text: &str) {
        self.section.text[0].text = text.to_string();
    }
    pub fn set_colour(&mut self, colour: ColourRGBA) {
        self.section.text[0].extra.color = colour;
    }
    pub fn set_scale(&mut self, scale: f32) {
        self.section.text[0].scale = PxScale::from(scale);
    }
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.section.screen_position = (x, y);
    }
    pub fn set_size(&mut self, size: [f32; 2]) {
        self.section.bounds = (size[0], size[1]);
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
        if self.cursor_pos < self.text().len() {
            self.section.text[0].text.insert(self.cursor_pos, ch);
        }
        self.move_cursor(Motion::Right, Some(Quantity::default()));
        true
    }

    fn handle_delete(&mut self, motion: Motion, quantity: Option<Quantity>) -> bool {
        let q = quantity.unwrap_or_default();
        match q {
            Quantity::All | Quantity::Line(_) => {
                self.section.text[0].text = "".to_string();
            },
            Quantity::Number(n) => {
                for c in 0..n {
                    match motion {
                        Motion::Left => {
                            if self.section.text[0].text.len() > 0
                            && self.cursor_pos < self.section.text[0].text.len() {
                                self.section.text[0].text.remove(self.cursor_pos);
                                self.move_cursor(Motion::Left, Some(Quantity::default()));
                            }
                        },
                        Motion::Right => {
                            if self.cursor_pos + 1 < self.section.text[0].text.len() {
                                self.section.text[0].text.remove(self.cursor_pos + 1);
                            }
                        },
                        _ => break,
                    }
                }
            }
            _ => return false,
        };

        true
    }
}
