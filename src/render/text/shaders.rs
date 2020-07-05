use vulkano;

use cgmath::Matrix4;

#[derive(Default, Debug, Clone)]
pub struct TextVertex {
    pub left_top: [f32; 3],
    pub right_bottom: [f32; 2],
    pub tex_left_top: [f32; 2],
    pub tex_right_bottom: [f32; 2],
    pub colour: [f32; 4],
}
vulkano::impl_vertex!(TextVertex, left_top, right_bottom, tex_left_top, tex_right_bottom, colour);

pub struct TextTransform {
    pub transform: Matrix4<f32>
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

