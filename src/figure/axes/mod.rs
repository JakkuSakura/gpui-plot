mod model;
#[cfg(feature = "plotters")]
mod plotters;
mod view;

pub use model::*;
#[cfg(feature = "plotters")]
pub use plotters::*;
use std::any::Any;
pub use view::*;

use crate::figure::SharedModel;
use crate::geometry::{AxesBounds, AxesBoundsPixels, AxisType, GeometryAxes, Point2};
use gpui::{App, Bounds, MouseMoveEvent, Pixels, Point, Window};

pub trait Axes: Any {
    fn update(&mut self);
    fn new_render(&mut self);
    fn pan_begin(&mut self, position: Point<Pixels>);
    fn pan(&mut self, event: &MouseMoveEvent);
    fn pan_end(&mut self);
    fn zoom_begin(&mut self, position: Point<Pixels>);
    fn zoom(&mut self, zoom_in: f64);
    fn zoom_end(&mut self);
    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App);
}
impl<T: Axes + 'static> Axes for SharedModel<T> {
    fn update(&mut self) {
        self.write().update();
    }

    fn new_render(&mut self) {
        self.write().new_render();
    }
    fn pan_begin(&mut self, position: Point<Pixels>) {
        self.write().pan_begin(position);
    }

    fn pan(&mut self, event: &MouseMoveEvent) {
        self.write().pan(event);
    }

    fn pan_end(&mut self) {
        self.write().pan_end();
    }
    fn zoom_begin(&mut self, position: Point<Pixels>) {
        self.write().zoom_begin(position);
    }
    fn zoom(&mut self, delta: f64) {
        self.write().zoom(delta);
    }
    fn zoom_end(&mut self) {
        self.write().zoom_end();
    }

    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        self.write().render(bounds, window, cx);
    }
}
pub struct AxesContext<'a, X: AxisType, Y: AxisType> {
    pub axes_bounds: AxesBounds<X, Y>,
    pub pixel_bounds: AxesBoundsPixels,
    pub cx: Option<(&'a mut Window, &'a mut App)>,
}
impl<'a, X: AxisType, Y: AxisType> AxesContext<'a, X, Y> {
    pub fn new(model: &AxesModel<X, Y>, window: &'a mut Window, cx: &'a mut App) -> Self {
        Self {
            axes_bounds: model.axes_bounds,
            pixel_bounds: model.pixel_bounds,
            cx: Some((window, cx)),
        }
    }
    pub fn new_without_context(model: &AxesModel<X, Y>) -> Self {
        Self {
            axes_bounds: model.axes_bounds,
            pixel_bounds: model.pixel_bounds,
            cx: None,
        }
    }
    pub fn transform_point(&self, point: Point2<X, Y>) -> Point<Pixels> {
        self.axes_bounds.transform_point(self.pixel_bounds, point)
    }
    pub fn plot(&mut self, mut element: impl GeometryAxes<X = X, Y = Y> + 'static) {
        element.render_axes(self);
    }
    pub fn contains(&self, point: Point2<X, Y>) -> bool {
        self.axes_bounds.contains(point)
    }
}
