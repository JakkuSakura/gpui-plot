use crate::figure::axes::{AxesContext, AxesModel};
use crate::geometry::{point2, AxisType, GeometryAxes, Line, Size2};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct GridModel<X: AxisType, Y: AxisType> {
    pub grid_density: Size2<X::Delta, Y::Delta>,
    // TODO: move to TicksModel?
    pub grid_x_lines: Vec<X>,
    pub grid_y_lines: Vec<Y>,
}
impl<X: AxisType, Y: AxisType> GridModel<X, Y> {
    pub fn new(grid_density: Size2<X::Delta, Y::Delta>) -> Self {
        Self {
            grid_density,
            grid_x_lines: Vec::new(),
            grid_y_lines: Vec::new(),
        }
    }
    pub fn should_update_grid(&self, _axes_bounds: &mut AxesContext<X, Y>) -> bool {
        self.grid_x_lines.is_empty() || self.grid_y_lines.is_empty()
    }
    pub fn update_grid(&mut self, axes_bounds: &mut AxesContext<X, Y>) {
        self.grid_x_lines = axes_bounds
            .axes_bounds
            .x
            .iter_step_by(self.grid_density.width)
            .collect();
        self.grid_y_lines = axes_bounds
            .axes_bounds
            .y
            .iter_step_by(self.grid_density.height)
            .collect();
    }
}
#[derive(Clone, Debug)]
pub struct GridViewer<X: AxisType, Y: AxisType> {
    model: Arc<RwLock<AxesModel<X, Y>>>,
}
impl<X: AxisType, Y: AxisType> GridViewer<X, Y> {
    pub fn new(context: Arc<RwLock<AxesModel<X, Y>>>) -> Self {
        Self { model: context }
    }
}
impl<X: AxisType, Y: AxisType> GeometryAxes for GridViewer<X, Y> {
    type X = X;
    type Y = Y;
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        let model = self.model.read();
        for x in model.grid.grid_x_lines.iter().cloned() {
            let top_point = point2(x, cx.axes_bounds.y.min);
            let bottom_point = point2(x, cx.axes_bounds.y.max);
            let mut line = Line::between_points(top_point, bottom_point);
            line.render_axes(cx);
        }

        for y in model.grid.grid_y_lines.iter().cloned() {
            let left_point = point2(cx.axes_bounds.x.min, y);
            let right_point = point2(cx.axes_bounds.x.max, y);
            let mut line = Line::between_points(left_point, right_point);
            line.render_axes(cx);
        }
    }
}
