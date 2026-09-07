//! PyO3 wrappers around [`crate::native`].
//!
//! Native types stay PyO3-free. This module is the only place that defines
//! `#[pyclass]` types and the `rapidquery._core.*` submodules they live in.
//! Serializers live here because they inspect Python values.

pub mod data_type;
pub mod dialect;

pub use data_type::data_type_mod;
pub use dialect::dialect_mod;
