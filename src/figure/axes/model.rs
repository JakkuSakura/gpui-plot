use crate::figure::axes::{Axes, AxesContext, AxesView};
use crate::figure::grid::GridModel;
use crate::geometry::{
    AxesBounds, AxesBoundsPixels, AxisType, GeometryAxes, GeometryAxesFn, GeometryPixels, Point2,
};
use gpui::{size, App, Bounds, MouseMoveEvent, Pixels, Point, Window};
use std::fmt::Debug;

pub(crate) struct PanState<X: AxisType, Y: AxisType> {
    initial_axes_bounds: AxesBounds<X, Y>,
    initial_pan_position: Point<Pixels>,
}
pub(crate) struct ZoomState<X: AxisType, Y: AxisType> {
    initial_axes_bounds: AxesBounds<X, Y>,
    pixel_bounds: AxesBoundsPixels,
    initial_zoom_position: Point<Pixels>,
    zoom_point: Point<f64>,
    accumulated_zoom_delta: f64,
}

pub enum ViewUpdateType {
    /// Freely movable
    Free,
    /// Fixed to the current state
    Fixed,
    /// Automatically updated
    Auto,
}

pub struct AxesModel<X: AxisType, Y: AxisType> {
    pub axes_bounds: AxesBounds<X, Y>,
    pub pixel_bounds: AxesBoundsPixels,
    pub grid: GridModel<X, Y>,
    pub(crate) pan_state: Option<PanState<X, Y>>,
    pub(crate) zoom_state: Option<ZoomState<X, Y>>,
    pub(crate) event_processed: bool,
    pub(crate) elements: Vec<Box<dyn GeometryAxes<X = X, Y = Y>>>,
    pub update_type: ViewUpdateType,
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
            zoom_state: None,
            event_processed: false,
            elements: Vec::new(),
            update_type: ViewUpdateType::Free,
        }
    }
    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }
    pub fn add_element(&mut self, element: Box<dyn GeometryAxes<X = X, Y = Y>>) {
        self.elements.push(element);
    }
    pub fn plot(&mut self, element: impl GeometryAxes<X = X, Y = Y> + 'static) {
        self.elements.push(Box::new(element));
    }
    pub fn plot_fn(&mut self, element: impl FnMut(&mut AxesContext<X, Y>) + Send + Sync + 'static) {
        self.elements.push(Box::new(GeometryAxesFn::new(element)));
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
    pub fn update(&mut self) {
        self.update_type = ViewUpdateType::Auto;
        // update the axes bounds
        let mut new_axes_bounds = None;
        for element in self.elements.iter_mut() {
            let Some(x) = element.get_x_range() else {
                continue;
            };
            let Some(y) = element.get_y_range() else {
                continue;
            };
            match new_axes_bounds {
                None => {
                    new_axes_bounds = Some(AxesBounds::new(x, y));
                }
                Some(ref mut bounds) => {
                    if let Some(x_union) = bounds.x.union(&x) {
                        bounds.x = x_union;
                    }
                    if let Some(y_union) = bounds.y.union(&y) {
                        bounds.y = y_union;
                    }
                }
            }
        }

        let Some(new_pixel_bounds) = new_axes_bounds else {
            return;
        };
        self.axes_bounds = new_pixel_bounds;

        let cx1 = AxesContext::new_without_context(self);
        self.grid.update_grid(&cx1);
    }
}

impl<X: AxisType, Y: AxisType> Axes for AxesModel<X, Y> {
    fn update(&mut self) {
        self.update()
    }

    fn new_render(&mut self) {
        self.event_processed = false;

        let cx1 = AxesContext::new_without_context(self);
        self.grid.try_update_grid(&cx1)
    }
    fn pan_begin(&mut self, position: Point<Pixels>) {
        if matches!(self.update_type, ViewUpdateType::Fixed) {
            return;
        }
        if self.event_processed {
            return;
        }
        self.pan_state = Some(PanState {
            initial_axes_bounds: self.axes_bounds,
            initial_pan_position: position,
        });
    }

    fn pan(&mut self, event: &MouseMoveEvent) {
        if self.event_processed {
            return;
        }
        let Some(pan_state) = &self.pan_state else {
            return;
        };
        let delta_pixels = event.position - pan_state.initial_pan_position;
        let delta_elements = size(
            self.axes_bounds
                .x
                .elements_per_pixels(-delta_pixels.x, self.pixel_bounds.x),
            self.axes_bounds
                .y
                .elements_per_pixels(delta_pixels.y, self.pixel_bounds.y),
        );
        self.axes_bounds = pan_state.initial_axes_bounds + delta_elements;

        let cx1 = AxesContext::new_without_context(self);
        self.grid.try_update_grid(&cx1);
    }

    fn pan_end(&mut self) {
        if self.event_processed {
            return;
        }
        self.pan_state = None;
    }
    fn zoom_begin(&mut self, position: Point<Pixels>) {
        if self.event_processed {
            return;
        }
        self.zoom_state = Some(ZoomState {
            initial_axes_bounds: self.axes_bounds,
            pixel_bounds: self.pixel_bounds,
            initial_zoom_position: position,
            accumulated_zoom_delta: 0.0,
            zoom_point: self
                .axes_bounds
                .transform_point_reverse_f64(self.pixel_bounds, position),
        });
    }
    fn zoom(&mut self, zoom_in: f64) {
        if self.event_processed {
            return;
        }
        let Some(zoom_state) = &mut self.zoom_state else {
            return;
        };
        let zoom_point = zoom_state.zoom_point;
        zoom_state.accumulated_zoom_delta += zoom_in;
        let zoom_factor = zoom_state.accumulated_zoom_delta.exp();

        self.axes_bounds.x.min_to_base =
            (zoom_state.initial_axes_bounds.x.min_to_base - zoom_point.x) * zoom_factor
                + zoom_point.x;
        self.axes_bounds.x.max_to_base =
            (zoom_state.initial_axes_bounds.x.max_to_base - zoom_point.x) * zoom_factor
                + zoom_point.x;
        self.axes_bounds.y.min_to_base =
            (zoom_state.initial_axes_bounds.y.min_to_base - zoom_point.y) * zoom_factor
                + zoom_point.y;
        self.axes_bounds.y.max_to_base =
            (zoom_state.initial_axes_bounds.y.max_to_base - zoom_point.y) * zoom_factor
                + zoom_point.y;
        self.pixel_bounds.x.pixels_per_element =
            zoom_state.pixel_bounds.x.pixels_per_element / zoom_factor;
        self.pixel_bounds.y.pixels_per_element =
            zoom_state.pixel_bounds.y.pixels_per_element / zoom_factor;
        let afterwards_zoom_point = self
            .axes_bounds
            .transform_point_reverse_f64(self.pixel_bounds, zoom_state.initial_zoom_position);
        let diff = zoom_point - afterwards_zoom_point;
        self.axes_bounds.x.min_to_base += diff.x;
        self.axes_bounds.x.max_to_base += diff.x;
        self.axes_bounds.y.min_to_base += diff.y;
        self.axes_bounds.y.max_to_base += diff.y;
        // let adjusted_zoom_point = self
        //     .axes_bounds
        //     .transform_point_reverse_f64(self.pixel_bounds, zoom_state.initial_zoom_position);
        // assert_eq!(self.pixel_bounds.x, adjusted_zoom_point.x);
        // assert_eq!(self.pixel_bounds.y, adjusted_zoom_point.y);

        let cx1 = AxesContext::new_without_context(self);
        self.grid.try_update_grid(&cx1);
    }

    fn zoom_end(&mut self) {
        if self.event_processed {
            return;
        }
        self.zoom_state = None;
    }

    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        AxesView::new(self).render_pixels(bounds, window, cx);
    }
}
