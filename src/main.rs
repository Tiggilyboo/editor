mod events;
mod editor;

use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::Arc;
use std::env;

use render::Renderer;
use editor::{
    EditorState,
    EditorEvent,
};
use events::state::InputState;
use ui::widget::Widget;

use winit::{
    event_loop::{
        EventLoop,
        ControlFlow, 
    },
    event::{
        WindowEvent,
        Event,
    },
};

fn get_filepath() -> Option<String> {
   let args: Vec<String> = env::args().collect();

   if args.len() > 0 {
        if let Some(last_arg) = args.get(args.len()-1) {
            return Some(last_arg.to_string())
        }
   }


    None
}

fn setup_profile_server() -> puffin_http::Server {
    puffin::set_scopes_on(true);

    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    println!("Starting Profiler at {}", server_addr);
    let server = puffin_http::Server::new(&server_addr).unwrap();

    server
}

fn main() {
    let _profiler_server = setup_profile_server();

    let filepath = get_filepath();
    let el = EventLoop::<EditorEvent>::with_user_event();
    let proxy = el.create_proxy();

    let renderer = RefCell::new(Renderer::new(&el, "Editor"));
    let font_bounds = renderer.borrow_mut().get_text_renderer().borrow().get_font_bounds();
    let editor = Arc::new(Mutex::new(EditorState::new(proxy, font_bounds)));
    let input = Arc::new(Mutex::new(InputState::new()));
    
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    if let Ok(mut editor) = editor.lock() {
        editor.do_new_view(filepath);
    }

    el.run(move |event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        puffin::GlobalProfiler::lock().new_frame();
        {
            puffin::profile_scope!("WindowEvent");

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    puffin::profile_scope!("close_requested");
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {

                    puffin::profile_scope!("resized");
                    {
                        puffin::profile_scope!("recreate_swap_chain_next_frame");
                        renderer.borrow_mut().recreate_swap_chain_next_frame();
                    }
                    screen_dimensions[0] = size.width as f32;
                    screen_dimensions[1] = size.height as f32;

                    if let Ok(mut editor) = editor.lock() {
                        // TODO: input is processed at f32, other event handling is 64...
                        puffin::profile_scope!("resize");
                        editor.resize(size.width as f64, size.height as f64);
                    }

                    {
                        puffin::profile_scope!("request_redraw");
                        renderer.borrow().request_redraw();
                    }
                },
                Event::RedrawRequested(_window_id) => {

                    puffin::profile_scope!("redraw_requested");
                    if let Ok(editor) = editor.lock() {
                        puffin::profile_scope!("queue_draw");
                        for view_widget in editor.get_dirty_views() {
                            if let Ok(mut view_widget) = view_widget.lock() {
                                view_widget.queue_draw(&mut renderer.borrow_mut());
                                view_widget.set_dirty(false);
                            }
                            
                        }
                    }

                    {
                        puffin::profile_scope!("draw_frame");
                        renderer.borrow_mut().draw_frame();
                    }
                },
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::ModifiersChanged(_) => {
                        if let Ok(mut input) = input.lock() {
                            {
                                puffin::profile_scope!("input_update");
                                input.update(event, screen_dimensions);
                            }

                            if let Ok(mut editor) = editor.lock() {
                                {
                                    puffin::profile_scope!("process_input_actions");
                                    editor.process_input_actions(&input);   
                                }
                                if editor.requires_redraw() {
                                    puffin::profile_scope!("request_redraw");
                                    renderer.borrow().request_redraw();
                                }
                            }
                        }
                    },
                    WindowEvent::Focused(focus) => {
                        if let Ok(editor) = editor.lock() {
                            for view_widget in editor.get_views() {
                                if let Ok(mut view_widget) = view_widget.lock() {
                                    view_widget.set_dirty(true)
                                }
                            }
                        }
                      
                        if focus {
                            {
                                puffin::profile_scope!("focus: request_redraw");
                                renderer.borrow().request_redraw();
                            }
                        }
                    },
                    _ => {
                        // println!("Unhandled window event: {:?}", event);
                    },
                }
                _ => (),
            }
        }
    });
}
