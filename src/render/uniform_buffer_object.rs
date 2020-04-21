use cgmath::{
    Rad,
    Deg,
    Matrix4,
    Vector3,
    Point3,
};

#[derive(Debug)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn new(
        eye_position: Point3<f32>,
        look_direction: Vector3<f32>,
        pitch: f32,
        dimensions: [f32; 2]
    ) -> UniformBufferObject {
        
        let model = Matrix4::from_angle_z(Rad::from(Deg(0.0)));

        let view = Matrix4::look_at(
            eye_position,
            Point3::new(look_direction.x, look_direction.y, look_direction.z),
            Vector3::new(0.0, 0.0, -1.0),
        );

        let proj = cgmath::perspective(
            Rad::from(Deg(pitch)),
            dimensions[0] as f32 / dimensions[1] as f32,
            0.1,
            10.0,
        );

        UniformBufferObject { model, view, proj }
    }
}
