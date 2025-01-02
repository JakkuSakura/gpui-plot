use crate::figure::axes::AxesContext;
use crate::geometry::{AxisType, GeometryAxes, GeometryPixels, Point2};
use gpui::{point, Bounds, Hsla, Path, Pixels, Point, WindowContext};
use tracing::warn;

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
    pub fn render(&mut self, cx: &mut WindowContext) {
        if self.points.len() < 1 {
            warn!("Line must have at least 1 points to render");
            return;
        }
        let first_point: Point<Pixels> = self.points[0].into();
        let width = self.width;

        let mut angle = f32::atan2(
            self.points.first().unwrap().y.0 - self.points.last().unwrap().y.0,
            self.points.first().unwrap().x.0 - self.points.last().unwrap().x.0,
        );
        angle += std::f32::consts::FRAC_PI_2;
        let shift = point(width * f32::cos(angle), width * f32::sin(angle));
        let mut reversed_points = vec![first_point + shift];
        let mut path = Path::new(first_point);
        for p in self.points.iter().cloned().skip(1) {
            let p: Point<Pixels> = p.into();
            path.line_to(p);

            reversed_points.push(p + shift);
        }
        // now do the reverse to close the path
        for p in reversed_points.into_iter().rev() {
            path.line_to(p);
        }

        cx.paint_path(path, self.color);
    }
}
impl GeometryPixels for Line<Pixels, Pixels> {
    fn render_pixels(&mut self, _bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        self.render(cx);
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
        line.render(cx.cx());
    }
}
