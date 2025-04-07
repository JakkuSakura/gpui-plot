use crate::figure::plot::{PlotModel, PlotView};
use crate::figure::text::centered_text;
use gpui::{
    div, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};
use parking_lot::RwLock;
use std::fmt::Debug;
use std::sync::Arc;

pub struct FigureModel {
    pub title: String,
    pub plots: Vec<Arc<RwLock<PlotModel>>>,
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
        }
    }
    pub fn clear_plots(&mut self) {
        self.plots.clear();
    }
    pub fn add_plot(&mut self) -> &mut Arc<RwLock<PlotModel>> {
        #[allow(clippy::arc_with_non_send_sync)]
        let model = Arc::new(RwLock::new(PlotModel::new()));
        self.plots.push(model);
        self.plots.last_mut().unwrap()
    }
    /// Update the figure model.
    pub fn update(&mut self) {
        for plot in self.plots.iter() {
            let mut plot = plot.write();
            plot.update();
        }
    }
}

/// A Figure is per definition of matplotlib: https://matplotlib.org/stable/users/explain/quick_start.html
/// It contains a title, a canvas, 2 axes, and a legend.
/// The canvas is the main area where the plot is drawn.
pub struct FigureView {
    pub model: Arc<RwLock<FigureModel>>,
    pub plots: Vec<Entity<PlotView>>,
}
impl FigureView {
    pub fn new(model: Arc<RwLock<FigureModel>>) -> Self {
        Self {
            model,
            plots: Vec::new(),
        }
    }
    fn add_views(&mut self, cx: &mut App) {
        for i in self.plots.len()..self.model.read().plots.len() {
            let plot_model = self.model.read().plots[i].clone();
            let view = PlotView::new(plot_model.clone());
            let plot = cx.new(move |_| view);
            self.plots.push(plot);
        }
    }
}
impl Render for FigureView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.add_views(cx);
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
