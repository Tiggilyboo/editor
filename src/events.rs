pub mod state;
pub mod mapper_winit;
pub mod binding;

use winit::event_loop::EventLoop;

pub enum EditorEvent {}

pub fn create_event_loop() -> EventLoop<EditorEvent> {
    EventLoop::<EditorEvent>::with_user_event()
}

