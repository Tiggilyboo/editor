

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/shader.frag"
    }
}

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shader.vert"
    }
}
