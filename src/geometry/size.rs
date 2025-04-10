use crate::geometry::AxisType;
use gpui::{size, Size};
use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size2<X, Y> {
    pub width: X,
    pub height: Y,
}

impl<X: AxisType, Y: AxisType> Size2<X, Y> {
    pub fn new(width: X, height: Y) -> Self {
        Self { width, height }
    }
    pub fn to_f64(self) -> Size<f64> {
        size(self.width.to_f64(), self.height.to_f64())
    }
}

impl<X: Mul<f32>, Y: Mul<f32>> Mul<f32> for Size2<X, Y> {
    type Output = Size2<X::Output, Y::Output>;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

pub fn size2<X, Y>(width: X, height: Y) -> Size2<X, Y> {
    Size2 { width, height }
}
