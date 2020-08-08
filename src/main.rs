
extern crate glyph_brush;
extern crate serde;
extern crate serde_json;
extern crate winit;
extern crate xi_core_lib;
extern crate xi_rpc;

extern crate render;
extern crate rpc;

mod events;
mod editor;

use std::env;

fn main() {
    let filename = env::args().nth(1);
    
    editor::run("Editor", filename);    
}

