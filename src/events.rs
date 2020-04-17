use std::sync::Arc;
use std::cell::RefCell;

use winit::platform::desktop::EventLoopExtDesktop;

use winit::event_loop::{
    EventLoop,
    ControlFlow,
};  
use winit::event::{
    WindowEvent,
    Event,
    DeviceEvent,
    KeyboardInput,
    ElementState,
    VirtualKeyCode,
};

use crate::render::Renderer;

pub struct EventDelegator {
    events_loop: EventLoop<()>,
}

impl EventDelegator {
    pub fn new() -> Self {
        let events_loop = EventLoop::new();

        Self {
            events_loop,
        }
    }

    pub fn run(
        &mut self, 
        renderer: RefCell<Renderer>,
    ) {
        let mut events_loop = &mut self.get_events_loop();
        let mut done = false;
        let mut renderer = renderer
            .try_borrow_mut()
            .expect("unable to borrow renderer");

        while !done {
            events_loop.run_return(|event: Event<'_, ()>, _, control_flow: &mut ControlFlow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::UserEvent(event) => {
                        println!("user event: {:?}", event);
                    },
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        done = true;
                    },
                    Event::MainEventsCleared => {
                        *control_flow = ControlFlow::Exit;
                    },
                    Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                        renderer.recreate_swap_chain_next_frame();
                    },
                    Event::RedrawEventsCleared => {
                        renderer.draw_frame();
                    },
                    Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                        //mouse_delta += Vector2 { x: delta.0, y: delta.1 };
                        ()
                    },
                    Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => match input {
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state: ElementState::Released,
                            ..
                        } => match key {
                            VirtualKeyCode::A => {
                                ()
                                //text = String::from("a");
                            }
                            _ => ()
                        },
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    }

    pub fn get_events_loop(&mut self) -> &mut EventLoop<()> {
        &mut self.events_loop
    }
}
