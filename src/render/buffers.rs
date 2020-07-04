use std::sync::Arc;
use vulkano;
use vulkano::device::Queue;
use vulkano::buffer::{
    BufferUsage,
    ImmutableBuffer,
};

pub struct Vertex {
    position: [f32; 3],
    colour: [f32; 4],
}

vulkano::impl_vertex!(Vertex, position, colour);

impl Default for Vertex {
    fn default() -> Vertex {
        Self { 
            position: [0.0, 0.0, 0.0],
            colour: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        Self { 
            position: self.position,
            colour: self.colour,
        }
    }
}

fn vertices<'a>() -> Vec<Vertex> {
    vec!()
}

fn indices() -> Vec<u16> {
    vec!()
}

pub fn create_vertex_buffer(graphics_queue: &Arc<Queue>) -> Arc<ImmutableBuffer<[Vertex]>> { 
    let (buffer, _) = ImmutableBuffer::from_iter(
        vertices().iter().cloned(), 
        BufferUsage::vertex_buffer(),
        graphics_queue.clone()).unwrap();

    buffer
}

pub fn create_index_buffer(graphics_queue: &Arc<Queue>) -> Arc<ImmutableBuffer<[u16]>> {
    let (buffer, _) = ImmutableBuffer::from_iter(
        indices().iter().cloned(),
        BufferUsage::index_buffer(),
        graphics_queue.clone()) .unwrap();

    buffer
}
