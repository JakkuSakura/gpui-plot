use gpui::{IntoElement, Render, ViewContext};
use std::time::Instant;

pub struct FpsModel {
    pub fps: f32,
    pub last_time: Instant,
    pub last_fps: f64,
    pub frame_count: f32,
}
impl FpsModel {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            last_time: Instant::now(),
            last_fps: 0.0,
            frame_count: 0.0,
        }
    }
    pub fn next_fps(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now - self.last_time;
        self.frame_count += 1.0;
        if delta.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count / delta.as_secs_f32();
            self.frame_count = 0.0;
            self.last_time = now;
        }
        let fps = self.fps;
        fps
    }
}
pub struct FpsViewer {
    pub model: FpsModel,
}
impl FpsViewer {
    pub fn new() -> Self {
        Self {
            model: FpsModel::new(),
        }
    }
}
impl Render for FpsViewer {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let fps = self.model.next_fps();
        let text = format!("fps: {:.2}", fps);
        text
    }
}