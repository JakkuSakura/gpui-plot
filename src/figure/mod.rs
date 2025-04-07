use std::sync::Arc;
use parking_lot::RwLock;

pub mod axes;
#[allow(clippy::module_inception)]
pub mod figure;
pub mod grid;
pub mod plot;
pub mod text;
pub mod ticks;

// figure -> (sub)plot -> axes(multiple)
pub type SharedModel<T> = Arc<RwLock<T>>;
