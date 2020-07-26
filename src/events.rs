pub mod state;
pub mod mapper_winit;
pub mod binding;

use winit::event_loop::EventLoop;
use crate::editor::ui::widget::WidgetKind;

pub enum EditorEvent {
    OpenWidget(WidgetKind), 
}

pub type EditorEventLoop = EventLoop<EditorEvent>;

pub fn create_event_loop() -> EditorEventLoop {
    EventLoop::<EditorEvent>::with_user_event()
}

