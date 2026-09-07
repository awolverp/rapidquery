//! PyO3 wrappers around [`crate::native`].
//!
//! Native types stay PyO3-free. This module is the only place that defines
//! `#[pyclass]` types and the `rapidquery._core.*` submodules they live in.
//! Keep that split as Ident / DataType / statement builders land.

pub mod dialect;
pub use dialect::dialect_mod;
