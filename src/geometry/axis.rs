use crate::geometry::point::Point2;
use crate::geometry::{point2, Size2};
use crate::utils::math::display_double_smartly;
use chrono::NaiveDate;
use gpui::{px, Bounds, Pixels, Point, Size};
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::ops::{Add, Range, Sub};

pub trait AxisType:
    Copy
    + Clone
    + PartialOrd
    + Debug
    + Send
    + Sync
    + Add<Self::Delta, Output = Self>
    + Sub<Self::Delta, Output = Self>
    + Sub<Self, Output = Self::Delta>
    + 'static
{
    type Delta: Copy + Clone + PartialOrd + Debug + Send + Sync + 'static;
    fn format(&self) -> String;
    fn delta_to_f32(value: Self::Delta) -> f32;
    fn delta_from_f32(value: f32) -> Self::Delta;
    fn min_delta() -> Self::Delta;
}
impl AxisType for f32 {
    type Delta = f32;
    fn format(&self) -> String {
        display_double_smartly(*self as f64)
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value
    }
    fn delta_from_f32(value: f32) -> Self::Delta {
        value
    }
    fn min_delta() -> Self::Delta {
        f32::MIN_POSITIVE * 2.0
    }
}
impl AxisType for f64 {
    type Delta = f64;
    fn format(&self) -> String {
        display_double_smartly(*self)
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value as f32
    }
    fn delta_from_f32(value: f32) -> Self::Delta {
        value as f64
    }
    fn min_delta() -> Self::Delta {
        f64::MIN_POSITIVE * 2.0
    }
}
impl AxisType for NaiveDate {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.num_nanoseconds().expect("out of range") as f32
    }

    fn delta_from_f32(value: f32) -> Self::Delta {
        chrono::Duration::nanoseconds(value as i64)
    }
    fn min_delta() -> Self::Delta {
        chrono::Duration::days(2)
    }
}
impl<Tz> AxisType for chrono::DateTime<Tz>
where
    Tz: chrono::TimeZone + Copy + 'static,
    Tz::Offset: Copy + Send + Sync + Display,
{
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.num_nanoseconds().expect("out of range") as f32
    }

    fn delta_from_f32(value: f32) -> Self::Delta {
        chrono::Duration::nanoseconds(value as i64)
    }
    fn min_delta() -> Self::Delta {
        chrono::Duration::nanoseconds(2)
    }
}
impl AxisType for chrono::NaiveDateTime {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.num_nanoseconds().expect("out of range") as f32
    }

    fn delta_from_f32(value: f32) -> Self::Delta {
        chrono::Duration::nanoseconds(value as i64)
    }
    fn min_delta() -> Self::Delta {
        chrono::Duration::nanoseconds(2)
    }
}
impl AxisType for chrono::NaiveTime {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.num_nanoseconds().expect("out of range") as f32
    }

    fn delta_from_f32(value: f32) -> Self::Delta {
        chrono::Duration::nanoseconds(value as i64)
    }
    fn min_delta() -> Self::Delta {
        chrono::Duration::nanoseconds(2)
    }
}
impl AxisType for chrono::Duration {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.num_nanoseconds().expect("out of range") as f32
    }

    fn delta_from_f32(value: f32) -> Self::Delta {
        chrono::Duration::nanoseconds(value as i64)
    }
    fn min_delta() -> Self::Delta {
        chrono::Duration::nanoseconds(2)
    }
}

impl AxisType for std::time::Duration {
    type Delta = std::time::Duration;
    fn format(&self) -> String {
        format!("{:?}", self)
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.as_micros() as f32
    }
    fn delta_from_f32(value: f32) -> Self::Delta {
        std::time::Duration::from_micros(value as u64)
    }
    fn min_delta() -> Self::Delta {
        std::time::Duration::from_micros(2)
    }
}

/// more for internal use
impl AxisType for Pixels {
    type Delta = Pixels;
    fn format(&self) -> String {
        self.to_string()
    }
    fn delta_to_f32(value: Self::Delta) -> f32 {
        value.0
    }
    fn delta_from_f32(value: f32) -> Self::Delta {
        px(value)
    }
    fn min_delta() -> Self::Delta {
        px(2.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AxisRangePixels {
    min: Pixels,
    max: Pixels,
    size: f32,
    pub(crate) pixels_per_element: f32,
}
impl AxisRangePixels {
    pub fn from_bounds(min: Pixels, max: Pixels, size: f32) -> Self {
        Self {
            min,
            max,
            size,
            pixels_per_element: f32::NAN,
        }
    }
}
impl AxesBoundsPixels {
    pub fn from_bounds(bounds: Bounds<Pixels>) -> Self {
        Self {
            x: AxisRangePixels::from_bounds(
                bounds.origin.x,
                bounds.origin.x + bounds.size.width,
                bounds.size.width.0,
            ),
            y: AxisRangePixels::from_bounds(
                bounds.origin.y + bounds.size.height,
                bounds.origin.y,
                bounds.size.height.0,
            ),
        }
    }
    pub fn into_bounds(self) -> Bounds<Pixels> {
        Bounds {
            origin: Point {
                x: self.x.min,
                y: self.y.max,
            },
            size: Size {
                width: px(self.x.size),
                height: px(self.y.size),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AxisRange<T> {
    pub min: T,
    pub max: T,
    size_in_f32: f32,
}

impl<T: Clone> AxisRange<T> {
    /// Only for plotters' usage. Our range is always inclusive.
    pub fn to_range(&self) -> Range<T> {
        self.min.clone()..self.max.clone()
    }
}
impl<T: AxisType> AxisRange<T> {
    pub fn new(min: T, max: T) -> Option<Self> {
        let delta = max - min;
        // protect against NaN
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(delta >= T::min_delta()) {
            return None;
        }
        let size_in_f32 = T::delta_to_f32(delta);
        Some(Self {
            min,
            max,
            size_in_f32,
        })
    }

    pub fn clap(&self, value: T) -> T {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
    pub fn contains(&self, value: T) -> bool {
        value >= self.min && value <= self.max
    }
    pub fn difference(&self) -> T::Delta {
        self.max - self.min
    }
    pub fn pixels_per_element(&self, bounds: AxisRangePixels) -> f32 {
        bounds.size / self.size_in_f32
    }
    pub fn elements_per_pixels(&self, delta: Pixels, bounds: AxisRangePixels) -> T::Delta {
        T::delta_from_f32(delta.0 * self.size_in_f32 / bounds.size)
    }
    /// Transform a value from the range `[min, max]` to the range `[bounds.min, bounds.max]`
    pub fn transform(&self, bounds: AxisRangePixels, value: T) -> Pixels {
        let adjusted_pixels =
            T::delta_to_f32(value - self.min) * bounds.pixels_per_element + bounds.min.0;
        Pixels(adjusted_pixels)
    }
    pub fn transform_reverse(&self, bounds: AxisRangePixels, value: Pixels) -> T {
        self.min + T::delta_from_f32((value.0 - bounds.min.0) / bounds.pixels_per_element)
    }
    pub fn iter_step_by(&self, step: T::Delta) -> impl Iterator<Item = T> + '_ {
        let mut current = self.min;
        std::iter::from_fn(move || {
            if current > self.max {
                return None;
            }
            let result = current;
            current = current + step;
            Some(result)
        })
    }
    pub fn union(&self, other: &Self) -> Option<Self> {
        let min = match self.min.partial_cmp(&other.min)? {
            Ordering::Less => self.min,
            Ordering::Greater => other.min,
            Ordering::Equal => self.min,
        };
        let max = match self.max.partial_cmp(&other.max)? {
            Ordering::Less => other.max,
            Ordering::Greater => self.max,
            Ordering::Equal => self.max,
        };
        let delta = max - min;
        let size_in_f32 = T::delta_to_f32(delta);
        Some(Self {
            min,
            max,
            size_in_f32,
        })
    }
}
impl<T: AxisType> Add<T::Delta> for AxisRange<T> {
    type Output = Self;
    fn add(self, rhs: T::Delta) -> Self::Output {
        Self {
            min: self.min + rhs,
            max: self.max + rhs,
            size_in_f32: self.size_in_f32,
        }
    }
}
impl<T: AxisType> Sub<T::Delta> for AxisRange<T> {
    type Output = Self;
    fn sub(self, rhs: T::Delta) -> Self::Output {
        Self {
            min: self.min - rhs,
            max: self.max - rhs,
            size_in_f32: self.size_in_f32,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AxesBoundsPixels {
    pub x: AxisRangePixels,
    pub y: AxisRangePixels,
}
impl AxesBoundsPixels {
    pub fn min_x(&self) -> Pixels {
        self.x.min
    }
    pub fn max_x(&self) -> Pixels {
        self.x.max
    }
    pub fn width(&self) -> Pixels {
        px(self.x.size)
    }
    pub fn min_y(&self) -> Pixels {
        self.y.max
    }
    pub fn max_y(&self) -> Pixels {
        self.y.min
    }
    pub fn height(&self) -> Pixels {
        px(self.y.size)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AxesBounds<X, Y> {
    pub x: AxisRange<X>,
    pub y: AxisRange<Y>,
}

impl<X: AxisType, Y: AxisType> AxesBounds<X, Y> {
    pub fn new(x: AxisRange<X>, y: AxisRange<Y>) -> Self {
        Self { x, y }
    }
    pub fn transform_point(&self, bounds: AxesBoundsPixels, point: Point2<X, Y>) -> Point<Pixels> {
        Point {
            x: self.x.transform(bounds.x, point.x),
            y: self.y.transform(bounds.y, point.y),
        }
    }
    pub fn transform_point_reverse(
        &self,
        bounds: AxesBoundsPixels,
        point: Point<Pixels>,
    ) -> Point2<X, Y> {
        Point2 {
            x: self.x.transform_reverse(bounds.x, point.x),
            y: self.y.transform_reverse(bounds.y, point.y),
        }
    }
    pub fn min_point(&self) -> Point2<X, Y> {
        point2(self.x.min, self.y.min)
    }
    pub fn max_point(&self) -> Point2<X, Y> {
        point2(self.x.max, self.y.max)
    }
    pub fn contains(&self, point: Point2<X, Y>) -> bool {
        self.x.contains(point.x) && self.y.contains(point.y)
    }
}

impl<X: AxisType, Y: AxisType> Add<Size2<X::Delta, Y::Delta>> for AxesBounds<X, Y> {
    type Output = Self;

    fn add(self, rhs: Size2<X::Delta, Y::Delta>) -> Self::Output {
        Self {
            x: self.x + rhs.width,
            y: self.y + rhs.height,
        }
    }
}
