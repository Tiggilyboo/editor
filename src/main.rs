#![macro_use]

extern crate cgmath;
extern crate winit;
extern crate vulkano;
extern crate vulkano_win;

extern crate image;

mod render;
mod events;
mod editor;

fn main() {
    editor::run("Editor");    
}

