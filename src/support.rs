use cgmath::{perspective, vec3, Angle, Deg, Euler, InnerSpace, Matrix4, Point3, Rad, Vector3};

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
    pub fn new<A: Into<Rad<Scal>>>(
        width: Scal,
        height: Scal,
        fov: A,
        near: Scal,
        far: Scal,
    ) -> Self {
        Self {
            aspect_ratio: width / height,
            fov: fov.into(),
            near,
            far,
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)),
        }
    }

    pub fn rotate_by(&mut self, pitch: Scal, yaw: Scal, roll: Scal) {
        self.rotation.x = Deg(self.rotation.x.0 + pitch);
        // TODO : replace with clamp when it'll be stable
        if self.rotation.x.0 > 89.9 {
            self.rotation.x.0 = 89.9;
        } else if self.rotation.x.0 < -89.9 {
            self.rotation.x.0 = -89.9;
        }
        self.rotation.y += Deg(yaw);
        self.rotation.z += Deg(roll);
    }

    pub const fn up() -> Vector3<Scal> {
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
        self.forward().cross(Self::up()).normalize()
    }

    pub fn set_position(&mut self, x: Scal, y: Scal, z: Scal) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
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
        Matrix4::look_at_rh(self.position, self.position + self.forward(), Self::up())
    }
}

use log::{LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}]: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Debug))
}
