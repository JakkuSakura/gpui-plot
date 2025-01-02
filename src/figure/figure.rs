use crate::figure::plot::{PlotModel, PlotViewer};
use crate::figure::text::centered_text;
use gpui::{div, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext};
use parking_lot::RwLock;
use std::fmt::Debug;
use std::sync::Arc;

pub struct FigureModel {
    pub title: String,
    pub plots: Vec<Arc<RwLock<PlotModel>>>,
    pub plot_index: usize,
}
impl Debug for FigureModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FigureContext")
            .field("title", &self.title)
            .field("plots", &self.plots)
            .finish()
    }
}

impl FigureModel {
    pub fn new(title: String) -> Self {
        Self {
            title,
            plots: Vec::new(),
            plot_index: 0,
        }
    }
    pub fn add_plot(&mut self) -> &mut Arc<RwLock<PlotModel>> {
        let index = self.plot_index;
        self.plot_index += 1;
        if index < self.plots.len() {
            &mut self.plots[index]
        } else {
            let model = Arc::new(RwLock::new(PlotModel::new()));
            self.plots.push(model);
            self.plots.last_mut().unwrap()
        }
    }
}

/// A Figure is per definition of matplotlib: https://matplotlib.org/stable/users/explain/quick_start.html
/// It contains a title, a canvas, 2 axes, and a legend.
/// The canvas is the main area where the plot is drawn.

pub struct FigureViewer {
    pub model: Arc<RwLock<FigureModel>>,
    pub plots: Vec<View<PlotViewer>>,
}
impl FigureViewer {
    pub fn new(model: Arc<RwLock<FigureModel>>) -> Self {
        Self {
            model,
            plots: Vec::new(),
        }
    }
    fn add_views(&mut self, cx: &mut ViewContext<Self>) {
        for i in self.plots.len()..self.model.read().plots.len() {
            let plot_model = self.model.read().plots[i].clone();
            let view = PlotViewer::new(plot_model.clone());
            let plot = cx.new_view(move |_| view);
            self.plots.push(plot);
        }
    }
}
impl Render for FigureViewer {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.add_views(cx);
        self.model.write().plot_index = 0;
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(gpui::white())
            .text_color(gpui::black())
            .child(centered_text(self.model.read().title.clone()))
            .children(self.plots.clone())
    }
}
