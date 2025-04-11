use crate::geometry::{GeometryPixels, Point2};
use gpui::{App, Bounds, Pixels, SharedString, Window};

pub struct Text {
    pub origin: Point2<Pixels, Pixels>,
    pub size: Pixels,
    pub text: String,
}
impl Text {
    pub fn render(
        &mut self,
        window: &mut Window,
        cx: &mut App,
        pixel_bounds: Option<Bounds<Pixels>>,
    ) {
        if let Some(bounds) = pixel_bounds {
            if !bounds.contains(&self.origin.into()) {
                return;
            }
        }
        let shared_string = SharedString::from(self.text.clone());
        let shaped_line = window
            .text_system()
            .shape_line(shared_string, self.size, &[])
            .unwrap();
        shaped_line
            .paint(self.origin.into(), self.size, window, cx)
            .unwrap();
    }
}

impl GeometryPixels for Text {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        self.render(window, cx, Some(bounds));
    }
}
