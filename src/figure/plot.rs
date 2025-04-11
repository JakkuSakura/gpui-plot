use crate::figure::axes::{Axes, AxesContext, AxesModel, PlottersModel};
use crate::figure::SharedModel;
use crate::fps::FpsModel;
use crate::geometry::AxisType;
use gpui::{
    canvas, div, Bounds, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Pixels, Point, Render, ScrollDelta, ScrollWheelEvent, Styled,
    Window,
};
use parking_lot::RwLock;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters_gpui::backend::GpuiBackend;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct PlotModel {
    pub panning: bool,
    pub zooming: bool,
    pub zoom_pinch_precision: f64,
    pub zoom_scroll_precision: f64,
    pub fps: FpsModel,
    pub bounds: Bounds<Pixels>,
    pub axes: Vec<Box<dyn Axes>>,
}
impl Debug for PlotModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlotModel")
            .field("panning", &self.panning)
            .field("zooming", &self.zooming)
            .field("zoom_pinch_precision", &self.zoom_pinch_precision)
            .field("zoom_scroll_precision", &self.zoom_scroll_precision)
            .field("bounds", &self.bounds)
            .field("axes", &self.axes.len())
            .finish()
    }
}
impl Default for PlotModel {
    fn default() -> Self {
        Self::new()
    }
}
impl PlotModel {
    pub fn new() -> Self {
        Self {
            panning: false,
            zooming: false,
            zoom_pinch_precision: 1.0 / 200.0,
            zoom_scroll_precision: 1.0 / 100.0,
            fps: FpsModel::new(),
            bounds: Bounds::default(),
            axes: Vec::new(),
        }
    }
    pub fn clear_axes(&mut self) {
        self.axes.clear();
    }

    pub fn add_axes<X: AxisType, Y: AxisType>(
        &mut self,
        model: SharedModel<AxesModel<X, Y>>,
    ) -> &SharedModel<AxesModel<X, Y>> {
        self.axes.push(Box::new(model));
        let any = self.axes.last_mut().unwrap();

        unsafe {
            let axes_ptr = any.as_mut() as *const dyn Axes;
            let erased_ptr = axes_ptr as *const SharedModel<AxesModel<X, Y>>;
            (&*(erased_ptr as *const SharedModel<AxesModel<X, Y>>)) as _
        }
    }
    #[cfg(feature = "plotters")]
    pub fn add_axes_plotters<X: AxisType, Y: AxisType>(
        &mut self,
        model: SharedModel<AxesModel<X, Y>>,
        draw: impl FnMut(&mut DrawingArea<GpuiBackend, Shift>, &mut AxesContext<X, Y>) + 'static,
    ) {
        let axes = PlottersModel::new(model, Box::new(draw));

        self.axes.push(Box::new(axes));
    }
    pub fn update(&mut self) {
        for axes in self.axes.iter_mut() {
            axes.update();
        }
    }
    pub fn pan_begin(&mut self, position: Point<Pixels>) {
        if self.panning {
            return;
        }
        self.panning = true;
        for axes in self.axes.iter_mut() {
            axes.pan_begin(position);
        }
    }
    pub fn pan(&mut self, event: &MouseMoveEvent) {
        if !self.panning {
            return;
        }
        self.panning = true;
        for axes in self.axes.iter_mut() {
            axes.pan(event);
        }
    }
    pub fn pan_end(&mut self) {
        if !self.panning {
            return;
        }
        self.panning = false;
        for axes in self.axes.iter_mut() {
            axes.pan_end();
        }
    }
    pub fn zoom_begin(&mut self, position: Point<Pixels>) {
        if self.zooming {
            return;
        }
        self.zooming = true;
        for axes in self.axes.iter_mut() {
            axes.zoom_begin(position);
        }
    }
    pub fn zoom(&mut self, zoom_in: f64) {
        if !self.zooming {
            return;
        }
        for axes in self.axes.iter_mut() {
            axes.zoom(zoom_in);
        }
    }
    pub fn zoom_end(&mut self) {
        if !self.zooming {
            return;
        }
        for axes in self.axes.iter_mut() {
            axes.zoom_end();
        }
    }
}

#[derive(Clone)]
pub struct PlotView {
    pub model: Arc<RwLock<PlotModel>>,
    pub last_zoom_ts: Option<Instant>,
    pub last_zoom_rb: Option<Point<Pixels>>,
}
impl PlotView {
    pub fn new(model: Arc<RwLock<PlotModel>>) -> Self {
        Self {
            model,
            last_zoom_ts: None,
            last_zoom_rb: None,
        }
    }

    fn try_clean_zoom(&mut self) {
        if let Some(last_time) = self.last_zoom_ts {
            if last_time.elapsed() > Duration::from_secs_f32(0.2) {
                self.model.write().zoom_end();
                self.last_zoom_ts = None;
            }
        }
    }

    pub fn zoom(
        &mut self,
        zoom_point: Point<Pixels>,
        zoom_in: f64,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.try_clean_zoom();
        let mut model = self.model.write();
        if self.last_zoom_ts.is_none() {
            model.zoom_begin(zoom_point);
        }
        self.last_zoom_ts = Some(Instant::now());
        model.zoom(zoom_in);
        cx.notify();
    }
    pub fn zoom_rubberband(
        &mut self,
        zoom_point: Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(last_zoom_point) = self.last_zoom_rb else {
            return;
        };
        let delta = zoom_point.y - last_zoom_point.y;
        self.model.write().zoom(delta.0 as f64);
        cx.notify()
    }
}
impl Render for PlotView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.try_clean_zoom();
        for axes in self.model.write().axes.iter_mut() {
            axes.new_render();
        }

        div()
            .size_full()
            .child(
                canvas(|_, _window, _cx| (), {
                    let this = self.clone();
                    move |bounds, _ele: (), window, cx| {
                        this.model.write().bounds = bounds;
                        for axes in this.model.write().axes.iter_mut() {
                            axes.render(bounds, window, cx);
                        }
                    }
                })
                .size_full(),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _window, _cx| {
                    let mut model = this.model.write();
                    model.pan_begin(ev.position);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, ev: &MouseDownEvent, _window, _cx| {
                    this.try_clean_zoom();
                    this.last_zoom_rb = Some(ev.position);
                    this.model.write().zoom_begin(ev.position);
                }),
            )
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, cx| {
                // println!("Mouse move event captured: {:?}", ev);
                match ev.pressed_button {
                    Some(MouseButton::Left) => {
                        let mut model = this.model.write();
                        model.pan(ev);
                        cx.notify();
                    }
                    // it won't work on MacOS
                    Some(MouseButton::Right) => {
                        this.zoom_rubberband(ev.position, window, cx);
                    }
                    _ => {}
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _ev, _window, _cx| {
                    let mut model = this.model.write();
                    model.pan_end();
                }),
            )
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(|this, _ev, _window, cx| {
                    this.last_zoom_rb = None;
                    this.model.write().zoom_end();
                    cx.notify();
                }),
            )
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, window, cx| {
                let model = this.model.read();
                let zoom_in = match ev.delta {
                    ScrollDelta::Pixels(p) => {
                        // println!("Scroll event captured: {:?}", p);
                        // Swipe swipe down to zoom in. This is aligned with Google Maps and some tools like Mac Mouse Fix or Scroll Inverter
                        -p.y.0 as f64 * model.zoom_pinch_precision
                    }
                    ScrollDelta::Lines(l) => {
                        // println!("Scroll event in lines {:?}, ignoring.",&q);
                        // Scroll up to zoom in
                        l.y as f64 * model.zoom_scroll_precision
                    }
                };
                drop(model);

                this.zoom(ev.position, zoom_in, window, cx);
            }))
    }
}
