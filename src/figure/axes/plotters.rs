use crate::figure::axes::{Axes, AxesContext, AxesModel};
use crate::geometry::{AxisType, GeometryPixels};
use gpui::{px, App, Bounds, Edges, MouseMoveEvent, Pixels, Point, Window};
use parking_lot::RwLock;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_gpui::backend::GpuiBackend;
use std::sync::Arc;
use tracing::error;

const CONTENT_BOARDER: Pixels = px(30.0);
type ChartFn<X, Y> = Box<dyn FnMut(&mut DrawingArea<GpuiBackend, Shift>, &mut AxesContext<X, Y>)>;
pub struct PlottersModel<X: AxisType, Y: AxisType> {
    pub backend_color: RGBColor,
    pub chart: ChartFn<X, Y>,
    model: Arc<RwLock<AxesModel<X, Y>>>,
}
impl<X: AxisType, Y: AxisType> PlottersModel<X, Y> {
    pub fn new(model: Arc<RwLock<AxesModel<X, Y>>>, chart: ChartFn<X, Y>) -> Self {
        Self {
            backend_color: RGBColor(0, 0, 0),
            chart,
            model,
        }
    }
}
impl<X: AxisType, Y: AxisType> Axes for PlottersModel<X, Y> {
    fn update(&mut self) {
        self.model.write().update();
    }

    fn new_render(&mut self) {
        self.model.write().new_render();
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
    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        PlottersView::new(self).render_pixels(bounds, window, cx);
    }
}
pub struct PlottersView<'a, X: AxisType, Y: AxisType> {
    pub model: &'a mut PlottersModel<X, Y>,
}
impl<'a, X: AxisType, Y: AxisType> PlottersView<'a, X, Y> {
    pub fn new(model: &'a mut PlottersModel<X, Y>) -> Self {
        Self { model }
    }

    pub fn plot(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<(), DrawingAreaErrorKind<plotters_gpui::Error>> {
        let mut root = GpuiBackend::new(bounds, window, cx).into_drawing_area();
        let base_model = self.model.model.read();
        let cx1 = &mut AxesContext::new_without_context(&base_model);
        (self.model.chart)(&mut root, cx1);
        root.present()?;
        Ok(())
    }
}
impl<'a, X: AxisType, Y: AxisType> GeometryPixels for PlottersView<'a, X, Y> {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let bounds = bounds.extend(Edges {
            top: px(0.0),
            right: -CONTENT_BOARDER,
            bottom: px(0.0),
            left: px(0.0),
        });
        let shrunk_bounds = bounds.extend(Edges {
            top: px(0.0),
            right: px(0.0),
            bottom: -CONTENT_BOARDER,
            left: -CONTENT_BOARDER,
        });
        self.model.model.write().update_scale(shrunk_bounds);
        if let Err(err) = self.plot(bounds, window, cx) {
            error!("failed to plot: {}", err);
        }
    }
}
