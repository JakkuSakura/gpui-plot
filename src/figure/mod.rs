pub mod axes;
#[allow(clippy::module_inception)]
pub mod figure;
pub mod grid;
pub mod plot;
#[cfg(feature = "plotters")]
pub mod plotters;
pub mod text;
pub mod ticks;

// figure -> (sub)plot -> axes(multiple)
