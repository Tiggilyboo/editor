pub mod state;
pub mod mapper_winit;
pub mod binding;

use winit::event_loop::{
    EventLoop,
    EventLoopProxy,
};
use rpc::Action;

#[derive(Clone)]
pub enum EditorEvent {
    Action(Action),
}

pub type EditorEventLoopProxy = EventLoopProxy<EditorEvent>;

pub fn create_event_loop() -> EventLoop<EditorEvent> {
    EventLoop::<EditorEvent>::with_user_event()
}

