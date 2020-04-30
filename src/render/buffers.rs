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
    const SIZE: usize = 32;
    let mut verts = vec!();

    for x in 0..SIZE {
        for y in 0..SIZE {
            verts.push(Vertex { 
                position: [
                    -160.0 * (x as f32 / SIZE as f32) + 160.0, 
                    -160.0 * (y as f32 / SIZE as f32) + 160.0, 
                    0.0,
                ],
                colour: [0.25, 0.4, 0.25, 1.0]
            });
        }
    }

    verts
}

/*
 0, 1
 2, 3

 0  1  2  3 
 4  5  6  7 
 8  9  10 11

*/
fn indices() -> Vec<u16> {
    //[0, 1, 2, 2, 3, 0]
    const SIZE: u16 = 32; 
    let mut indices = vec!();

    // 0 --> 1
    //    /  ^
    //  /    |
    // 32 -> 33
    for x in 0..SIZE - 1 {
        for y in 0..SIZE -1 {
            let w = y * SIZE;
            let z = w + SIZE;
            
            indices.push(w + x);
            indices.push(w + x + 1);
            indices.push(z + x);

            indices.push(z + x);
            indices.push(z + x + 1);
            indices.push(w + x + 1);
        }
    }

    indices
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
