use crate::figure::axes::{Axes, AxesContext, AxesModel, AxesViewer, SharedModel};
use crate::figure::plotters::{PlottersAxes, PlottersFunc};
use crate::fps::FpsModel;
use crate::geometry::{AxisType, Text};
use gpui::{
    canvas, div, px, App, Bounds, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, Point, Render, ScrollDelta,
    ScrollWheelEvent, Styled, Window,
};
use parking_lot::RwLock;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters_gpui::backend::GpuiBackend;
use std::fmt::Debug;
use std::sync::Arc;

pub struct PlotModel {
    pub panning: bool,
    pub zooming: bool,
    pub fps: FpsModel,
    pub bounds: Bounds<Pixels>,
    pub axes: Vec<Box<dyn Axes>>,
    pub axes_index: usize,
}
impl Debug for PlotModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlotModel")
            .field("panning", &self.panning)
            .field("zooming", &self.zooming)
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
            fps: FpsModel::new(),
            bounds: Bounds::default(),
            axes: Vec::new(),
            axes_index: 0,
        }
    }
    pub fn add_axes<X: AxisType, Y: AxisType>(
        &mut self,
        model: SharedModel<AxesModel<X, Y>>,
    ) -> &SharedModel<AxesModel<X, Y>> {
        let index = self.axes_index;
        self.axes_index += 1;
        let any = if index < self.axes.len() {
            self.axes[index].get_model()
        } else {
            let axes = AxesViewer::new(model);
            self.axes.push(Box::new(axes));
            self.axes.last_mut().unwrap().get_model()
        };
        let model = any
            .as_any()
            .downcast_ref::<SharedModel<AxesModel<X, Y>>>()
            .unwrap();
        model.write().elements.clear();
        model
    }
    #[cfg(feature = "plotters")]
    pub fn add_axes_plotters<X: AxisType, Y: AxisType>(
        &mut self,
        model: SharedModel<AxesModel<X, Y>>,
        draw: impl FnMut(&mut DrawingArea<GpuiBackend, Shift>, &mut AxesContext<X, Y>) + 'static,
    ) {
        let index = self.axes_index;
        self.axes_index += 1;
        let axes = PlottersAxes::new(model, Box::new(PlottersFunc::new(draw)));

        if index < self.axes.len() {
            self.axes[index] = Box::new(axes);
        } else {
            self.axes.push(Box::new(axes));
        }
    }
}

pub struct PlotViewer {
    pub model: Arc<RwLock<PlotModel>>,
}
impl PlotViewer {
    pub fn new(model: Arc<RwLock<PlotModel>>) -> Self {
        Self { model }
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
    pub fn zoom(
        &mut self,
        zoom_point: Point<Pixels>,
        delta: f32,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for axes in self.model.write().axes.iter_mut() {
            axes.zoom(zoom_point, delta);
        }
        cx.notify();
    }
}
impl Render for PlotViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.model.write().axes_index = 0;
        for axes in self.model.write().axes.iter_mut() {
            axes.get_model_mut().new_render();
        }

        div()
            .size_full()
            .child(
                canvas(|_, _window, _cx| (), {
                    let model = self.model.clone();
                    move |bounds, _ele: (), window, cx| {
                        model.write().bounds = bounds;
                        let mut plot_cx = PlotContext { model, window, cx };
                        plot_cx.render_fps();
                        plot_cx.render_axes(bounds);
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
            .on_mouse_move(cx.listener(|this, ev, window, cx| {
                let model = this.model.read();
                if model.panning {
                    drop(model);
                    this.pan(ev, window, cx);
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
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, window, cx| {
                let delta = match ev.delta {
                    ScrollDelta::Pixels(p) => {
                        println!("Scroll event captured: {:?}", p);
                        p.y.0 / 100.0
                    }
                    ScrollDelta::Lines(l) => {
                        // println!("Scroll event in lines {:?}, ignoring.",&q);
                        l.y / 10.0
                    }
                };
                println!(
                    "Zooming at position: {:?} with delta: {}",
                    ev.position, delta
                );
                this.zoom(ev.position, delta, window, cx);
            }))
    }
}

pub struct PlotContext<'a> {
    pub(crate) model: Arc<RwLock<PlotModel>>,
    pub(crate) window: &'a mut Window,
    pub(crate) cx: &'a mut App,
}
impl<'a> PlotContext<'a> {
    pub fn render_fps(&mut self) {
        let mut model = self.model.write();
        let fps = model.fps.next_fps();
        let text = format!("fps: {:.2}", fps);
        let mut origin: Point<Pixels> = model.bounds.origin;
        origin.y += px(3.0);
        Text {
            origin: origin.into(),
            size: px(12.0),
            text,
        }
        .render(self.window, self.cx);
    }

    pub fn render_axes(&mut self, bounds: Bounds<Pixels>) {
        let mut model = self.model.write();
        for axes in model.axes.iter_mut() {
            axes.render_pixels(bounds, self.window, self.cx);
        }
    }
}
