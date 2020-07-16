use crate::math::{perspective, vec3, Deg, Euler, Matrix3, Matrix4, Point3, Rad, Scal, Vector3};

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
            fov: Rad(3.14/2.0),
            near: 0.001,
            far: 100.0,
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)),
        }
    }

    pub fn rotate_by(&mut self, pitch: Scal, yaw: Scal, roll: Scal) {
        let pitch = (self.rotation.x.0 + pitch).max(90.0).min(0.0);
        let yaw = (self.rotation.y.0 + yaw) % 360.0;
        let roll = (self.rotation.z.0 + roll) % 90.0;
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
