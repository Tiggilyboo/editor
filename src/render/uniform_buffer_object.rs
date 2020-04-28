use cgmath::{
    Rad,
    Deg,
    Matrix4,
    Vector3,
    Point3,
};
use super::camera::Camera;

#[derive(Debug)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub eye_position: Point3<f32>,
}

impl UniformBufferObject {
    pub fn new(
        camera: &Camera,
        dimensions: [f32; 2]
    ) -> UniformBufferObject {
        
        let model = Matrix4::from_angle_y(Rad::from(Deg(0.0)));

        let eye_position = camera.position;
        let view = camera.view_matrix(); 
        let proj = cgmath::perspective(
            Rad::from(Deg(camera.zoom)),
            dimensions[0] as f32 / dimensions[1] as f32,
            0.1,
            100.0,
        );

        UniformBufferObject { 
            model, 
            view, 
            proj,
            eye_position,
        }
    }
}
