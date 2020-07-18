#![macro_use]

extern crate cgmath;
extern crate winit;
extern crate vulkano;
extern crate vulkano_win;

mod render;
mod events;
mod editor;
mod unicode;

fn main() {
    editor::run("Editor");    
}

