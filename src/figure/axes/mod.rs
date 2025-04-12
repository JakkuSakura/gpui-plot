mod model;
#[cfg(feature = "plotters")]
mod plotters;
mod view;

pub use model::*;
#[cfg(feature = "plotters")]
pub use plotters::*;
use std::any::Any;
pub use view::*;

use crate::geometry::{AxesBounds, AxesBoundsPixels, AxisType, GeometryAxes, Point2};
use gpui::{App, Bounds, MouseMoveEvent, Pixels, Point, Window};

pub trait Axes: Any {
    fn update(&mut self);
    fn new_render(&mut self);
    fn pan_begin(&mut self, position: Point<Pixels>);
    fn pan(&mut self, event: &MouseMoveEvent);
    fn pan_end(&mut self);
    fn zoom_begin(&mut self, position: Point<Pixels>);
    fn zoom(&mut self, factor: f64);
    fn zoom_end(&mut self);
    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App);
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
    pub fn plot<T>(&mut self, mut element: impl AsMut<T>)
    where
        T: GeometryAxes<X = X, Y = Y>,
    {
        element.as_mut().render_axes(self);
    }
    pub fn contains(&self, point: Point2<X, Y>) -> bool {
        self.axes_bounds.contains(point)
    }
}
