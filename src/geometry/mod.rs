//! Useful geometric structures and functions used inside canvas

use gpui::{App, Bounds, Pixels, Window};
use std::marker::PhantomData;

mod axis;
mod line;
mod point;
mod size;
mod text;

use crate::figure::axes::AxesContext;
use crate::figure::SharedModel;
pub use axis::*;
pub use line::*;
pub use point::*;
pub use size::*;
pub use text::*;

/// Low-level Geometry
pub trait GeometryPixels {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App);
}

/// High-level Geometry
pub trait GeometryAxes: Send + Sync {
    type X: AxisType;
    type Y: AxisType;
    fn get_x_range(&self) -> Option<AxisRange<Self::X>> {
        None
    }
    fn get_y_range(&self) -> Option<AxisRange<Self::Y>> {
        None
    }
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>);
}
impl<T: GeometryAxes> GeometryAxes for SharedModel<T> {
    type X = T::X;
    type Y = T::Y;
    fn get_x_range(&self) -> Option<AxisRange<Self::X>> {
        self.read().get_x_range()
    }
    fn get_y_range(&self) -> Option<AxisRange<Self::Y>> {
        self.read().get_y_range()
    }
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        self.write().render_axes(cx);
    }
}
pub struct GeometryAxesFn<X: AxisType, Y: AxisType, F: FnMut(&mut AxesContext<X, Y>) + Send + Sync>
{
    f: F,
    x_range: Option<AxisRange<X>>,
    y_range: Option<AxisRange<Y>>,
    _phantom: PhantomData<(X, Y)>,
}
impl<X: AxisType, Y: AxisType, F: FnMut(&mut AxesContext<X, Y>) + Send + Sync>
    GeometryAxesFn<X, Y, F>
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            x_range: None,
            y_range: None,
            _phantom: PhantomData,
        }
    }
    pub fn with_x_range(mut self, x_range: AxisRange<X>) -> Self {
        self.x_range = Some(x_range);
        self
    }
    pub fn with_y_range(mut self, y_range: AxisRange<Y>) -> Self {
        self.y_range = Some(y_range);
        self
    }
}
impl<X: AxisType, Y: AxisType, F: FnMut(&mut AxesContext<X, Y>) + Send + Sync> GeometryAxes
    for GeometryAxesFn<X, Y, F>
{
    type X = X;
    type Y = Y;
    fn get_x_range(&self) -> Option<AxisRange<Self::X>> {
        self.x_range
    }
    fn get_y_range(&self) -> Option<AxisRange<Self::Y>> {
        self.y_range
    }
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        (self.f)(cx);
    }
}
