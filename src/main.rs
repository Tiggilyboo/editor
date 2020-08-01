#![macro_use]

extern crate cgmath;
extern crate winit;
extern crate vulkano;
extern crate vulkano_win;
extern crate syntect;

mod render;
mod events;
mod editor;
mod unicode;

use std::env;

fn main() {
    let filename = env::args().nth(1);
    
    editor::run("Editor", filename);    
}

