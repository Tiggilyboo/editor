use cgmath::Matrix4;

pub struct UniformTransform {
    pub transform: Matrix4<f32>
}

#[inline]
pub fn calculate_transform(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> UniformTransform {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);

    UniformTransform {
        transform: cgmath::Matrix4::new(
            2.0 / (right - left), 0.0, 0.0, 0.0,
            0.0, 2.0 / (top - bottom), 0.0, 0.0,
            0.0, 0.0, -2.0 / (far - near), 0.0,
            tx, ty, tz, 1.0,
        ),
    }
}
