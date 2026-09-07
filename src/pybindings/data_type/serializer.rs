//! Python `Serializer` class hierarchy.
//!
//! Native inference is [`DataTypeGroup`](crate::native::data_type::DataTypeGroup).
//! Python sees frozen subclasses plus `ArraySerializer` / `CustomSerializer`,
//! which have no native counterpart.
//!
//! Unit instances compare and hash by variant (`IntegerSerializer() ==
//! IntegerSerializer()`) and `repr` as `IntegerSerializer()`. Array compares
//! by element tree; Custom by function identity.

use std::hash::{Hash, Hasher};

use crate::{internal::alias, native::data_type::DataTypeGroup};

// Value-serialization tree. Lives only in pybindings (`Custom` holds a
// Python callable; `Array` may wrap that). Not a pyclass — must not leak
// into Python `__doc__`.
#[derive(Debug, Clone)]
pub(super) enum SerializerInner {
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

impl PartialEq for SerializerInner {
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

impl Eq for SerializerInner {}

impl Hash for SerializerInner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Array(element) => element.hash(state),
            Self::Custom(function) => (function.as_ptr() as usize).hash(state),
            _ => {}
        }
    }
}

impl SerializerInner {
    fn from_group(kind: DataTypeGroup) -> Self {
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

    fn class_name(&self) -> &'static str {
        match self {
            Self::Integer => "IntegerSerializer",
            Self::Float => "FloatSerializer",
            Self::Boolean => "BooleanSerializer",
            Self::String => "StringSerializer",
            Self::Uuid => "UuidSerializer",
            Self::Bytes => "BytesSerializer",
            Self::Bit => "BitSerializer",
            Self::Json => "JSONSerializer",
            Self::Date => "DateSerializer",
            Self::Time => "TimeSerializer",
            Self::Datetime => "DatetimeSerializer",
            Self::Inet => "InetSerializer",
            Self::MacAddress => "MacAddressSerializer",
            Self::Unspecified => "UnspecifiedSerializer",
            Self::Array(_) => "ArraySerializer",
            Self::Custom(_) => "CustomSerializer",
        }
    }
}

// `subclass` so the built-in serializers can `extends` this type from Rust.
// `from_py_object` so `ArraySerializer(IntegerSerializer())` extracts the
// base. Python subclasses of `Serializer` are not used as built-ins.
/// Converts a Python object into a dialect-specific SQL literal string.
///
/// Serializers produce a correctly escaped and formatted SQL value that can
/// be embedded in queries. Each concrete subclass handles a particular
/// category of values (integers, strings, dates, arrays, etc.). Use
/// `CustomSerializer` when the built-in ones are insufficient.
///
/// This base class cannot be instantiated. Pass a dialect class or instance
/// to `serialize()`. Unit serializers compare equal by variant;
/// `ArraySerializer` by its element tree; `CustomSerializer` by function
/// identity.
#[rapidquery_macros::pyclass(
    "Serializer",
    subclass,
    from_py_object,
    module = "rapidquery._core.data_type"
)]
#[derive(Debug, Clone)]
pub struct PySerializer(pub(super) SerializerInner);

impl PySerializer {
    pub(super) fn inferred(kind: DataTypeGroup) -> Self {
        Self(SerializerInner::from_group(kind))
    }

    pub(crate) fn serialize_value(
        &self,
        dialect: crate::pybindings::dialect::PySqlDialect,
        object: alias::BoundObject<'_>,
    ) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.write_sql(&dialect.0, &object, &mut buf)?;
        Ok(buf)
    }

    // Wrap `self` as the matching Python subclass (`IntegerSerializer()`, …).
    pub(super) fn into_subclasses<'py>(
        self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        match &self.0 {
            SerializerInner::Integer => wrap::<PyIntegerSerializer>(py, self),
            SerializerInner::Float => wrap::<PyFloatSerializer>(py, self),
            SerializerInner::Boolean => wrap::<PyBooleanSerializer>(py, self),
            SerializerInner::String => wrap::<PyStringSerializer>(py, self),
            SerializerInner::Uuid => wrap::<PyUuidSerializer>(py, self),
            SerializerInner::Bytes => wrap::<PyBytesSerializer>(py, self),
            SerializerInner::Bit => wrap::<PyBitSerializer>(py, self),
            SerializerInner::Json => wrap::<PyJSONSerializer>(py, self),
            SerializerInner::Date => wrap::<PyDateSerializer>(py, self),
            SerializerInner::Time => wrap::<PyTimeSerializer>(py, self),
            SerializerInner::Datetime => wrap::<PyDatetimeSerializer>(py, self),
            SerializerInner::Inet => wrap::<PyInetSerializer>(py, self),
            SerializerInner::MacAddress => wrap::<PyMacAddressSerializer>(py, self),
            SerializerInner::Unspecified => wrap::<PyUnspecifiedSerializer>(py, self),
            SerializerInner::Array(_) => wrap::<PyArraySerializer>(py, self),
            SerializerInner::Custom(_) => wrap::<PyCustomSerializer>(py, self),
        }
    }
}

fn wrap<'py, T>(
    py: pyo3::Python<'py>,
    base: PySerializer,
) -> pyo3::PyResult<alias::BoundObject<'py>>
where
    T: pyo3::PyClass<BaseType = PySerializer> + Default,
{
    pyo3::Bound::new(
        py,
        pyo3::PyClassInitializer::from(base).add_subclass(T::default()),
    )
    .map(|x| x.into_any())
}

#[pyo3::pymethods]
impl PySerializer {
    /// Renders `object` as a SQL literal for `dialect`.
    ///
    /// Args:
    ///     dialect: A dialect class (`PostgreSQL`) or instance (`PostgreSQL()`).
    ///     object: The Python value to convert.
    ///
    /// Returns:
    ///     A SQL fragment suitable for embedding in a statement.
    ///
    /// Raises:
    ///     TypeError: If `object` is not an accepted type for this serializer.
    ///     ValueError: If `object` is the right type but cannot be represented
    ///             (for example a non-finite float).
    fn serialize(
        &self,
        dialect: crate::pybindings::dialect::PySqlDialect,
        object: alias::BoundObject<'_>,
    ) -> pyo3::PyResult<String> {
        self.serialize_value(dialect, object)
    }

    fn __repr__(&self) -> String {
        match &self.0 {
            SerializerInner::Array(element) => {
                format!("ArraySerializer({}())", element.class_name())
            }
            other => {
                let name = other.class_name();
                let mut buf = String::with_capacity(name.len() + 2);
                buf.push_str(name);
                buf.push_str("()");
                buf
            }
        }
    }

    // Bound<Self> so only instances match.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

macro_rules! impl_unit_serializers {
    ($($variant:ident => $name:ident => $pyname:literal, $doc:literal,)+) => {
        $(
            #[doc = $doc]
            #[rapidquery_macros::pyclass(
                $pyname,
                extends = PySerializer,
                module = "rapidquery._core.data_type"
            )]
            #[derive(Debug, Default)]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[inline]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(PySerializer(SerializerInner::$variant))
                        .add_subclass(Self)
                }
            }
        )+
    };
}

impl_unit_serializers! {
    Integer => PyIntegerSerializer => "IntegerSerializer",
        "Serializes Python integers as bare SQL integer literals.\n\n\
         Accepts Python `int` of any width (including values beyond 64-bit).\n\
         Rejects `bool`; `True` and `False` are not treated as integers.\n\n\
         Raises:\n    \
             TypeError: If the value is not an `int`.",
    Float => PyFloatSerializer => "FloatSerializer",
        "Serializes numeric values as bare SQL numeric literals.\n\n\
         Accepts `float`, `int`, and `decimal.Decimal`. Finite floats are formatted\n\
         so that `1.0` stays distinct from `1`. `str` is not accepted.\n\n\
         Raises:\n    \
             TypeError: If the value is not a `float`, `int`, or `decimal.Decimal`.\n    \
             ValueError: If the value is a non-finite float (`nan`, `inf`, `-inf`).",
    Boolean => PyBooleanSerializer => "BooleanSerializer",
        "Serializes Python booleans as dialect boolean literals.\n\n\
         Accepts Python `bool` only. The dialect chooses the spelling\n\
         (`TRUE`/`FALSE` or `1`/`0`).\n\n\
         Raises:\n    \
             TypeError: If the value is not a `bool`.",
    String => PyStringSerializer => "StringSerializer",
        "Serializes text as a quoted, escaped SQL string literal.\n\n\
         Accepts `str`, or an `enum.Enum` member whose `.value` is a `str`. The\n\
         dialect supplies quoting and escaping.\n\n\
         Raises:\n    \
             TypeError: If the value is not a `str` or a string-valued enum member.",
    Uuid => PyUuidSerializer => "UuidSerializer",
        "Serializes UUIDs as quoted SQL string literals.\n\n\
         Accepts `uuid.UUID` or `str`.\n\n\
         Raises:\n    \
             TypeError: If the value is not a `uuid.UUID` or `str`.",
    Bytes => PyBytesSerializer => "BytesSerializer",
        "Serializes binary data as a dialect blob/bytea literal.\n\n\
         Accepts exact `bytes` (not `bytearray` or `memoryview`).\n\n\
         Raises:\n    \
             TypeError: If the value is not `bytes`.",
    Bit => PyBitSerializer => "BitSerializer",
        "Serializes bit strings as dialect bit-string literals.\n\n\
         Accepts a `str` consisting only of `'0'` and `'1'` characters.\n\n\
         Raises:\n    \
             TypeError: If the value is not a `str`.\n    \
             ValueError: If the string is empty or contains characters other than\n            \
                 `'0'` and `'1'`.",
    Json => PyJSONSerializer => "JSONSerializer",
        "Serializes a JSON-encodable object as a quoted SQL string literal.\n\n\
         Accepts any object that the JSON encoder can dump. Uses `orjson` when it\n\
         is installed, otherwise the stdlib `json` module. The JSON text is then\n\
         wrapped with the dialect's string quoting.\n\n\
         Raises:\n    \
             TypeError: If the value cannot be JSON-encoded.",
    Date => PyDateSerializer => "DateSerializer",
        "Serializes dates as quoted `YYYY-MM-DD` literals.\n\n\
         Accepts exact `datetime.date` (not `datetime.datetime`).\n\n\
         Raises:\n    \
             TypeError: If the value is not a `datetime.date`.",
    Time => PyTimeSerializer => "TimeSerializer",
        "Serializes times as quoted `HH:MM:SS[.ffffff]` literals.\n\n\
         Accepts `datetime.time`.\n\n\
         Raises:\n    \
             TypeError: If the value is not a `datetime.time`.",
    Datetime => PyDatetimeSerializer => "DatetimeSerializer",
        "Serializes timestamps as quoted SQL timestamp literals.\n\n\
         Accepts naive or aware `datetime.datetime`, or `int`/`float` UTC epoch\n\
         seconds. Naive values keep wall time. Aware values keep their original\n\
         offset (they are not converted to UTC).\n\n\
         Raises:\n    \
             TypeError: If the value is not a `datetime.datetime`, `int`, or `float`.\n    \
             OverflowError: If an epoch value is out of range for a timestamp.",
    Inet => PyInetSerializer => "InetSerializer",
        "Serializes IP addresses and networks as quoted SQL literals.\n\n\
         Accepts `str` or `ipaddress` objects (`IPv4Address`, `IPv6Address`,\n\
         `IPv4Network`, `IPv6Network`, and related types whose `str()` is a valid\n\
         address or CIDR).\n\n\
         Raises:\n    \
             ValueError: If the value is not a valid IPv4/IPv6 address or network.",
    MacAddress => PyMacAddressSerializer => "MacAddressSerializer",
        "Serializes MAC addresses as quoted SQL literals.\n\n\
         Accepts a `str` using `:` or `-` as the separator.\n\n\
         Raises:\n    \
             ValueError: If the value is not a valid MAC address.",
    Unspecified => PyUnspecifiedSerializer => "UnspecifiedSerializer",
        "Inserts `str(obj)` into SQL with no quoting or type check.\n\n\
         Used when no serializer could be inferred from the type name. The result\n\
         is not a safe SQL value; prefer an explicit serializer.",
}

/// Serializes sequences as `ARRAY[elem, ...]` literals.
///
/// Each element is serialized independently with `element`. Accepts `list`
/// or `tuple`. Nested arrays are expressed by wrapping another
/// `ArraySerializer`.
#[rapidquery_macros::pyclass(
    "ArraySerializer",
    extends = PySerializer,
    module = "rapidquery._core.data_type"
)]
#[derive(Debug, Default)]
pub struct PyArraySerializer;

#[pyo3::pymethods]
impl PyArraySerializer {
    /// Creates an array serializer.
    ///
    /// Args:
    ///     element: The serializer applied to every array item. Nested arrays
    ///             are expressed by passing another `ArraySerializer`.
    #[new]
    #[inline]
    fn __new__(element: PySerializer) -> pyo3::PyClassInitializer<Self> {
        let inner = SerializerInner::Array(Box::new(element.0));
        pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self)
    }

    /// Returns the serializer used for each array element.
    ///
    /// Returns:
    ///     The `Serializer` passed to the constructor.
    #[getter]
    fn get_element(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let base = slf.as_super();
        match &base.0 {
            SerializerInner::Array(element) => {
                PySerializer((**element).clone()).into_subclasses(slf.py())
            }
            _ => unreachable!("ArraySerializer base is always Array"),
        }
    }
}

/// Serializes values by calling a user-supplied Python function.
///
/// `function` has the form `fn(dialect_name: str, obj) -> str`. The return
/// value is inserted into SQL as-is (not re-quoted or escaped). It is the
/// caller's responsibility to produce a safe fragment.
#[rapidquery_macros::pyclass(
    "CustomSerializer",
    extends = PySerializer,
    module = "rapidquery._core.data_type"
)]
#[derive(Debug, Default)]
pub struct PyCustomSerializer;

#[pyo3::pymethods]
impl PyCustomSerializer {
    /// Creates a serializer backed by a Python callable.
    ///
    /// Args:
    ///     function: A callable of the form `fn(dialect_name: str, obj) -> str`.
    ///             The return value is inserted into SQL as-is (not re-quoted).
    ///             Must return a `str`.
    #[new]
    fn __new__(function: alias::PyObject) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        unsafe {
            if pyo3::ffi::PyCallable_Check(function.as_ptr()) != 1 {
                return rapidquery_macros::pyerr!(
                    Err(PyTypeError),
                    "the 'function' parameter must be callable"
                );
            }
        }
        let inner = SerializerInner::Custom(function);
        Ok(pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self))
    }

    /// Returns the callable passed to the constructor.
    ///
    /// Returns:
    ///     The function given as `function` to `__new__`.
    #[getter]
    fn get_function(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        match &slf.as_super().0 {
            SerializerInner::Custom(function) => function.clone(),
            _ => unreachable!("CustomSerializer base is always Custom"),
        }
    }
}
