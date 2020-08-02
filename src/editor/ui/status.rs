use glyph_brush::{
    Section,
    OwnedSection,
    OwnedText,
    Layout,
    Text,
};

use super::widget::Widget;
use super::text::TextWidget;
use super::primitive::PrimitiveWidget;
use super::view::Resources;

use crate::events::binding::Mode;
use crate::render::Renderer;

type ColourRGBA = [f32; 4];

const MODE_NORMAL_COLOUR: ColourRGBA = [0.0, 1.0, 0.0, 1.0];
const MODE_INSERT_COLOUR: ColourRGBA = [0.0, 0.0, 1.0, 1.0];
const MODE_SELECT_COLOUR: ColourRGBA = [0.5, 0.0, 0.5, 1.0];

pub struct StatusWidget {
    index: usize,
    position: [f32; 2],
    depth: f32,
    scale: f32,
    size: [f32; 2],
    bg_colour: ColourRGBA,
    fg_colour: ColourRGBA,
    status: Status,
    background: PrimitiveWidget,
    mode_primitive: PrimitiveWidget,
    mode_section: OwnedSection,
    filename_section: OwnedSection,
    status_section: OwnedSection,
    command_text: TextWidget,

    dirty: bool,
}

pub struct Status {
    pub mode: Mode,
    pub filename: Option<String>,
    pub line_current: usize,
    pub line_count: usize,
    pub language: Option<String>,
}

impl Widget for StatusWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        let (width, height) = (self.size[0], self.size[1]);
        let text_ctx = renderer.get_text_context().clone();

        // Primitives (Background quads)
        self.background.queue_draw(renderer);
        self.mode_primitive.queue_draw(renderer);

        // Mode
        self.command_text.queue_draw(renderer);

        // Filename
        text_ctx.borrow_mut().queue_text(&self.filename_section.to_borrowed());

        // Status
        let status_width = 6.0 + text_ctx.borrow().get_text_width(&self.status_section.text[0].text.to_string());
        self.status_section.screen_position = (width - status_width, self.position[1]);
        text_ctx.borrow_mut().queue_text(&self.status_section.to_borrowed());
    }
}

#[inline]
fn create_empty_section() -> OwnedSection {
    Section::default()
        .with_layout(Layout::default_single_line())
        .to_owned()
}

impl StatusWidget {
    pub fn new(index: usize, status: Status, resources: &Resources) -> Self {
        let background = PrimitiveWidget::new(3, [0.0, 0.0, 0.2], [0.0, 0.0], resources.bg);
        let mode_primitive = PrimitiveWidget::new(4, [0.0, 0.0, 0.2], [0.0, 0.0], MODE_NORMAL_COLOUR);
        let command_text = TextWidget::new(5, "", resources.scale, resources.fg, 0.2);

        let filename_section = create_empty_section();
        let mode_section = create_empty_section();
        let status_section = create_empty_section();

        Self {
            index,
            status,
            size: [0.0, 0.0],
            position: [0.0, 0.0],
            bg_colour: resources.bg,
            fg_colour: resources.fg,
            scale: resources.scale,
            depth: 0.5,
            dirty: true,
            background,
            mode_primitive,
            command_text,
            filename_section,
            mode_section,
            status_section,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = [x, y];
        self.dirty = true;
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        // TODO: Stack overflow???!?!?!
        let (width, height) = (size[0], size[1]);

        // Parent
        self.set_size([width, self.scale * 2.06]);
        self.set_position(width - self.size[0], height - self.size[1]);
        self.background.set_size(self.size);
        self.background.set_position(self.position[0], self.position[1]);

        // First row
        self.mode_primitive.set_position(self.position[0], self.position[1]);
        self.filename_section.screen_position = (self.position[0] + 64.0, self.position[1]);

        // Second row
        self.command_text.set_position(self.position[0], self.position[1] + (self.scale * 1.03));

        self.dirty = true;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode_section.text[0].text = mode.to_string();
        self.mode_section.text[0].extra.color = mode.colour();
        self.status.mode = mode; 
    }

    pub fn update_line_status(&mut self, line_num: usize, line_count: usize, language: Option<String>) {
        let line_percent: usize = if line_num > 0 {
            line_count / line_num * 100
        } else {
            0
        };

        let status_content = format!("{}\t{}%\t{}/{}", 
            language.clone().unwrap_or(String::new()), 
            line_percent,
            line_num, line_count);

        self.status.line_count = line_count;
        self.status.line_current = line_num;
        self.status.language = language;

        self.status_section.text = vec![ 
            OwnedText::from(&Text::new(status_content.as_str())
                .with_scale(self.scale)
                .with_color(self.fg_colour)
        )];
    }

    fn update_command_widget(&mut self, position: [f32; 2], command: String) {
        self.command_text.set_position(position[0], position[1]);

        let section = &mut self.command_text.get_section().to_borrowed();
        section.text[0].text = &command;
        self.command_text.set_dirty(true);
    }

    fn update_filename(&mut self, filename: Option<String>) {
        self.status.filename = filename;
    }

    pub fn mode(&self) -> Mode {
        self.status.mode
    }
}

impl Mode {
    fn colour(&self) -> ColourRGBA {
        match self {
            Mode::Normal => MODE_NORMAL_COLOUR,
            Mode::Insert => MODE_INSERT_COLOUR,
            Mode::Select => MODE_SELECT_COLOUR,
            _ => MODE_NORMAL_COLOUR,
        }
    }
}
