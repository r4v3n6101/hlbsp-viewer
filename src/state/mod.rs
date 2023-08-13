use self::camera::Camera;
use crate::{render::Level, Args};
use anyhow::Result;
use glium::{Display, DrawParameters, Frame};

mod camera;

const MOVE_SPEED: f32 = 100.0;
const CAMERA_OFFSET: f32 = 64.0;

pub struct State {
    pub camera: Camera,
    level: Level,
}

impl State {
    pub fn new(display: &Display, args: &Args) -> Result<Self> {
        let mut camera = Camera::new(1024.0, 768.0, 90.0f32.to_radians(), 1.0, 8192.0, MOVE_SPEED);

        let level = Level::new(display, args)?;
        if let Some((x, y, z)) = level.start_point() {
            camera.set_position(x, y + CAMERA_OFFSET, z);
        }

        Ok(Self { camera, level })
    }

    pub fn render(&self, frame: &mut Frame, draw_params: &DrawParameters) {
        let projection = self.camera.perspective();
        let view = self.camera.view();

        self.level.render(frame, projection, view, &draw_params);
    }
}
