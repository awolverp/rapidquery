//! PyO3 wrappers around [`crate::native`].
//!
//! This module is the only place that defines `#[pyclass]` types and the
//! `rapidquery._core.*` submodules they live in. Native may hold and inspect
//! Python objects (`Serializer`, `Value`, `PrepareContext`); pyclasses wrap
//! those types.

pub mod data_type;
pub mod dialect;

mod ident;
mod value;

pub use data_type::data_type_mod;
pub use dialect::dialect_mod;
pub use ident::{IdentExtractor, PyAsterisk, PyIdent, PyUnquote};
pub use value::PyValue;
