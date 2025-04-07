use crate::figure::axes::AxesContext;
use crate::geometry::{point2, size2, AxisType, GeometryAxes, Line, Size2};

pub enum GridType<X: AxisType, Y: AxisType> {
    Density(Size2<X::Delta, Y::Delta>),
    Numbers(usize, usize),
}

pub struct GridModel<X: AxisType, Y: AxisType> {
    pub ty: GridType<X, Y>,
    pub movable: bool,
    pub grid_x_lines: Vec<X>,
    pub grid_y_lines: Vec<Y>,
}
impl<X: AxisType, Y: AxisType> GridModel<X, Y> {
    pub fn from_density(x: X::Delta, y: Y::Delta) -> Self {
        Self::new(GridType::Density(size2(x, y)))
    }
    pub fn from_numbers(x: usize, y: usize) -> Self {
        Self::new(GridType::Numbers(x, y))
    }
    pub fn new(ty: GridType<X, Y>) -> Self {
        Self {
            ty,
            movable: true,
            grid_x_lines: Vec::new(),
            grid_y_lines: Vec::new(),
        }
    }
    pub fn with_fixed(mut self) -> Self {
        self.movable = false;
        self
    }
    fn should_update_grid(&self, _axes_bounds: &AxesContext<X, Y>) -> bool {
        if self.movable {
            return self.grid_x_lines.is_empty() || self.grid_y_lines.is_empty();
        }
        true
    }
    pub fn update_grid(&mut self, axes_bounds: &AxesContext<X, Y>) {
        if !self.should_update_grid(axes_bounds) {
            return;
        }
        let density = match self.ty {
            GridType::Density(density) => density,
            GridType::Numbers(x, y) => Size2 {
                width: X::delta_from_f32(
                    X::delta_to_f32(axes_bounds.axes_bounds.x.difference()) / x as f32,
                ),
                height: Y::delta_from_f32(
                    Y::delta_to_f32(axes_bounds.axes_bounds.y.difference()) / y as f32,
                ),
            },
        };
        self.update_grid_by_density(axes_bounds, density);
    }
    fn update_grid_by_density(
        &mut self,
        axes_bounds: &AxesContext<X, Y>,
        density: Size2<X::Delta, Y::Delta>,
    ) {
        // TODO: clap beforehand to have better performance
        self.grid_x_lines = axes_bounds
            .axes_bounds
            .x
            .iter_step_by(density.width)
            .collect();
        self.grid_x_lines
            .retain(|x| axes_bounds.axes_bounds.x.contains(*x));
        self.grid_y_lines = axes_bounds
            .axes_bounds
            .y
            .iter_step_by(density.height)
            .collect();
        self.grid_y_lines
            .retain(|y| axes_bounds.axes_bounds.y.contains(*y));
    }
}

pub struct GridView<'a, X: AxisType, Y: AxisType> {
    model: &'a GridModel<X, Y>,
}
impl<'a, X: AxisType, Y: AxisType> GridView<'a, X, Y> {
    pub fn new(model: &'a GridModel<X, Y>) -> Self {
        Self { model }
    }
}
impl<'a, X: AxisType, Y: AxisType> GeometryAxes for GridView<'a, X, Y> {
    type X = X;
    type Y = Y;
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        let grid = self.model;
        for x in grid.grid_x_lines.iter().cloned() {
            let top_point = point2(x, cx.axes_bounds.y.min);
            let bottom_point = point2(x, cx.axes_bounds.y.max);
            let mut line = Line::between_points(top_point, bottom_point);
            line.render_axes(cx);
        }

        for y in grid.grid_y_lines.iter().cloned() {
            let left_point = point2(cx.axes_bounds.x.min, y);
            let right_point = point2(cx.axes_bounds.x.max, y);
            let mut line = Line::between_points(left_point, right_point);
            line.render_axes(cx);
        }
    }
}
