use crate::geometry::{AxisType, Size2};
use gpui::{Point, Size};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2<X, Y> {
    pub x: X,
    pub y: Y,
}
impl<X: AxisType, Y: AxisType> Point2<X, Y> {
    pub fn new(x: X, y: Y) -> Self {
        Self { x, y }
    }
    pub fn sub_to_f32(self, other: Self) -> Size<f32> {
        Size {
            width: X::delta_to_f32(self.x - other.x),
            height: Y::delta_to_f32(self.y - other.y),
        }
    }
    pub fn add_from_f32(self, other: Size<f32>) -> Self {
        Self {
            x: self.x + X::delta_from_f32(other.width),
            y: self.y + Y::delta_from_f32(other.height),
        }
    }
}
impl<T: AxisType> From<Point2<T, T>> for (T, T) {
    fn from(point: Point2<T, T>) -> Self {
        (point.x, point.y)
    }
}
impl<T: AxisType> From<(T, T)> for Point2<T, T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}
impl<T: AxisType + Default> From<Point<T>> for Point2<T, T> {
    fn from(point: Point<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}
impl<T: AxisType + Default> From<Point2<T, T>> for Point<T> {
    fn from(point: Point2<T, T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}
impl<X: AxisType, Y: AxisType> Add<Size2<X::Delta, Y::Delta>> for Point2<X, Y> {
    type Output = Self;
    fn add(self, rhs: Size2<X::Delta, Y::Delta>) -> Self::Output {
        Self {
            x: self.x + rhs.width,
            y: self.y + rhs.height,
        }
    }
}
impl<X: AxisType, Y: AxisType> Sub<Size2<X::Delta, Y::Delta>> for Point2<X, Y> {
    type Output = Self;
    fn sub(self, rhs: Size2<X::Delta, Y::Delta>) -> Self::Output {
        Self {
            x: self.x - rhs.width,
            y: self.y - rhs.height,
        }
    }
}
impl<X: AxisType, Y: AxisType> Sub for Point2<X, Y> {
    type Output = Size2<X::Delta, Y::Delta>;
    fn sub(self, rhs: Self) -> Self::Output {
        Size2 {
            width: self.x - rhs.x,
            height: self.y - rhs.y,
        }
    }
}

pub fn point2<X, Y>(x: X, y: Y) -> Point2<X, Y> {
    Point2 { x, y }
}