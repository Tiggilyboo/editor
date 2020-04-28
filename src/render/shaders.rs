pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/vert.glsl"
    }
}
/*
pub mod tesselate_eval_shader {
    vulkano_shaders::shader! {
        ty: "tess_eval",
        path: "shaders/tess_eval.glsl"
    }
}

pub mod tesselate_ctrl_shader {
    vulkano_shaders::shader! {
        ty: "tess_ctrl",
        path: "shaders/tess_ctrl.glsl"
    }
}
*/
pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/frag.glsl"
    }
}
