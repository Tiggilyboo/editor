use vulkano;

#[derive(Default, Copy, Clone)]
pub struct Vertex {
  pub position: [f32; 2],
  pub tex_position: [f32; 2],
  pub colour: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, tex_position, colour);

vulkano_shaders::shader!{
    shaders: {
        vs: {
            ty: "vertex",
            path: "shaders/text_vertex.glsl",
        },
        fs: {
            ty: "fragment",
            path: "shaders/text_frag.glsl",
        }
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


