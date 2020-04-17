use super::render::Renderer;
use super::events::EventDelegator;

use std::cell::RefCell;
use std::thread;

use winit::event_loop::EventLoop;

use std::sync::Arc;
use std::boxed::Box;

pub fn run(title: &str) {
    let mut events_delegator = EventDelegator::new();
    let events_loop = events_delegator.get_events_loop();
    let renderer = RefCell::from(Renderer::new(&*events_loop, "Editor"));

    events_delegator.run(renderer) 
}
