use std::sync::Arc;
use vulkano;
use vulkano::device::Queue;
use vulkano::buffer::{
    BufferUsage,
    ImmutableBuffer,
};

pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position, color);

impl Default for Vertex {
    fn default() -> Vertex {
        Self { 
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        }
    }
}
impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        Self { 
            position: self.position,
            color: self.color,
        }
    }
}

fn vertices() -> [Vertex; 4] {
    [
        Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
        Vertex { position: [0.5, -0.5], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, 0.5], color: [0.0, 0.0, 1.0] },
        Vertex { position: [-0.5, 0.5], color: [1.0, 1.0, 1.0] },
    ]
}

fn indices() -> [u16; 6] {
    [0, 1, 2, 2, 3, 0]
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
