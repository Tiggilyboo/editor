use vulkano;

use cgmath::Matrix4;

#[derive(Default, Debug, Clone)]
pub struct TextVertex {
    pub left_top: [f32; 2],
    pub right_bottom: [f32; 2],
    pub depth: f32,
    pub tex_left_top: [f32; 2],
    pub tex_right_bottom: [f32; 2],
    pub colour: [f32; 4],
}

pub struct TextTransform {
    pub transform: Matrix4<f32>
}

#[derive(Default, Copy, Clone)]
pub struct Vertex {
  pub position: [f32; 2],
  pub tex_position: [f32; 2],
  pub colour: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, tex_position, colour);

pub mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/text_vertex.glsl",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/text_frag.glsl",
    }
}

impl Vertex {
    pub fn new(position: [f32; 2], tex_position: [f32; 2], colour: [f32; 4]) -> Self {
        Self {
            position,
            tex_position,
            colour,
        }
    }
}

impl TextVertex {

    #[inline]
    pub fn to_verts(&self) -> [Vertex; 4] {
        [
            // 1  2  5  6
            // 3  4  7  8
            Vertex::new([self.left_top[0], self.left_top[1]], self.tex_left_top, self.colour),
            Vertex::new([self.left_top[0], self.right_bottom[1]], [self.tex_left_top[0], self.tex_right_bottom[1]], self.colour),
            Vertex::new([self.right_bottom[0], self.left_top[1]], [self.tex_right_bottom[0], self.tex_left_top[1]], self.colour),
            Vertex::new([self.right_bottom[0], self.right_bottom[1]], self.tex_right_bottom, self.colour),
        ]
    }
}
