use std::f32::consts::FRAC_PI_2;
use glam::{Vec3, Quat, Mat4};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub aspect: f32,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub position: Vec3,
    pub rotation: Quat,
}

impl Camera {
    pub fn new(
        width: f32,
        height: f32,
        fov: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect: width / height,
            fov,
            z_near,
            z_far,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }

    pub fn rotate_by(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation.x += pitch.to_radians();
        // TODO : replace with clamp when it'll be stable
        if self.rotation.x > SAFE_FRAC_PI_2 {
            self.rotation.x = SAFE_FRAC_PI_2;
        } else if self.rotation.x < -SAFE_FRAC_PI_2 {
            self.rotation.x = -SAFE_FRAC_PI_2;
        }
        self.rotation.y += yaw.to_radians();
        self.rotation.z += roll.to_radians();
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.rotation.y.cos() * self.rotation.x.cos(),
            self.rotation.x.sin(),
            self.rotation.y.sin() * self.rotation.x.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
    }

    pub fn move_forward(&mut self, speed: f32) {
        self.position += self.forward() * speed;
    }

    pub fn move_back(&mut self, speed: f32) {
        self.move_forward(-speed);
    }

    pub fn move_right(&mut self, speed: f32) {
        self.position += self.right() * speed;
    }

    pub fn move_left(&mut self, speed: f32) {
        self.move_right(-speed);
    }

    pub fn perspective(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.z_near, self.z_far)
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward(), Vec3::Y)
    }
}
