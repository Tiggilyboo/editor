extern crate scribe;

mod events;


use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::{Arc};
use render::Renderer;
use events::state::InputState;

use winit::event_loop::{
    EventLoop,
    ControlFlow, 
};
use winit::event::{
    WindowEvent,
    Event,
};

enum EditorEvent {}

fn main() {
    let el = EventLoop::<EditorEvent>::with_user_event();
    let renderer = RefCell::new(Renderer::new(&el, "Editor"));
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();
    let input = Arc::new(Mutex::new(InputState::new()));

    el.run(move |event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                renderer.borrow_mut().recreate_swap_chain_next_frame();
                screen_dimensions[0] = size.width as f32;
                screen_dimensions[1] = size.height as f32;

                renderer.borrow().request_redraw();
            },
            Event::RedrawRequested(_window_id) => {
                renderer.borrow_mut().draw_frame();
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::Focused(_) => {
                    if let Ok(mut input) = input.try_lock() {
                        input.update(event, screen_dimensions);
                    } else {
                        panic!("Unable to lock input")
                    }
                },
                _ => {
                    //println!("Unhandled window event: {:?}", event);
                },
            }
            _ => (),
        }
    });
}
