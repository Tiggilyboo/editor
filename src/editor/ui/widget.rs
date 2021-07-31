use std::hash::{
    Hash,
    Hasher,
};
use std::collections::hash_map::DefaultHasher;

use crate::render::Renderer;

pub trait Widget: Hash {
    fn index(&self) -> usize;
    fn position(&self) -> [f32; 2];
    fn size(&self) -> [f32; 2];
    fn queue_draw(&mut self, renderer: &mut Renderer);
    fn dirty(&self) -> bool;
}

#[inline]
fn hash_f32<H: Hasher>(value: f32, state: &mut H) { 
    let b = value.to_le_bytes();
    b.hash(state);
}
fn hash_f32slice<H: Hasher>(value: &[f32], state: &mut H) {
    for v in value.iter() {
        hash_f32(*v, state);
    }
}

pub fn hash_widget<H: Hasher, T: Widget>(widget: &T, state: &mut H) {
    widget.index().hash(state);
    widget.dirty().hash(state);
    
    hash_f32slice(&widget.position(), state);
    hash_f32slice(&widget.size(), state);
}

pub fn calculate_hash<T: Widget>(widget: &T) -> usize {
    let mut s = DefaultHasher::new();
    hash_widget(widget, &mut s);

    s.finish() as usize
}
