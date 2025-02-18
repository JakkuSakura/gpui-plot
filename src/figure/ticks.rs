use crate::figure::axes::AxesModel;
use crate::geometry::{point2, AxisType, GeometryPixels, Text};
use gpui::{px, App, Bounds, Pixels, Window};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct TicksViewer<X: AxisType, Y: AxisType> {
    context: Arc<RwLock<AxesModel<X, Y>>>,
}
impl<X: AxisType, Y: AxisType> TicksViewer<X, Y> {
    pub fn new(context: Arc<RwLock<AxesModel<X, Y>>>) -> Self {
        Self { context }
    }
    pub fn render(&mut self, window: &mut Window, cx: &mut App) {
        let context = self.context.read();
        let size = px(12.0);

        for x in context.grid.grid_x_lines.iter().cloned() {
            let text = x.format();
            let x_px = context.axes_bounds.x.transform(context.pixel_bounds.x, x)
                - size * text.len() / 2.0 * 0.5;
            let y_px = context.pixel_bounds.max_y() + px(3.0);
            Text {
                origin: point2(x_px, y_px),
                size,
                text,
            }
            .render(window, cx);
        }
        for y in context.grid.grid_y_lines.iter().cloned() {
            let text = y.format();

            let x_px = context.pixel_bounds.min_x() - size * text.len() as f32 * 0.5 - px(3.0);
            let y_px = context.axes_bounds.y.transform(context.pixel_bounds.y, y) - size / 2.0;
            Text {
                origin: point2(x_px, y_px),
                size,
                text,
            }
            .render(window, cx);
        }
    }
}
impl<X: AxisType, Y: AxisType> GeometryPixels for TicksViewer<X, Y> {
    fn render_pixels(&mut self, _bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        self.render(window, cx);
    }
}
