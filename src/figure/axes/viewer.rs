use crate::figure::axes::model::{AxesModel, DynAxesModel};
use crate::figure::axes::{Axes, AxesContext};
use crate::figure::grid::GridViewer;
use crate::figure::ticks::TicksViewer;
use crate::geometry::{AxisType, GeometryAxes, GeometryPixels, Line};
use gpui::{px, App, Bounds, Edges, MouseMoveEvent, Pixels, Point, Window};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct AxesViewer<X: AxisType, Y: AxisType> {
    pub model: Arc<RwLock<AxesModel<X, Y>>>,
}
impl<X: AxisType, Y: AxisType> AxesViewer<X, Y> {
    pub fn new(model: Arc<RwLock<AxesModel<X, Y>>>) -> Self {
        Self { model }
    }
    pub fn paint(&mut self, window: &mut Window, cx: &mut App) {
        {
            let model = self.model.read();
            let shrunk_bounds = model.pixel_bounds.into_bounds();
            for (x, y) in [
                (shrunk_bounds.origin, shrunk_bounds.top_right()),
                (shrunk_bounds.top_right(), shrunk_bounds.bottom_right()),
                (shrunk_bounds.bottom_right(), shrunk_bounds.bottom_left()),
                (shrunk_bounds.bottom_left(), shrunk_bounds.origin),
            ] {
                Line::between_points(x.into(), y.into()).render(window, cx);
            }
        }

        let cx1 = &mut AxesContext::new(self.model.clone(), window, cx);
        let should_update = self.model.read().grid.should_update_grid(cx1);
        // kept aside to avoid deadlock
        if should_update {
            self.model.write().grid.update_grid(cx1);
        }

        let mut ticks = TicksViewer::new(self.model.clone());
        {
            let (window, cx1) = cx1.cx.as_mut().unwrap();
            ticks.render(window, cx1);
        }
        let mut grid = GridViewer::new(self.model.clone());
        {
            grid.render_axes(cx1);
        }

        for element in self.model.read().elements.iter() {
            element.write().render_axes(cx1);
        }
        if let Some(new_axes_bounds) = cx1.new_axes_bounds.take() {
            self.model.write().axes_bounds = new_axes_bounds;
        }
    }
}

const CONTENT_BOARDER: Pixels = px(30.0);

impl<X: AxisType, Y: AxisType> GeometryPixels for AxesViewer<X, Y> {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let shrunk_bounds = bounds.extend(Edges {
            top: px(-0.0),
            right: -CONTENT_BOARDER,
            bottom: -CONTENT_BOARDER,
            left: -CONTENT_BOARDER,
        });
        self.model.write().update_scale(shrunk_bounds);
        self.paint(window, cx);
    }
}

impl<X: AxisType, Y: AxisType> Axes for AxesViewer<X, Y> {
    fn update(&mut self) {
        self.model.write().update()
    }
    fn get_model(&self) -> &dyn DynAxesModel {
        &self.model
    }

    fn get_model_mut(&mut self) -> &mut dyn DynAxesModel {
        &mut self.model
    }

    fn pan_begin(&mut self, position: Point<Pixels>) {
        self.model.write().pan_begin(position);
    }

    fn pan(&mut self, event: &MouseMoveEvent) {
        self.model.write().pan(event);
    }

    fn pan_end(&mut self) {
        self.model.write().pan_end();
    }
    fn zoom(&mut self, point: Point<Pixels>, delta: f32) {
        self.model.write().zoom(point, delta);
    }
}
