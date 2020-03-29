#![macro_use]

extern crate winit;
extern crate vulkano;
extern crate vulkano_win;

mod render;

use render::EditorApplication;


fn main() {
    let app = EditorApplication::new("Editor");

    app.run_events();    
}
