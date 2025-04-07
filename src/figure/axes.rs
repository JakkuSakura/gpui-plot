use crate::figure::grid::{GridModel, GridViewer};
use crate::figure::ticks::TicksViewer;
use crate::geometry::{
    AxesBounds, AxesBoundsPixels, AxisRange, AxisType, GeometryAxes, GeometryPixels, Line, Point2,
    Size2,
};
use gpui::{px, App, Bounds, Edges, MouseMoveEvent, Pixels, Point, Window};
use parking_lot::RwLock;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

pub const CONTENT_BOARDER: Pixels = px(30.0);

pub(crate) struct PanState<X: AxisType, Y: AxisType> {
    initial_axes_bounds: AxesBounds<X, Y>,
    initial_pan_position: Point<Pixels>,
}

pub struct AxesModel<X: AxisType, Y: AxisType> {
    pub axes_bounds: AxesBounds<X, Y>,
    pub pixel_bounds: AxesBoundsPixels,
    pub grid: GridModel<X, Y>,
    pub(crate) pan_state: Option<PanState<X, Y>>,
    pub(crate) event_processed: bool,
    pub(crate) elements: Vec<RwLock<Box<dyn GeometryAxes<X = X, Y = Y>>>>,
}
impl<X: AxisType, Y: AxisType> Debug for AxesModel<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AxesModel")
            .field("axes_bounds", &self.axes_bounds)
            .finish()
    }
}

impl<X: AxisType, Y: AxisType> AxesModel<X, Y> {
    pub fn new(axes_bounds: AxesBounds<X, Y>, grid: GridModel<X, Y>) -> Self {
        Self {
            axes_bounds,
            pixel_bounds: AxesBoundsPixels::from_bounds(Bounds::default()),
            grid,
            pan_state: None,
            event_processed: false,
            elements: Vec::new(),
        }
    }
    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }
    pub fn add_element(&mut self, element: Box<dyn GeometryAxes<X = X, Y = Y>>) {
        self.elements.push(RwLock::new(element));
    }
    pub fn plot(&mut self, element: impl GeometryAxes<X = X, Y = Y> + 'static) {
        self.elements.push(RwLock::new(Box::new(element)));
    }
    pub fn pan_begin(&mut self, position: Point<Pixels>) {
        if self.event_processed {
            return;
        }
        self.pan_state = Some(PanState {
            initial_axes_bounds: self.axes_bounds,
            initial_pan_position: position,
        });
    }
    pub fn pan(&mut self, event: &MouseMoveEvent) {
        if self.event_processed {
            return;
        }
        let pan_state = self.pan_state.as_mut().unwrap();
        let delta_pixels = event.position - pan_state.initial_pan_position;
        let delta_elements = Size2 {
            width: self
                .axes_bounds
                .x
                .elements_per_pixels(-delta_pixels.x, self.pixel_bounds.x),
            height: self
                .axes_bounds
                .y
                .elements_per_pixels(delta_pixels.y, self.pixel_bounds.y),
        };
        self.axes_bounds = pan_state.initial_axes_bounds + delta_elements;
    }
    pub fn pan_end(&mut self) {
        if self.event_processed {
            return;
        }
        self.pan_state = None;
    }

    pub fn zoom(&mut self, point: Point<Pixels>, delta: f32) {
        if self.event_processed {
            return;
        }
        let zoom_point = self.transform_point_reverse(point);
        let zoom_factor = 1.0 + delta * 1.0;

        // now we have (min, p) and (p, end)
        // we keep p to zero and scale the rest (min - p, 0) and (0, end - p)
        let min_point = self.axes_bounds.min_point();
        let max_point = self.axes_bounds.max_point();
        let min_shifted = min_point.sub_to_f32(zoom_point);
        let max_shifted = max_point.sub_to_f32(zoom_point);
        let min_zoomed = min_shifted * zoom_factor;
        let max_zoomed = max_shifted * zoom_factor;

        // map back to the view
        let min = zoom_point.add_from_f32(min_zoomed);
        let max = zoom_point.add_from_f32(max_zoomed);

        let Some(x_range) = AxisRange::new(min.x, max.x) else {
            eprintln!("Invalid zoom: min: {:?}, max: {:?}", min, max);
            return;
        };
        let Some(y_range) = AxisRange::new(min.y, max.y) else {
            eprintln!("Invalid zoom: min: {:?}, max: {:?}", min, max);
            return;
        };
        self.axes_bounds.x = x_range;
        self.axes_bounds.y = y_range;
    }
    pub fn update_scale(&mut self, shrunk_bounds: Bounds<Pixels>) {
        self.pixel_bounds = AxesBoundsPixels::from_bounds(shrunk_bounds);
        self.pixel_bounds.x.pixels_per_element =
            self.axes_bounds.x.pixels_per_element(self.pixel_bounds.x);
        self.pixel_bounds.y.pixels_per_element =
            -self.axes_bounds.y.pixels_per_element(self.pixel_bounds.y);
    }
    pub fn transform_point(&self, point: Point2<X, Y>) -> Point<Pixels> {
        self.axes_bounds.transform_point(self.pixel_bounds, point)
    }
    pub fn transform_point_reverse(&self, point: Point<Pixels>) -> Point2<X, Y> {
        self.axes_bounds
            .transform_point_reverse(self.pixel_bounds, point)
    }
}

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
pub type SharedModel<T> = Arc<RwLock<T>>;

pub trait DynAxesModel {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn new_render(&mut self);
}
impl<X: AxisType, Y: AxisType> DynAxesModel for SharedModel<AxesModel<X, Y>> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn new_render(&mut self) {
        self.write().event_processed = false;
    }
}

pub trait Axes: GeometryPixels {
    fn get_model(&self) -> &dyn DynAxesModel;
    fn get_model_mut(&mut self) -> &mut dyn DynAxesModel;
    fn pan_begin(&mut self, position: Point<Pixels>);
    fn pan(&mut self, event: &MouseMoveEvent);
    fn pan_end(&mut self);
    fn zoom(&mut self, point: Point<Pixels>, delta: f32);
}

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
