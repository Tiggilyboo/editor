use vulkano;

#[derive(Default, Debug, Clone)]
pub struct TextVertex {
    position: [f32; 2],
    tex_position: [f32; 2],
    colour: [f32; 4],
}
vulkano::impl_vertex!(TextVertex, position, tex_position, colour);

impl TextVertex {
    pub fn new(position: [f32; 2], tex_position: [f32; 2], colour: [f32; 4]) -> Self {
        Self {
            position,
            tex_position,
            colour,
        }
    }
}


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
