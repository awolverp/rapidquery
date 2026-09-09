//! SQL data-type names.
//!
//! A [`DataTypeName`] is a canonical spelling plus a [`DataTypeGroup`]. The
//! group is the inference key for which [`crate::native::Serializer`]
//! `DataType(name)` constructs when none is passed. Groups are
//! serializer-oriented: `NUMERIC` and `DECIMAL` sit in
//! [`DataTypeGroup::Float`] because they share a serializer with IEEE floats.

mod name;

#[cfg(test)]
mod tests;

pub use name::{DataTypeGroup, DataTypeName};
