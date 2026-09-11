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
use crate::internal::modules::json_dumps;

/// How a Python object is written as a SQL literal.
#[derive(Debug, Clone)]
pub enum SerializerEnum {
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
    Array(Box<SerializerEnum>),
    Custom(alias::PyObject),
}

impl PartialEq for SerializerEnum {
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

impl Eq for SerializerEnum {}

impl Hash for SerializerEnum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Array(element) => element.hash(state),
            Self::Custom(function) => (function.as_ptr() as usize).hash(state),
            _ => {}
        }
    }
}

impl SerializerEnum {
    /// Built-in serializer for a [`DataTypeGroup`].
    ///
    /// [`DataTypeGroup::Custom`] maps to [`Self::Unspecified`]. Json and Jsonb
    /// share [`Self::Json`].
    #[inline]
    pub fn from_group(kind: crate::data_type::DataTypeGroup) -> Self {
        match kind {
            crate::data_type::DataTypeGroup::Integer => Self::Integer,
            crate::data_type::DataTypeGroup::Float => Self::Float,
            crate::data_type::DataTypeGroup::Boolean => Self::Boolean,
            crate::data_type::DataTypeGroup::String => Self::String,
            crate::data_type::DataTypeGroup::Uuid => Self::Uuid,
            crate::data_type::DataTypeGroup::Bytes => Self::Bytes,
            crate::data_type::DataTypeGroup::Bit => Self::Bit,
            crate::data_type::DataTypeGroup::Json | crate::data_type::DataTypeGroup::Jsonb => {
                Self::Json
            }
            crate::data_type::DataTypeGroup::Date => Self::Date,
            crate::data_type::DataTypeGroup::Time => Self::Time,
            crate::data_type::DataTypeGroup::Datetime => Self::Datetime,
            crate::data_type::DataTypeGroup::Inet => Self::Inet,
            crate::data_type::DataTypeGroup::MacAddress => Self::MacAddress,
            crate::data_type::DataTypeGroup::Custom => Self::Unspecified,
        }
    }

    pub fn infer_bound(bound: &alias::BoundObject) -> Self {
        unsafe {
            let object_ptr = bound.as_ptr();
            let object_type_ptr = pyo3::ffi::Py_TYPE(object_ptr);

            // None / bool -> Boolean
            //
            // Why `None`?
            // Actually we don't use [`Serializer`] on `None` values.
            // So it's not important what [`Serializer`] is.
            // `None` values are handled by [`crate::sql::SqlWriter`]
            if pyo3::ffi::Py_IsNone(object_ptr) == 1 || pyo3::ffi::PyBool_Check(object_ptr) == 1 {
                return Self::Boolean;
            }

            // float / decimal -> Float
            if pyo3::ffi::PyFloat_CheckExact(object_ptr) == 1
                || object_type_ptr == crate::typeref::STD_DECIMAL_TYPE
            {
                return Self::Float;
            }

            // int -> Integer
            if pyo3::ffi::PyLong_CheckExact(object_ptr) == 1 {
                return Self::Integer;
            }

            // str/Enum -> String
            if pyo3::ffi::PyUnicode_CheckExact(object_ptr) == 1
                || object_type_ptr == crate::typeref::STD_ENUM_TYPE
            {
                return Self::String;
            }

            // bytes -> Bytes
            if pyo3::ffi::PyBytes_CheckExact(object_ptr) == 1 {
                return Self::Bytes;
            }

            // date -> Date
            if object_type_ptr == crate::typeref::STD_DATE_TYPE {
                return Self::Date;
            }

            // time -> Time
            if object_type_ptr == crate::typeref::STD_TIME_TYPE {
                return Self::Time;
            }

            // datetime -> Datetime
            if object_type_ptr == crate::typeref::STD_DATETIME_TYPE {
                return Self::Datetime;
            }

            // UUID -> Uuid
            if object_type_ptr == crate::typeref::STD_UUID_TYPE {
                return Self::Uuid;
            }

            // list/dict -> Json
            if (pyo3::ffi::PyList_CheckExact(object_ptr) == 1
                || pyo3::ffi::PyDict_CheckExact(object_ptr) == 1)
                && json_dumps(bound.clone(), |_| Ok(())).is_ok()
            {
                return Self::Json;
            }
        }

        // * -> Unspecified
        Self::Unspecified
    }
}
