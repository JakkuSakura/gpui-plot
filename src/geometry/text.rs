use crate::geometry::{GeometryPixels, Point2};
use gpui::{Bounds, Pixels, SharedString, WindowContext};

pub struct Text {
    pub origin: Point2<Pixels, Pixels>,
    pub size: Pixels,
    pub text: String,
}
impl Text {
    pub fn render(&mut self, cx: &mut WindowContext) {
        let shared_string = SharedString::from(self.text.clone());
        let shaped_line = cx
            .text_system()
            .shape_line(shared_string, self.size, &[])
            .unwrap();
        shaped_line
            .paint(self.origin.into(), self.size, cx)
            .unwrap();
    }
}

impl GeometryPixels for Text {
    fn render_pixels(&mut self, _bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        self.render(cx);
    }
}
