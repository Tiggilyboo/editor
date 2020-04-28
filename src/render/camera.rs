use cgmath;
use cgmath::vec3;
use cgmath::prelude::*;

type Point3 = cgmath::Point3<f32>;
type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

pub struct Camera {
    pub position: Point3,
    pub front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,

    // Eulars!
    pub yaw: f32,
    pub pitch: f32,

    pub move_speed: f32,
    pub mouse_speed: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let mut camera = Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            front: vec3(0.0, 0.0, 1.0),
            up: Vector3::zero(),
            right: Vector3::zero(),
            world_up: Vector3::unit_y(),
            yaw: -90.0,
            pitch: 45.0,
            move_speed: 0.005,
            mouse_speed: 0.25,
            zoom: 45.0,
        };
        camera.update();

        camera
    }
}

impl Camera {
    pub fn view_matrix(&self) -> Matrix4 {
        Matrix4::look_at_dir(self.position, self.front, self.up)
    }

    pub fn move_camera(&mut self, direction: (f32, f32), delta_time: f32) {
        let velocity = self.move_speed * delta_time;

        if direction.0 > 0.0 {
            self.position += self.front * velocity;
        } else if direction.0 < 0.0 {
            self.position -= self.front * velocity;
        }

        if direction.1 > 0.0 {
            self.position += self.right * velocity;
        } else if direction.1 < 0.0 {
            self.position -= self.right * velocity;
        }
    }

    pub fn zoom(&mut self, delta: f32) {
        if self.zoom >= 1.0 && self.zoom <= 45.0 {
            self.zoom -= delta;
        }
        if self.zoom <= 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom >= 45.0 {
            self.zoom = 45.0;
        }
    }

    pub fn direction(&mut self, mouse_delta: (f32, f32)) {
        let (x, y) = (
            mouse_delta.0 * self.mouse_speed,
            mouse_delta.1 * self.mouse_speed);
        
        self.yaw += x;
        self.pitch += y;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.update();    
    }

    fn update(&mut self){
        let front = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };

        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
