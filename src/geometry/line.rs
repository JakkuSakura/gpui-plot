use crate::figure::axes::AxesContext;
use crate::geometry::{AxisType, GeometryAxes, GeometryPixels, Point2};
use gpui::{App, Bounds, Hsla, Pixels, Window};

#[derive(Clone, Debug, PartialEq)]
pub enum LineDirection {
    Horizontal,
    Vertical,
    Any,
}
#[derive(Clone, Debug)]
pub struct Line<X: AxisType, Y: AxisType> {
    pub points: Vec<Point2<X, Y>>,
    pub width: Pixels,
    pub color: Hsla,
    pub direction: LineDirection,
}

impl<X: AxisType, Y: AxisType> Line<X, Y> {
    pub fn new() -> Self {
        Self {
            points: vec![],
            width: 1.0.into(),
            color: gpui::black(),
            direction: LineDirection::Any,
        }
    }
    pub fn between_points(start: Point2<X, Y>, end: Point2<X, Y>) -> Self {
        let mut line = Self::new();
        line.add_point(start);
        line.add_point(end);
        if start.x == end.x {
            line.direction = LineDirection::Vertical;
        } else if start.y == end.y {
            line.direction = LineDirection::Horizontal;
        }
        line
    }
    pub fn width(mut self, width: f64) -> Self {
        self.width = width.into();
        self
    }
    pub fn add_point(&mut self, point: Point2<X, Y>) {
        self.points.push(point);
    }
}
impl Line<Pixels, Pixels> {
    pub fn render(&mut self, window: &mut Window, _cx: &mut App) {
        let mut line = plotters_gpui::line::Line::new();
        for point in self.points.iter().cloned() {
            line.add_point(point.into());
        }
        let mut line = line.width(self.width).color(self.color);
        line.render_pixels(window);

    }
}
impl GeometryPixels for Line<Pixels, Pixels> {
    fn render_pixels(&mut self, _bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        self.render(window, cx);
    }
}
impl<X: AxisType, Y: AxisType> GeometryAxes for Line<X, Y> {
    type X = X;
    type Y = Y;

    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        let mut line = Line::new();
        for point in self.points.iter().cloned() {
            let point = cx.transform_point(point);
            line.add_point(point.into());
        }
        let (window, cx) = cx.cx.as_mut().unwrap();
        line.render(window, cx);
    }
}
