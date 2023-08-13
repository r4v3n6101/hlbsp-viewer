use self::{camera::Camera, level::Level};
use crate::Args;
use anyhow::Result;
use glium::{Display, Surface, DrawParameters};

mod level;
mod camera;

const CAMERA_OFFSET: f32 = 64.0;

pub struct State {
    pub mouse_grabbed: bool,
    pub camera: Camera,
    level: Level,
}

impl State {
    pub fn new(display: &Display, args: &Args) -> Result<Self> {
        let mut camera = Camera::new(1024.0, 768.0, 90.0f32.to_radians(), 1.0, 8192.0, 100.0);

        let level = Level::new(&display, args)?;
        if let Some((x, y, z)) = level.start_point() {
            camera.set_position(x, y + CAMERA_OFFSET, z);
        }

        let mouse_grabbed = true;

        Ok(Self {
            mouse_grabbed,
            camera,
            level,
        })
    }

    pub fn render(&self, display: &Display, draw_params: &DrawParameters) {
        let mut frame = display.draw();

        frame.clear_color_and_depth((1.0, 1.0, 0.0, 1.0), 1.0);

        let projection = self.camera.perspective();
        let view = self.camera.view();

        self.level.render(&mut frame, projection, view, &draw_params);

        frame.finish().unwrap();
    }
}
