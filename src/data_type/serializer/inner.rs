use crate::{
    dialect::PySqlDialect,
    internal::{alias, modules},
};

/// A thin enum which specifies the serializer type.
///
/// You can use it to serialize different Python objects.
#[derive(Debug, Clone)]
pub enum SerializerType {
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
    Array(Box<SerializerInner>),
    Custom(alias::PyObject),
}

/// How a Python object is written as a SQL literal.
///
/// It wraps [`SerializerType`] with additional `prefix` and `suffix` fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializerInner {
    pub r#type: SerializerType,
    pub prefix: Option<arcstr::ArcStr>,
    pub suffix: Option<arcstr::ArcStr>,
}

impl PartialEq for SerializerType {
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
            (Self::Custom(a), Self::Custom(b)) => {
                // We prefer to do not use Python level equal operator
                // to avoid GIL problems. This guarantees safety.
                std::ptr::eq(a.as_ptr(), b.as_ptr())
            }
            _ => false,
        }
    }
}
impl Eq for SerializerType {}

impl std::hash::Hash for SerializerType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Array(element) => element.hash(state),
            Self::Custom(function) => {
                // Same note like [`SerializerType::eq`]
                (function.as_ptr() as usize).hash(state)
            }
            _ => {}
        }
    }
}

impl SerializerType {
    /// Built-in serializer for a [`DataTypeKind`].
    ///
    /// [`DataTypeKind::Custom`] maps to [`Self::Unspecified`]. Json and Jsonb
    /// share [`Self::Json`].
    #[inline]
    pub fn from_kind(kind: crate::data_type::DataTypeKind) -> Self {
        match kind {
            crate::data_type::DataTypeKind::Integer => Self::Integer,
            crate::data_type::DataTypeKind::Float => Self::Float,
            crate::data_type::DataTypeKind::Boolean => Self::Boolean,
            crate::data_type::DataTypeKind::String => Self::String,
            crate::data_type::DataTypeKind::Uuid => Self::Uuid,
            crate::data_type::DataTypeKind::Bytes => Self::Bytes,
            crate::data_type::DataTypeKind::Bit => Self::Bit,
            crate::data_type::DataTypeKind::Json | crate::data_type::DataTypeKind::Jsonb => {
                Self::Json
            }
            crate::data_type::DataTypeKind::Date => Self::Date,
            crate::data_type::DataTypeKind::Time => Self::Time,
            crate::data_type::DataTypeKind::Datetime => Self::Datetime,
            crate::data_type::DataTypeKind::Inet => Self::Inet,
            crate::data_type::DataTypeKind::MacAddress => Self::MacAddress,
            crate::data_type::DataTypeKind::Custom => Self::Unspecified,
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

            // str/enum.StrEnum -> String
            if pyo3::ffi::PyUnicode_Check(object_ptr) == 1 {
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
                && modules::json_dumps(bound.clone(), |_| Ok(())).is_ok()
            {
                return Self::Json;
            }
        }

        // * -> Unspecified
        Self::Unspecified
    }
}

impl SerializerType {
    pub fn serialize(
        &self,
        dialect: &PySqlDialect,
        object: &alias::BoundObject<'_>,
        out: &mut String,
    ) -> pyo3::PyResult<()> {
        let py = object.py();
        let ptr = object.as_ptr();

        unsafe {
            match self {
                Self::Integer => super::functions::write_integer(py, ptr, out),
                Self::Float => super::functions::write_float(py, ptr, out),
                Self::Boolean => super::functions::write_bool(py, ptr, &dialect.0, out),
                Self::String => super::functions::write_str(py, ptr, &dialect.0, out),
                Self::Uuid => super::functions::write_uuid(py, ptr, &dialect.0, out),
                Self::Bytes => super::functions::write_bytes(py, ptr, &dialect.0, out),
                Self::Bit => super::functions::write_bit(py, ptr, &dialect.0, out),
                Self::Json => super::functions::write_json(py, ptr, &dialect.0, out),
                Self::Date => super::functions::write_date(py, ptr, out),
                Self::Time => super::functions::write_time(py, ptr, out),
                Self::Datetime => super::functions::write_datetime(py, ptr, out),
                Self::Inet => super::functions::write_inet(py, ptr, &dialect.0, out),
                Self::MacAddress => super::functions::write_macaddr(py, ptr, &dialect.0, out),
                Self::Array(element) => {
                    super::functions::write_array(py, ptr, dialect, element, out)
                }
                Self::Unspecified => {
                    let text = super::functions::pyobject_string(py, ptr)?;
                    out.push_str(&text);
                    Ok(())
                }
                Self::Custom(s) => super::functions::write_custom(py, ptr, dialect, s, object, out),
            }
        }
    }
}

impl SerializerInner {
    #[inline(always)]
    pub fn new(
        r#type: SerializerType,
        prefix: Option<arcstr::ArcStr>,
        suffix: Option<arcstr::ArcStr>,
    ) -> Self {
        Self {
            r#type,
            prefix,
            suffix,
        }
    }

    /// Serializes the `object` into literal SQL value.
    pub fn write_sql(
        &self,
        dialect: &PySqlDialect,
        object: &alias::BoundObject<'_>,
        out: &mut String,
    ) -> pyo3::PyResult<()> {
        if let Some(ref x) = self.prefix {
            out.push_str(x.as_str());
        }

        self.r#type.serialize(dialect, object, out)?;

        if let Some(ref x) = self.suffix {
            out.push_str(x.as_str());
        }

        Ok(())
    }
}

impl From<SerializerType> for SerializerInner {
    fn from(value: SerializerType) -> Self {
        Self {
            r#type: value,
            prefix: None,
            suffix: None,
        }
    }
}
