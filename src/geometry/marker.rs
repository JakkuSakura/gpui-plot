use crate::figure::axes::AxesContext;
use crate::geometry::{AxisRange, AxisType, GeometryAxes, GeometryPixels, Point2};
use gpui::{point, Hsla, Path, PathBuilder, Pixels};

#[derive(Debug, Clone, Copy)]
pub enum MarkerShape {
    Circle,
    Square,
    TriangleUp,
    TriangleDown,
}
pub struct Marker<X: AxisType, Y: AxisType> {
    pub position: Point2<X, Y>,
    pub size: Pixels,
    pub color: Hsla,
    pub shape: MarkerShape,
}
impl<X: AxisType, Y: AxisType> Marker<X, Y> {
    pub fn new(position: Point2<X, Y>, size: Pixels) -> Self {
        Self {
            position,
            size,
            color: gpui::black(),
            shape: MarkerShape::Circle,
        }
    }
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}
impl Marker<Pixels, Pixels> {
    fn get_path(&self) -> Path<Pixels> {
        let mut builder = PathBuilder::fill();
        match self.shape {
            MarkerShape::Circle => {
                for i in 0..16 {
                    let angle = i as f32 * std::f32::consts::PI / 8.0;

                    let x = self.position.x + self.size * angle.cos();
                    let y = self.position.y + self.size * angle.sin();
                    if i == 0 {
                        builder.move_to(point(x, y));
                    } else {
                        builder.line_to(point(x, y));
                    }
                }
                builder.close();
            }
            MarkerShape::Square => {
                builder.move_to(point(
                    self.position.x - self.size / 2.0,
                    self.position.y - self.size / 2.0,
                ));
                builder.line_to(point(
                    self.position.x + self.size / 2.0,
                    self.position.y - self.size / 2.0,
                ));
                builder.line_to(point(
                    self.position.x + self.size / 2.0,
                    self.position.y + self.size / 2.0,
                ));
                builder.line_to(point(
                    self.position.x - self.size / 2.0,
                    self.position.y + self.size / 2.0,
                ));
                builder.close();
            }
            MarkerShape::TriangleUp => {
                builder.move_to(point(self.position.x, self.position.y));
                builder.line_to(point(
                    self.position.x + self.size,
                    self.position.y + self.size,
                ));
                builder.line_to(point(
                    self.position.x - self.size,
                    self.position.y + self.size,
                ));
                builder.close();
            }
            MarkerShape::TriangleDown => {
                builder.move_to(point(self.position.x, self.position.y));
                builder.line_to(point(
                    self.position.x + self.size,
                    self.position.y - self.size,
                ));
                builder.line_to(point(
                    self.position.x - self.size,
                    self.position.y - self.size,
                ));
                builder.close();
            }
        }
        builder.build().unwrap()
    }
    pub fn render(&self, window: &mut gpui::Window, pixel_bounds: Option<gpui::Bounds<Pixels>>) {
        if let Some(bounds) = pixel_bounds {
            if !bounds.contains(&self.position.into()) {
                return;
            }
        }
        let path = self.get_path();
        window.paint_path(path, self.color);
    }
}
impl GeometryPixels for Marker<Pixels, Pixels> {
    fn render_pixels(
        &mut self,
        bounds: gpui::Bounds<Pixels>,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) {
        self.render(window, Some(bounds));
    }
}
impl<X: AxisType, Y: AxisType> GeometryAxes for Marker<X, Y> {
    type X = X;
    type Y = Y;
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        let pixel_bounds = cx.pixel_bounds.into_bounds();
        let position = cx.transform_point(self.position);

        let mut marker = Marker::new(position.into(), self.size)
            .color(self.color)
            .shape(self.shape);
        let (window, cx) = cx.cx.as_mut().unwrap();

        marker.render_pixels(pixel_bounds, window, cx);
    }
}

pub struct Markers<X: AxisType, Y: AxisType> {
    pub markers: Vec<Marker<X, Y>>,
}
impl<X: AxisType, Y: AxisType> Markers<X, Y> {
    pub fn new() -> Self {
        Self { markers: vec![] }
    }
    pub fn add_marker(&mut self, marker: Marker<X, Y>) {
        self.markers.push(marker);
    }

    pub fn add_markers(&mut self, markers: Vec<Marker<X, Y>>) {
        self.markers.extend(markers);
    }
}
impl<X: AxisType, Y: AxisType> GeometryAxes for Markers<X, Y> {
    type X = X;
    type Y = Y;
    fn get_x_range(&self) -> Option<AxisRange<Self::X>> {
        if self.markers.is_empty() {
            return None;
        }
        let mut min = self.markers[0].position.x;
        let mut max = self.markers[0].position.x;
        for marker in self.markers.iter() {
            if marker.position.x < min {
                min = marker.position.x;
            }
            if marker.position.x > max {
                max = marker.position.x;
            }
        }
        Some(AxisRange::new(min, max))
    }
    fn get_y_range(&self) -> Option<AxisRange<Self::Y>> {
        if self.markers.is_empty() {
            return None;
        }
        let mut min = self.markers[0].position.y;
        let mut max = self.markers[0].position.y;
        for marker in self.markers.iter() {
            if marker.position.y < min {
                min = marker.position.y;
            }
            if marker.position.y > max {
                max = marker.position.y;
            }
        }
        Some(AxisRange::new(min, max))
    }
    fn render_axes(&mut self, cx: &mut AxesContext<Self::X, Self::Y>) {
        for marker in self.markers.iter_mut() {
            marker.render_axes(cx);
        }
    }
}
