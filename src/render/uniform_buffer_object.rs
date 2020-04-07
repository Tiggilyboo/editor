use cgmath::{
    Rad,
    Deg,
    Matrix4,
    Vector3,
    Point3
};
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::buffer::{
    BufferUsage,
    CpuBufferPool,
    CpuAccessibleBuffer,
};

use crate::render::shaders::vertex_shader;

pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl UniformBufferObject {

    pub fn from_dimensions(dimensions: [f32; 2]) -> UniformBufferObject {
        let model = Matrix4::from_angle_z(Rad::from(Deg(15.0)));
        let view = Matrix4::look_at(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        );
        let mut proj = cgmath::perspective(
            Rad::from(Deg(45.0)),
            dimensions[0] as f32 / dimensions[1] as f32,
            0.1,
            10.0
        );
        proj.y.y *= -1.0;

        UniformBufferObject { 
            model: model, 
            view: view, 
            proj: proj 
        }
    }
}
