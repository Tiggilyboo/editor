#![macro_use]

extern crate cgmath;
extern crate winit;
extern crate vulkano;
extern crate vulkano_win;
extern crate rusttype;

mod render;

use render::EditorApplication;

fn main() {
    let mut app = EditorApplication::new("Editor");

    app.run();    
}
