use vulkano;

#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub colour: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, colour);

vulkano_shaders::shader!{
    shaders: {
        vs: {
            ty: "vertex",
            path: "shaders/primitive_vertex.glsl",
        },
        fs: {
            ty: "fragment",
            path: "shaders/primitive_frag.glsl",
        }
    }
}

