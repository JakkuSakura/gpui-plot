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
    pub fps: FpsModel,
    pub bounds: Bounds<Pixels>,
    pub axes: Vec<Box<dyn Axes>>,
}
impl Debug for PlotModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlotModel")
            .field("panning", &self.panning)
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

    pub fn pan_begin(&mut self, position: Point<Pixels>) {
        for axes in self.model.write().axes.iter_mut() {
            axes.pan_begin(position);
        }
    }
    pub fn pan(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        for axes in self.model.write().axes.iter_mut() {
            axes.pan(event);
        }
        cx.notify();
    }
    pub fn pan_end(&mut self) {
        for axes in self.model.write().axes.iter_mut() {
            axes.pan_end();
        }
    }
    fn try_clean_zoom(&mut self) {
        if let Some(last_time) = self.last_zoom_ts {
            if last_time.elapsed() > Duration::from_secs_f32(0.2) {
                for axes in self.model.write().axes.iter_mut() {
                    axes.zoom_end();
                }
                self.last_zoom_ts = None;
            }
        }
    }

    pub fn zoom(
        &mut self,
        zoom_point: Point<Pixels>,
        delta: f64,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.try_clean_zoom();
        let mut model = self.model.write();
        if self.last_zoom_ts.is_none() {
            for axes in model.axes.iter_mut() {
                axes.zoom_begin(zoom_point);
            }
        }
        self.last_zoom_ts = Some(Instant::now());
        for axes in model.axes.iter_mut() {
            axes.zoom(delta);
        }
        cx.notify();
    }
    pub fn zoom_rubberband_begin(
        &mut self,
        zoom_point: Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.try_clean_zoom();
        self.last_zoom_rb = Some(zoom_point);
        for axes in self.model.write().axes.iter_mut() {
            axes.zoom_begin(zoom_point);
        }
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
        for axes in self.model.write().axes.iter_mut() {
            axes.zoom(delta.0 as f64);
        }
        cx.notify()
    }
    pub fn zoom_rubberband_end(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.last_zoom_rb = None;
        for axes in self.model.write().axes.iter_mut() {
            axes.zoom_end();
        }
        cx.notify();
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
                    if !model.panning {
                        model.panning = true;
                        drop(model);
                        this.pan_begin(ev.position);
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, ev: &MouseDownEvent, _window, _cx| {
                    if this.last_zoom_rb.is_none() {
                        this.zoom_rubberband_begin(ev.position, _window, _cx);
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, ev, window, cx| {
                let model = this.model.read();
                if model.panning {
                    drop(model);
                    this.pan(ev, window, cx);
                } else if this.last_zoom_rb.is_some() {
                    drop(model);
                    this.zoom_rubberband(ev.position, window, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _ev, _window, _cx| {
                    let mut model = this.model.write();

                    if model.panning {
                        model.panning = false;
                        drop(model);
                        this.pan_end();
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(|this, _ev, _window, _cx| {
                    this.zoom_rubberband_end(_window, _cx);
                }),
            )
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, window, cx| {
                let delta = match ev.delta {
                    ScrollDelta::Pixels(p) => {
                        // println!("Scroll event captured: {:?}", p);
                        // Swipe swipe down to zoom in. This is aligned with Google Maps and some tools like Mac Mouse Fix or Scroll Inverter
                        p.y.0 / 100.0
                    }
                    ScrollDelta::Lines(l) => {
                        // println!("Scroll event in lines {:?}, ignoring.",&q);
                        // Scroll up to zoom in
                        -l.y / 30.0
                    }
                };
                // println!(
                //     "Zooming at position: {:?} with delta: {}",
                //     ev.position, delta
                // );
                this.zoom(ev.position, delta as f64, window, cx);
            }))
    }
}
