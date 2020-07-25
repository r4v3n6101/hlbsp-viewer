use cgmath::{perspective, vec3, Angle, Deg, Euler, InnerSpace, Matrix4, Point3, Rad, Vector3};
use glium::implement_vertex;

pub type Scal = f32;

pub struct Camera {
    pub aspect_ratio: Scal,
    pub fov: Rad<Scal>,
    pub near: Scal,
    pub far: Scal,
    pub position: Point3<Scal>,
    pub rotation: Euler<Deg<Scal>>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 4.0 / 3.0,
            fov: Rad(std::f32::consts::PI / 2.0),
            near: 0.001,
            far: 1000.0,
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)),
        }
    }

    pub fn rotate_by(&mut self, pitch: Scal, yaw: Scal, roll: Scal) {
        self.rotation.x = Deg(self.rotation.x.0 + pitch);
        // TODO : repalce with clamp when it'll be stable
        if self.rotation.x.0 > 89.9 {
            self.rotation.x.0 = 89.9;
        } else if self.rotation.x.0 < -89.9 {
            self.rotation.x.0 = -89.9;
        }
        self.rotation.y += Deg(yaw);
        self.rotation.z += Deg(roll);
    }

    pub const fn up(&self) -> Vector3<Scal> {
        vec3(0.0, 1.0, 0.0)
    }

    pub fn forward(&self) -> Vector3<Scal> {
        vec3(
            self.rotation.y.cos() * self.rotation.x.cos(),
            self.rotation.x.sin(),
            self.rotation.y.sin() * self.rotation.x.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vector3<Scal> {
        self.forward().cross(self.up()).normalize()
    }

    pub fn move_forward(&mut self, speed: Scal) {
        self.position += self.forward() * speed;
    }

    pub fn move_back(&mut self, speed: Scal) {
        self.move_forward(-speed);
    }

    pub fn move_right(&mut self, speed: Scal) {
        self.position += self.right() * speed;
    }

    pub fn move_left(&mut self, speed: Scal) {
        self.move_right(-speed);
    }

    pub fn perspective(&self) -> Matrix4<Scal> {
        perspective(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn view(&self) -> Matrix4<Scal> {
        Matrix4::look_at(self.position, self.position + self.forward(), self.up())
    }
}

use map_impl::Vertex;
#[derive(Copy, Clone)]
pub struct GlVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
impl From<Vertex> for GlVertex {
    fn from(
        Vertex {
            position,
            tex_coords,
            normal,
        }: Vertex,
    ) -> Self {
        Self {
            position,
            tex_coords,
            normal,
        }
    }
}
implement_vertex!(GlVertex, position, tex_coords, normal);
