use crate::figure::grid::GridModel;
use crate::figure::SharedModel;
use crate::geometry::{
    AxesBounds, AxesBoundsPixels, AxisRange, AxisType, GeometryAxes, Point2, Size2,
};
use gpui::{Bounds, MouseMoveEvent, Pixels, Point};
use parking_lot::RwLock;
use std::any::Any;
use std::fmt::Debug;

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
    pub(crate) elements: Vec<RwLock<Box<dyn GeometryAxes<X = X, Y = Y>>>>,
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
        self.elements.push(RwLock::new(element));
    }
    pub fn plot(&mut self, element: impl GeometryAxes<X = X, Y = Y> + 'static) {
        self.elements.push(RwLock::new(Box::new(element)));
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
    pub fn update(&mut self) {
        self.update_type = ViewUpdateType::Auto;
        // update the axes bounds
        let mut new_axes_bounds = self.axes_bounds.clone();
        for element in self.elements.iter() {
            let element = element.read();
            let Some(x) = element.get_x_range() else {
                continue;
            };
            let Some(y) = element.get_y_range() else {
                continue;
            };
            if let Some(x_union) = new_axes_bounds.x.union(&x) {
                new_axes_bounds.x = x_union;
            }
            if let Some(y_union) = new_axes_bounds.y.union(&y) {
                new_axes_bounds.y = y_union;
            }
        }
        self.axes_bounds = new_axes_bounds;
    }
}
