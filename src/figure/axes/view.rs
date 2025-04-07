use crate::figure::axes::model::AxesModel;
use crate::figure::axes::AxesContext;
use crate::figure::grid::GridView;
use crate::figure::ticks::TicksView;
use crate::geometry::{AxisType, GeometryAxes, GeometryPixels, Line};
use gpui::{px, App, Bounds, Edges, Pixels, Window};

pub struct AxesView<'a, X: AxisType, Y: AxisType> {
    pub model: &'a mut AxesModel<X, Y>,
}
impl<'a, X: AxisType, Y: AxisType> AxesView<'a, X, Y> {
    pub fn new(model: &'a mut AxesModel<X, Y>) -> Self {
        Self { model }
    }
    pub fn paint(&mut self, window: &mut Window, cx: &mut App) {
        {
            let model = &self.model;
            let shrunk_bounds = model.pixel_bounds.into_bounds();
            for (x, y) in [
                (shrunk_bounds.origin, shrunk_bounds.top_right()),
                (shrunk_bounds.top_right(), shrunk_bounds.bottom_right()),
                (shrunk_bounds.bottom_right(), shrunk_bounds.bottom_left()),
                (shrunk_bounds.bottom_left(), shrunk_bounds.origin),
            ] {
                Line::between_points(x.into(), y.into()).render(window, cx, Some(shrunk_bounds));
            }
        }

        let cx1 = &mut AxesContext::new(self.model, window, cx);

        let mut ticks = TicksView::new(self.model);
        {
            let (window, cx1) = cx1.cx.as_mut().unwrap();
            ticks.render(window, cx1);
        }
        let mut grid = GridView::new(&self.model.grid);
        {
            grid.render_axes(cx1);
        }

        for element in self.model.elements.iter() {
            element.write().render_axes(cx1);
        }
    }
}

const CONTENT_BOARDER: Pixels = px(30.0);

impl<'a, X: AxisType, Y: AxisType> GeometryPixels for AxesView<'a, X, Y> {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let shrunk_bounds = bounds.extend(Edges {
            top: px(-0.0),
            right: -CONTENT_BOARDER,
            bottom: -CONTENT_BOARDER,
            left: -CONTENT_BOARDER,
        });
        self.model.update_scale(shrunk_bounds);
        self.paint(window, cx);
    }
}
