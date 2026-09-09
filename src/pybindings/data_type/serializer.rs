//! Python `Serializer` class hierarchy.
//!
//! Wraps native [`Serializer`](crate::native::Serializer). Python sees frozen
//! subclasses (`IntegerSerializer`, …) plus `ArraySerializer` /
//! `CustomSerializer`.
//!
//! Unit instances compare and hash by variant (`IntegerSerializer() ==
//! IntegerSerializer()`) and `repr` as `IntegerSerializer()`. Array compares
//! by element tree; Custom by function identity.

use std::hash::{Hash, Hasher};

use crate::internal::alias;
use crate::native::data_type::DataTypeGroup;
use crate::native::serializer::Serializer;

fn class_name(serializer: &Serializer) -> &'static str {
    match serializer {
        Serializer::Integer => "IntegerSerializer",
        Serializer::Float => "FloatSerializer",
        Serializer::Boolean => "BooleanSerializer",
        Serializer::String => "StringSerializer",
        Serializer::Uuid => "UuidSerializer",
        Serializer::Bytes => "BytesSerializer",
        Serializer::Bit => "BitSerializer",
        Serializer::Json => "JSONSerializer",
        Serializer::Date => "DateSerializer",
        Serializer::Time => "TimeSerializer",
        Serializer::Datetime => "DatetimeSerializer",
        Serializer::Inet => "InetSerializer",
        Serializer::MacAddress => "MacAddressSerializer",
        Serializer::Unspecified => "UnspecifiedSerializer",
        Serializer::Array(_) => "ArraySerializer",
        Serializer::Custom(_) => "CustomSerializer",
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
pub struct PySerializer(pub(super) Serializer);

impl PySerializer {
    pub(super) fn inferred(kind: DataTypeGroup) -> Self {
        Self(Serializer::from_group(kind))
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
            Serializer::Integer => wrap::<PyIntegerSerializer>(py, self),
            Serializer::Float => wrap::<PyFloatSerializer>(py, self),
            Serializer::Boolean => wrap::<PyBooleanSerializer>(py, self),
            Serializer::String => wrap::<PyStringSerializer>(py, self),
            Serializer::Uuid => wrap::<PyUuidSerializer>(py, self),
            Serializer::Bytes => wrap::<PyBytesSerializer>(py, self),
            Serializer::Bit => wrap::<PyBitSerializer>(py, self),
            Serializer::Json => wrap::<PyJSONSerializer>(py, self),
            Serializer::Date => wrap::<PyDateSerializer>(py, self),
            Serializer::Time => wrap::<PyTimeSerializer>(py, self),
            Serializer::Datetime => wrap::<PyDatetimeSerializer>(py, self),
            Serializer::Inet => wrap::<PyInetSerializer>(py, self),
            Serializer::MacAddress => wrap::<PyMacAddressSerializer>(py, self),
            Serializer::Unspecified => wrap::<PyUnspecifiedSerializer>(py, self),
            Serializer::Array(_) => wrap::<PyArraySerializer>(py, self),
            Serializer::Custom(_) => wrap::<PyCustomSerializer>(py, self),
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
            Serializer::Array(element) => {
                format!("ArraySerializer({}())", class_name(element))
            }
            other => {
                let name = class_name(other);
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
                    pyo3::PyClassInitializer::from(PySerializer(Serializer::$variant))
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
        let inner = Serializer::Array(Box::new(element.0));
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
            Serializer::Array(element) => {
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
        let inner = Serializer::Custom(function);
        Ok(pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self))
    }

    /// Returns the callable passed to the constructor.
    ///
    /// Returns:
    ///     The function given as `function` to `__new__`.
    #[getter]
    fn get_function(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        match &slf.as_super().0 {
            Serializer::Custom(function) => function.clone(),
            _ => unreachable!("CustomSerializer base is always Custom"),
        }
    }
}
