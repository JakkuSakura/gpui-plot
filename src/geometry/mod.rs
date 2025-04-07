//! Useful geometric structures and functions used inside canvas

use gpui::{App, Bounds, Pixels, Window};

mod axis;
mod line;
mod point;
mod size;
mod text;

use crate::figure::axes::AxesContext;
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
