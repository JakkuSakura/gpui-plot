use crate::figure::axes::{Axes, AxesContext, AxesView};
use crate::figure::grid::GridModel;
use crate::geometry::{
    AxesBounds, AxesBoundsPixels, AxisRange, AxisType, GeometryAxes, GeometryAxesFn,
    GeometryPixels, Point2, Size2,
};
use gpui::{App, Bounds, MouseMoveEvent, Pixels, Point, Window};
use std::fmt::Debug;

pub(crate) struct PanState<X: AxisType, Y: AxisType> {
    initial_axes_bounds: AxesBounds<X, Y>,
    initial_pan_position: Point<Pixels>,
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
    pub fn pan_begin(&mut self, position: Point<Pixels>) {
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
    pub fn pan(&mut self, event: &MouseMoveEvent) {
        if self.event_processed {
            return;
        }
        let Some(pan_state) = &self.pan_state else {
            return;
        };
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

        let cx1 = AxesContext::new_without_context(self);
        self.grid.try_update_grid(&cx1);
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

        let cx1 = AxesContext::new_without_context(self);
        self.grid.try_update_grid(&cx1);
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
    }
    fn pan_begin(&mut self, position: Point<Pixels>) {
        self.pan_begin(position);
    }

    fn pan(&mut self, event: &MouseMoveEvent) {
        self.pan(event);
    }

    fn pan_end(&mut self) {
        self.pan_end();
    }
    fn zoom(&mut self, point: Point<Pixels>, delta: f32) {
        self.zoom(point, delta);
    }
    fn render(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        AxesView::new(self).render_pixels(bounds, window, cx);
    }
}
