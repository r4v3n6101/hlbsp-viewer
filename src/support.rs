use cgmath::{perspective, vec3, Deg, Euler, Matrix3, Matrix4, Point3, Rad, Vector3};
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
            fov: Rad(3.14 / 2.0),
            near: 0.001,
            far: 1000.0,
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)),
        }
    }

    pub fn rotate_by(&mut self, pitch: Scal, yaw: Scal, roll: Scal) {
        let pitch = self.rotation.x.0 + pitch;
        let yaw = self.rotation.y.0 + yaw;
        let roll = self.rotation.z.0 + roll;
        self.rotation.x = Deg(pitch);
        self.rotation.y = Deg(yaw);
        self.rotation.z = Deg(roll);
    }

    // TODO : need tests
    pub fn direction(&self) -> Vector3<Scal> {
        Matrix3::from(self.rotation) * vec3(1.0, 1.0, 1.0)
    }

    pub fn perspective(&self) -> Matrix4<Scal> {
        perspective(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn view(&self) -> Matrix4<Scal> {
        Matrix4::look_at(
            self.position,
            self.position + self.direction(),
            vec3(0.0, 1.0, 0.0),
        )
    }
}

#[derive(Copy, Clone)]
pub struct GlVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
impl From<map_impl::Vertex> for GlVertex {
    fn from(t: map_impl::Vertex) -> Self {
        Self {
            position: t.position,
            tex_coords: t.tex_coords,
            normal: t.normal,
        }
    }
}
implement_vertex!(GlVertex, position, tex_coords, normal);
