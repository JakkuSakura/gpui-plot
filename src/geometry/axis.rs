use crate::geometry::point::Point2;
use crate::utils::math::display_double_smartly;
use chrono::{NaiveDate, Timelike};
use gpui::{point, px, Bounds, Pixels, Point, Size};
use std::cmp::Ordering;
use std::fmt::Debug;
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
    type Delta: AxisType;
    fn format(&self) -> String;
    fn to_f64(&self) -> f64;
    fn from_f64(value: f64) -> Self;
}
impl AxisType for f32 {
    type Delta = f32;
    fn format(&self) -> String {
        display_double_smartly(*self as f64)
    }
    fn to_f64(&self) -> f64 {
        *self as f64
    }
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}
impl AxisType for f64 {
    type Delta = f64;
    fn format(&self) -> String {
        display_double_smartly(*self)
    }
    fn to_f64(&self) -> f64 {
        *self
    }
    fn from_f64(value: f64) -> Self {
        value
    }
}
impl AxisType for NaiveDate {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn to_f64(&self) -> f64 {
        self.and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_nanos_opt()
            .unwrap() as f64
    }
    fn from_f64(value: f64) -> Self {
        let timestamp = value as i64;
        let date = chrono::DateTime::from_timestamp_nanos(timestamp);
        date.date_naive()
    }
}

impl AxisType for chrono::NaiveDateTime {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn to_f64(&self) -> f64 {
        self.and_utc().timestamp_nanos_opt().unwrap() as f64
    }
    fn from_f64(value: f64) -> Self {
        let timestamp = value as i64;
        let date = chrono::DateTime::from_timestamp_nanos(timestamp);
        date.naive_utc()
    }
}
impl AxisType for chrono::NaiveTime {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn to_f64(&self) -> f64 {
        self.num_seconds_from_midnight() as f64 / 1_000_000_000.0 + self.nanosecond() as f64
    }
    fn from_f64(value: f64) -> Self {
        let seconds = value as u32;
        let nanoseconds = ((value - seconds as f64) * 1_000_000_000.0) as u32;
        let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(seconds, nanoseconds);
        time.expect("out of range")
    }
}
impl AxisType for chrono::DateTime<chrono::Utc> {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn to_f64(&self) -> f64 {
        self.timestamp_nanos_opt().unwrap() as f64
    }
    fn from_f64(value: f64) -> Self {
        let timestamp = value as i64;
        chrono::DateTime::<chrono::Utc>::from_timestamp_nanos(timestamp)
    }
}
impl AxisType for chrono::Duration {
    type Delta = chrono::Duration;
    fn format(&self) -> String {
        self.to_string()
    }
    fn to_f64(&self) -> f64 {
        self.num_nanoseconds().expect("out of range") as f64
    }
    fn from_f64(value: f64) -> Self {
        chrono::Duration::nanoseconds(value as i64)
    }
}

impl AxisType for std::time::Duration {
    type Delta = std::time::Duration;
    fn format(&self) -> String {
        format!("{:?}", self)
    }

    fn to_f64(&self) -> f64 {
        self.as_nanos() as f64
    }

    fn from_f64(value: f64) -> Self {
        std::time::Duration::from_nanos(value as u64)
    }
}

/// more for internal use
impl AxisType for Pixels {
    type Delta = Pixels;
    fn format(&self) -> String {
        self.to_string()
    }

    fn to_f64(&self) -> f64 {
        self.0 as f64
    }

    fn from_f64(value: f64) -> Self {
        Pixels(value as f32)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AxisRangePixels {
    min: Pixels,
    max: Pixels,
    size: f64,
    pub(crate) pixels_per_element: f64,
}
impl AxisRangePixels {
    pub fn from_bounds(min: Pixels, max: Pixels, size: f64) -> Self {
        Self {
            min,
            max,
            size,
            pixels_per_element: f64::NAN,
        }
    }
}
impl AxesBoundsPixels {
    pub fn from_bounds(bounds: Bounds<Pixels>) -> Self {
        Self {
            x: AxisRangePixels::from_bounds(
                bounds.origin.x,
                bounds.origin.x + bounds.size.width,
                bounds.size.width.0 as f64,
            ),
            y: AxisRangePixels::from_bounds(
                bounds.origin.y + bounds.size.height,
                bounds.origin.y,
                bounds.size.height.0 as f64,
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
                width: px(self.x.size as f32),
                height: px(self.y.size as f32),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AxisRange<T> {
    pub(crate) base: T,
    pub(crate) min_to_base: f64,
    pub(crate) max_to_base: f64,
}

impl<T: AxisType> AxisRange<T> {
    /// Only for plotters' usage. Our range is always inclusive.
    pub fn to_range(&self) -> Range<T> {
        self.min()..self.max()
    }
    pub fn new(min: T, max: T) -> Self {
        let base = T::from_f64((max - min).to_f64() / 2.0);
        Self::new_with_base(base, min, max)
    }
    pub fn new_with_base(base: T, min: T, max: T) -> Self {
        Self {
            base,
            min_to_base: (min - base).to_f64(),
            max_to_base: (max - base).to_f64(),
        }
    }
    pub fn new_with_base_f64(base: T, min: f64, max: f64) -> Self {
        Self {
            base,
            min_to_base: min,
            max_to_base: max,
        }
    }
    pub fn set_min(&mut self, min: T) {
        self.min_to_base = (min - self.base).to_f64();
    }
    pub fn set_max(&mut self, max: T) {
        self.max_to_base = (max - self.base).to_f64();
    }
    pub fn min(&self) -> T {
        self.base + T::Delta::from_f64(self.min_to_base)
    }
    pub fn max(&self) -> T {
        self.base + T::Delta::from_f64(self.max_to_base)
    }

    pub fn contains(&self, value: T) -> bool {
        value >= self.min() && value <= self.max()
    }
    pub fn size_in_f64(&self) -> f64 {
        self.max_to_base - self.min_to_base
    }

    pub fn pixels_per_element(&self, bounds: AxisRangePixels) -> f64 {
        bounds.size / self.size_in_f64()
    }

    pub fn elements_per_pixels(&self, delta: Pixels, bounds: AxisRangePixels) -> f64 {
        delta.0 as f64 * self.size_in_f64() / bounds.size
    }
    /// Transform a value from the range `[min, max]` to the range `[bounds.min, bounds.max]`
    pub fn transform(&self, bounds: AxisRangePixels, value: T) -> Pixels {
        let adjusted_pixels =
            (value - self.min()).to_f64() * bounds.pixels_per_element + bounds.min.0 as f64;
        Pixels(adjusted_pixels as f32)
    }

    pub fn transform_reverse(&self, bounds: AxisRangePixels, value: Pixels) -> T {
        T::from_f64(
            self.min().to_f64()
                + ((value.0 - bounds.min.0) as f64 / bounds.pixels_per_element).to_f64(),
        )
    }
    pub fn transform_reverse_f64(&self, bounds: AxisRangePixels, value: f64) -> f64 {
        self.min_to_base + (value - bounds.min.0 as f64) / bounds.pixels_per_element
    }
    pub fn iter_step_by(&self, step: T::Delta) -> impl Iterator<Item = T> + '_ {
        let mut current = self.min();
        std::iter::from_fn(move || {
            if current > self.max() {
                return None;
            }
            let result = current;
            current = current + step;
            Some(result)
        })
    }
    pub fn iter_step_by_f64(&self, step: f64) -> impl Iterator<Item = T> + '_ {
        let mut current = self.min_to_base;
        std::iter::from_fn(move || {
            if current > self.max_to_base {
                return None;
            }
            let result = self.base + T::Delta::from_f64(current);
            current += step;
            Some(result)
        })
    }
    pub fn union(&self, other: &Self) -> Option<Self> {
        let base = match self.base.partial_cmp(&other.base)? {
            Ordering::Less => self.base,
            Ordering::Greater => other.base,
            Ordering::Equal => self.base,
        };
        let min = match self.min().partial_cmp(&other.min())? {
            Ordering::Less => self.min(),
            Ordering::Greater => other.min(),
            Ordering::Equal => self.min(),
        };
        let max = match self.max().partial_cmp(&other.max())? {
            Ordering::Less => other.max(),
            Ordering::Greater => self.max(),
            Ordering::Equal => self.max(),
        };

        Some(Self::new_with_base(base, min, max))
    }
}

impl<T: AxisType> Add<f64> for AxisRange<T> {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Self {
            base: self.base,
            min_to_base: self.min_to_base + rhs,
            max_to_base: self.max_to_base + rhs,
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
        px(self.x.size as f32)
    }
    pub fn min_y(&self) -> Pixels {
        self.y.max
    }
    pub fn max_y(&self) -> Pixels {
        self.y.min
    }
    pub fn height(&self) -> Pixels {
        px(self.y.size as f32)
    }
}
impl Add<Point<Pixels>> for AxesBoundsPixels {
    type Output = Self;

    fn add(self, rhs: Point<Pixels>) -> Self::Output {
        Self {
            x: AxisRangePixels {
                min: self.x.min + rhs.x,
                max: self.x.max + rhs.x,
                size: self.x.size,
                pixels_per_element: self.x.pixels_per_element,
            },
            y: AxisRangePixels {
                min: self.y.min + rhs.y,
                max: self.y.max + rhs.y,
                size: self.y.size,
                pixels_per_element: self.y.pixels_per_element,
            },
        }
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
    pub fn resize(&mut self, factor: f64) {
        let min_x = self.x.min_to_base;
        let max_x = self.x.max_to_base;
        let min_y = self.y.min_to_base;
        let max_y = self.y.max_to_base;
        let midpoint = point((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        let size = point((max_x - min_x) * factor, (max_y - min_y) * factor);
        self.x.min_to_base = midpoint.x - size.x / 2.0;
        self.x.max_to_base = midpoint.x + size.x / 2.0;
        self.y.min_to_base = midpoint.y - size.y / 2.0;
        self.y.max_to_base = midpoint.y + size.y / 2.0;
    }

    pub fn transform_point(&self, bounds: AxesBoundsPixels, point: Point2<X, Y>) -> Point<Pixels> {
        Point {
            x: self.x.transform(bounds.x, point.x),
            y: self.y.transform(bounds.y, point.y),
        }
    }

    pub fn transform_point_reverse_f64(
        &self,
        bounds: AxesBoundsPixels,
        p: Point<Pixels>,
    ) -> Point<f64> {
        point(
            self.x.transform_reverse_f64(bounds.x, p.x.0 as f64),
            self.y.transform_reverse_f64(bounds.y, p.y.0 as f64),
        )
    }

    pub fn min_point_f64(&self) -> Point<f64> {
        point(self.x.min_to_base, self.y.min_to_base)
    }

    pub fn max_point_f64(&self) -> Point<f64> {
        point(self.x.max_to_base, self.y.max_to_base)
    }
    pub fn contains(&self, point: Point2<X, Y>) -> bool {
        self.x.contains(point.x) && self.y.contains(point.y)
    }
}
// add Point<f64>
impl<X: AxisType, Y: AxisType> Add<Size<f64>> for AxesBounds<X, Y> {
    type Output = Self;

    fn add(mut self, rhs: Size<f64>) -> Self::Output {
        self.x = self.x + rhs.width;
        self.y = self.y + rhs.height;
        self
    }
}
