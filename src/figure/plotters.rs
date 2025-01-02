use crate::figure::axes::{Axes, AxesContext, AxesModel, DynAxesModel};
use crate::geometry::{AxisType, GeometryPixels};
use gpui::{px, Bounds, Edges, MouseMoveEvent, Pixels, Point, WindowContext};
use parking_lot::RwLock;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_gpui::backend::GpuiBackend;
use std::sync::Arc;
use tracing::error;

pub const CONTENT_BOARDER: Pixels = px(30.0);

pub trait GeometryAxesPlotters {
    type X: AxisType;
    type Y: AxisType;
    fn render_axes(
        &mut self,
        area: &mut DrawingArea<GpuiBackend, Shift>,
        cx: &mut AxesContext<Self::X, Self::Y>,
    );
}
pub(crate) struct PlottersFunc<X, Y, F> {
    func: F,
    phantom: std::marker::PhantomData<(X, Y, F)>,
}
impl<X, Y, F> PlottersFunc<X, Y, F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            phantom: std::marker::PhantomData,
        }
    }
}
impl<
        X: AxisType,
        Y: AxisType,
        F: FnMut(&mut DrawingArea<GpuiBackend, Shift>, &mut AxesContext<X, Y>) + 'static,
    > GeometryAxesPlotters for PlottersFunc<X, Y, F>
{
    type X = X;
    type Y = Y;

    fn render_axes(
        &mut self,
        area: &mut DrawingArea<GpuiBackend, Shift>,
        cx: &mut AxesContext<Self::X, Self::Y>,
    ) {
        (self.func)(area, cx);
    }
}
pub struct PlottersAxes<X: AxisType, Y: AxisType> {
    model: Arc<RwLock<AxesModel<X, Y>>>,
    draw: Box<dyn GeometryAxesPlotters<X = X, Y = Y>>,
}
impl<X: AxisType, Y: AxisType> PlottersAxes<X, Y> {
    pub fn new(
        model: Arc<RwLock<AxesModel<X, Y>>>,
        draw: Box<dyn GeometryAxesPlotters<X = X, Y = Y>>,
    ) -> Self {
        Self {
            model: model.clone(),
            draw,
        }
    }
    pub fn plot(
        &mut self,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
    ) -> Result<(), DrawingAreaErrorKind<plotters_gpui::Error>> {
        let mut root = GpuiBackend::new(bounds, cx).into_drawing_area();
        let cx1 = &mut AxesContext::new_without_context(self.model.clone());
        self.draw.render_axes(&mut root, cx1);
        root.present()?;
        Ok(())
    }
}
impl<X: AxisType, Y: AxisType> Axes for PlottersAxes<X, Y> {
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
impl<X: AxisType, Y: AxisType> GeometryPixels for PlottersAxes<X, Y> {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
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
        self.model.write().update_scale(shrunk_bounds);
        if let Err(err) = self.plot(bounds, cx) {
            error!("failed to plot: {}", err);
        }
    }
}
