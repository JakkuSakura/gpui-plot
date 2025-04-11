use crate::figure::axes::AxesContext;
use crate::geometry::{AxisRange, AxisType, GeometryAxes, GeometryPixels, Point2};
use gpui::{px, App, Bounds, Hsla, PathBuilder, Pixels, Window};
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
impl Default for Line<Pixels, Pixels> {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }
    pub fn add_point(&mut self, point: Point2<X, Y>) {
        self.points.push(point);
    }
    pub fn clear(&mut self) {
        self.points.clear();
    }
}
impl Line<Pixels, Pixels> {
    pub fn render(
        &mut self,
        window: &mut Window,
        _cx: &mut App,
        pixel_bounds: Option<Bounds<Pixels>>,
    ) {
        match pixel_bounds {
            Some(bounds) => {
                let mut i = 0;
                let mut line = Line::new().width(self.width).color(self.color);
                while i < self.points.len() {
                    while i < self.points.len() {
                        let point = self.points[i];

                        // Check if the point is within the bounds
                        if !bounds.contains(&point.into()) {
                            // break and draw the line
                            i += 1;
                            break;
                        }

                        line.add_point(point);
                        i += 1;
                    }
                    line.render(window, _cx, None);
                    line.clear();
                }
            }
            None => {
                if self.points.is_empty() {
                    warn!("Line must have at least 1 points to render");
                    return;
                }

                let mut builder = PathBuilder::stroke(px(self.width.0));
                let Some(first_p) = self.points.first() else {
                    return;
                };

                builder.move_to((*first_p).into());
                for p in self.points.iter().skip(1) {
                    builder.line_to((*p).into());
                }

                if let Ok(path) = builder.build() {
                    window.paint_path(path, self.color);
                }
            }
        }
    }
}
impl GeometryPixels for Line<Pixels, Pixels> {
    fn render_pixels(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        self.render(window, cx, Some(bounds));
    }
}
impl<X: AxisType, Y: AxisType> GeometryAxes for Line<X, Y> {
    type X = X;
    type Y = Y;
    fn get_x_range(&self) -> Option<AxisRange<Self::X>> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = self.points[0].x;
        let mut max = self.points[0].x;
        for point in self.points.iter() {
            if point.x < min {
                min = point.x;
            }
            if point.x > max {
                max = point.x;
            }
        }
        Some(AxisRange::new(min, max))
    }
    fn get_y_range(&self) -> Option<AxisRange<Self::Y>> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = self.points[0].y;
        let mut max = self.points[0].y;
        for point in self.points.iter() {
            if point.y < min {
                min = point.y;
            }
            if point.y > max {
                max = point.y;
            }
        }
        Some(AxisRange::new(min, max))
    }
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        let mut line = Line::new();
        for point in self.points.iter().cloned() {
            let point = cx.transform_point(point);
            line.add_point(point.into());
        }
        let pixel_bounds = cx.pixel_bounds.into_bounds();
        let (window, cx) = cx.cx.as_mut().unwrap();
        line.render(window, cx, Some(pixel_bounds));
    }
}
