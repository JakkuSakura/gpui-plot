mod model;
#[cfg(feature = "plotters")]
mod plotters;
mod view;
pub use model::*;
#[cfg(feature = "plotters")]
pub use plotters::*;
pub use view::*;

use crate::figure::SharedModel;
use crate::geometry::{
    AxesBounds, AxesBoundsPixels, AxisType, GeometryAxes, GeometryPixels, Point2,
};
use gpui::{App, MouseMoveEvent, Pixels, Point, Window};
use parking_lot::RwLock;
use std::sync::Arc;

pub trait Axes: GeometryPixels {
    fn update(&mut self);
    fn get_model(&self) -> &dyn DynAxesModel;
    fn get_model_mut(&mut self) -> &mut dyn DynAxesModel;
    fn pan_begin(&mut self, position: Point<Pixels>);
    fn pan(&mut self, event: &MouseMoveEvent);
    fn pan_end(&mut self);
    fn zoom(&mut self, point: Point<Pixels>, delta: f32);
}

pub struct AxesContext<'a, X: AxisType, Y: AxisType> {
    pub model: SharedModel<AxesModel<X, Y>>,
    pub axes_bounds: AxesBounds<X, Y>,
    pub pixel_bounds: AxesBoundsPixels,
    pub cx: Option<(&'a mut Window, &'a mut App)>,
    pub new_axes_bounds: Option<AxesBounds<X, Y>>,
}
impl<'a, X: AxisType, Y: AxisType> AxesContext<'a, X, Y> {
    pub fn new(
        model: Arc<RwLock<AxesModel<X, Y>>>,
        window: &'a mut Window,
        cx: &'a mut App,
    ) -> Self {
        let model1 = model.read();
        Self {
            axes_bounds: model1.axes_bounds,
            pixel_bounds: model1.pixel_bounds,
            model: {
                drop(model1);
                model
            },
            cx: Some((window, cx)),
            new_axes_bounds: None,
        }
    }
    pub fn new_without_context(model: Arc<RwLock<AxesModel<X, Y>>>) -> Self {
        let model1 = model.read();
        Self {
            axes_bounds: model1.axes_bounds,
            pixel_bounds: model1.pixel_bounds,
            model: {
                drop(model1);
                model
            },
            cx: None,
            new_axes_bounds: None,
        }
    }
    pub fn transform_point(&self, point: Point2<X, Y>) -> Point<Pixels> {
        self.axes_bounds.transform_point(self.pixel_bounds, point)
    }
    pub fn plot(&mut self, mut element: impl GeometryAxes<X = X, Y = Y> + 'static) {
        element.render_axes(self);
    }
}
