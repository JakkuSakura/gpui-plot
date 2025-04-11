use gpui::{
    div, prelude::*, px, size, App, AppContext, Application, Bounds, Entity, Hsla, Window,
    WindowBounds, WindowOptions,
};
use gpui_plot::figure::axes::AxesContext;
use gpui_plot::figure::axes::AxesModel;
use gpui_plot::figure::figure::{FigureModel, FigureView};
use gpui_plot::figure::grid::GridModel;
use gpui_plot::geometry::{point2, AxesBounds, AxisRange, GeometryAxes, Line};
use parking_lot::RwLock;
use plotters::prelude::*;
use std::sync::Arc;

#[allow(unused)]
struct MainView {
    model: Arc<RwLock<FigureModel>>,
    axes_model: Arc<RwLock<AxesModel<f64, f64>>>,
    animation: Animation,
    figure: Entity<FigureView>,
}

impl MainView {
    fn new(_window: &mut Window, cx: &mut App) -> Self {
        let model = FigureModel::new("Example Figure".to_string());
        let model = Arc::new(RwLock::new(model));
        let animation = Animation::new(0.0, 100.0, 0.1);

        let axes_bounds = AxesBounds::new(AxisRange::new(0.0, 100.0), AxisRange::new(0.0, 100.0));
        let grid = GridModel::from_numbers(10, 10);
        let axes_model = Arc::new(RwLock::new(AxesModel::new(axes_bounds, grid)));

        Self {
            figure: cx.new(|_| FigureView::new(model.clone())),
            axes_model: axes_model.clone(),
            model,
            animation,
        }
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = cx.entity_id();
        cx.defer(move |app| app.notify(id));

        let mut model = self.model.write();
        model.clear_plots();
        model.add_plot_with(|plot| {
            plot.add_axes_with(self.axes_model.clone(), |axes| {
                axes.clear_elements();
                axes.plot(self.animation.clone());
            });
            let mut animation = self.animation.clone();
            plot.add_axes_plotters(self.axes_model.clone(), move |area, cx| {
                let mut chart = ChartBuilder::on(&area)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(cx.axes_bounds.x.to_range(), cx.axes_bounds.y.to_range())
                    .unwrap();

                chart.configure_mesh().draw().unwrap();
                for shift in 0..20 {
                    let line = animation.next_line((shift * 5) as f64, false);

                    chart
                        .draw_series(LineSeries::new(
                            line.points.iter().map(|p| (p.x, p.y)),
                            &RED,
                        ))
                        .unwrap();
                }
            });
            plot.update();
        });
        div()
            .size_full()
            .flex_col()
            .bg(gpui::white())
            .text_color(gpui::black())
            .child(self.figure.clone())
    }
}
#[derive(Clone)]
struct Animation {
    start: f64,
    end: f64,
    step: f64,
    time_start: std::time::Instant,
}
impl Animation {
    fn new(start: f64, end: f64, step: f64) -> Self {
        Self {
            start,
            end,
            step,
            time_start: std::time::Instant::now(),
        }
    }
    fn next_line(&mut self, shift: f64, transpose: bool) -> Line<f64, f64> {
        let mut line = Line::new().color(Hsla::green());
        let t = self.time_start.elapsed().as_secs_f64() * 10.0;
        let mut x = self.start;
        while x <= self.end {
            let y = (x + t).sin();
            let mut point = point2(x, y + shift);
            if transpose {
                point = point.flip();
            }
            line.add_point(point);
            x += self.step;
        }
        line
    }
}
impl GeometryAxes for Animation {
    type X = f64;
    type Y = f64;

    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        for shift in 0..20 {
            let mut line = self.next_line((shift * 5) as f64, true);
            line.render_axes(cx);
        }
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| MainView::new(window, cx)),
        )
        .unwrap();
    });
}
