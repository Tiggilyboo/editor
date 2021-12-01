use vulkano;

#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub colour: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, colour);

pub mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/primitive_vertex.glsl",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/primitive_frag.glsl",
    }
}

