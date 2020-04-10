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
    pub fn from_dimensions(
        position: Point3<f32>,
        dimensions: [f32; 2]
    ) -> UniformBufferObject {
        
        let model = Matrix4::from_angle_z(Rad::from(Deg(0.0)));

        let view = Matrix4::look_at(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0)
        );

        let proj = cgmath::perspective(
            Rad::from(Deg(45.0)),
            dimensions[0] as f32 / dimensions[1] as f32,
            0.1,
            10.0
        );

        UniformBufferObject { model, view, proj }
    }
}
