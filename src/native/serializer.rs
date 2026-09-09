//! Value-serialization tree.
//!
//! [`Serializer`] turns a Python object into a SQL literal. Unit variants are
//! inferred from [`DataTypeGroup`]. [`Serializer::Array`] and
//! [`Serializer::Custom`] carry state (`Custom` holds a Python callable).
//!
//! Python sees frozen subclasses wrapping this enum (`IntegerSerializer`, …).
//! The write path lives in [`super::serialize`].

use std::hash::{Hash, Hasher};

use crate::internal::alias;
use crate::native::data_type::DataTypeGroup;

/// How a Python object is written as a SQL literal.
#[derive(Debug, Clone)]
pub enum Serializer {
    Integer,
    Float,
    Boolean,
    String,
    Uuid,
    Bytes,
    Bit,
    Json,
    Date,
    Time,
    Datetime,
    Inet,
    MacAddress,
    Unspecified,
    Array(Box<Serializer>),
    Custom(alias::PyObject),
}

impl PartialEq for Serializer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer, Self::Integer)
            | (Self::Float, Self::Float)
            | (Self::Boolean, Self::Boolean)
            | (Self::String, Self::String)
            | (Self::Uuid, Self::Uuid)
            | (Self::Bytes, Self::Bytes)
            | (Self::Bit, Self::Bit)
            | (Self::Json, Self::Json)
            | (Self::Date, Self::Date)
            | (Self::Time, Self::Time)
            | (Self::Datetime, Self::Datetime)
            | (Self::Inet, Self::Inet)
            | (Self::MacAddress, Self::MacAddress)
            | (Self::Unspecified, Self::Unspecified) => true,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Custom(a), Self::Custom(b)) => std::ptr::eq(a.as_ptr(), b.as_ptr()),
            _ => false,
        }
    }
}

impl Eq for Serializer {}

impl Hash for Serializer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Array(element) => element.hash(state),
            Self::Custom(function) => (function.as_ptr() as usize).hash(state),
            _ => {}
        }
    }
}

impl Serializer {
    /// Built-in serializer for a [`DataTypeGroup`].
    ///
    /// [`DataTypeGroup::Custom`] maps to [`Self::Unspecified`]. Json and Jsonb
    /// share [`Self::Json`].
    #[inline]
    pub fn from_group(kind: DataTypeGroup) -> Self {
        match kind {
            DataTypeGroup::Integer => Self::Integer,
            DataTypeGroup::Float => Self::Float,
            DataTypeGroup::Boolean => Self::Boolean,
            DataTypeGroup::String => Self::String,
            DataTypeGroup::Uuid => Self::Uuid,
            DataTypeGroup::Bytes => Self::Bytes,
            DataTypeGroup::Bit => Self::Bit,
            DataTypeGroup::Json | DataTypeGroup::Jsonb => Self::Json,
            DataTypeGroup::Date => Self::Date,
            DataTypeGroup::Time => Self::Time,
            DataTypeGroup::Datetime => Self::Datetime,
            DataTypeGroup::Inet => Self::Inet,
            DataTypeGroup::MacAddress => Self::MacAddress,
            DataTypeGroup::Custom => Self::Unspecified,
        }
    }
}
