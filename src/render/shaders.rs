pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/vert.glsl"
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/frag.glsl"
    }
}


