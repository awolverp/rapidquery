//! Serializers

mod functions;
mod inner;

pub use inner::SerializerInner;
pub use inner::SerializerType;

use crate::internal::alias;

pub fn serializer_repr(serializer: &SerializerInner) -> String {
    use std::fmt::Write;

    let name = serializer_type_to_pyname(&serializer.r#type);
    let mut buf = String::with_capacity(name.len() + 2);
    buf.push_str(name);
    buf.push('(');

    let mut sep = "";

    if let SerializerType::Array(element) = &serializer.r#type {
        buf.push_str(&serializer_repr(element));
        sep = ", ";
    }
    if let Some(x) = &serializer.prefix {
        let _ = write!(buf, "{sep}prefix={x:?}");
        sep = ", ";
    }
    if let Some(x) = &serializer.suffix {
        let _ = write!(buf, "{sep}suffix={x:?}");
    }

    buf.push(')');
    buf
}

/// Converts a Python object into a dialect-specific SQL literal string.
#[rapidquery_macros::pyclass("rapidquery.data_type.Serializer", subclass, from_py_object, generic)]
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PySerializer(pub SerializerInner);

#[pyo3::pymethods]
impl PySerializer {
    #[getter]
    fn get_prefix(&self) -> Option<String> {
        self.0.prefix.as_ref().map(arcstr::ArcStr::to_string)
    }

    #[getter]
    fn get_suffix(&self) -> Option<String> {
        self.0.suffix.as_ref().map(arcstr::ArcStr::to_string)
    }

    /// Renders `object` as a SQL literal for `dialect`.
    fn serialize(
        &self,
        dialect: crate::dialect::PySqlDialect,
        object: alias::BoundObject<'_>,
    ) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.write_sql(&dialect, &object, &mut buf)?;
        Ok(buf)
    }

    fn __repr__(&self) -> String {
        serializer_repr(&self.0)
    }

    // Bound<Self> so only instances match.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

macro_rules! impl_unit_serializers {
    (
        $(
            $(#[$outer:meta])*
            $variant:ident => $name:ident => $pyname:literal,
        )+
    ) => {
        $(
            $(#[$outer])*
            #[rapidquery_macros::pyclass("rapidquery.data_type" + $pyname, extends = PySerializer)]
            #[derive(Debug, Default)]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[pyo3(signature = (prefix=None, suffix=None))]
                fn __new__(prefix: Option<&str>, suffix: Option<&str>) -> pyo3::PyClassInitializer<Self> {
                    let inner = SerializerInner::new(
                        SerializerType::$variant,
                        prefix.map(arcstr::ArcStr::from),
                        suffix.map(arcstr::ArcStr::from),
                    );
                    pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self)
                }
            }
        )+

        pub(super) fn serializer_type_to_pyname(serializer_type: &SerializerType) -> &'static str {
            match serializer_type {
                $(
                    SerializerType::$variant {..} => $pyname,
                )+

                SerializerType::Array(_) => "ArraySerializer",
                SerializerType::Custom(_) => "CustomSerializer",
            }
        }

        impl PySerializer {
            /// Wrap `self` as the matching Python subclass.
            pub fn into_subclasses<'py>(
                self,
                py: pyo3::Python<'py>,
            ) -> pyo3::PyResult<alias::BoundObject<'py>> {
                match &self.0.r#type {
                    $(
                        SerializerType::$variant {..} => {
                            pyo3::Bound::new(
                                py,
                                pyo3::PyClassInitializer::from(self).add_subclass($name),
                            )
                            .map(|x| x.into_any())
                        }
                    )+

                    SerializerType::Array(_) => {
                        pyo3::Bound::new(
                            py,
                            pyo3::PyClassInitializer::from(self).add_subclass(PyArraySerializer),
                        )
                        .map(|x| x.into_any())
                    },
                    SerializerType::Custom(_) => {
                        pyo3::Bound::new(
                            py,
                            pyo3::PyClassInitializer::from(self).add_subclass(PyCustomSerializer),
                        )
                        .map(|x| x.into_any())
                    }
                }
            }
        }
    };
}

impl_unit_serializers! {
    /// Serializes Python integers as bare SQL integer literals.
    /// Accepts Python `int` of any width (including values beyond 64-bit).
    /// Rejects `bool`; `True` and `False` are not treated as integers.
    ///
    /// Raises:
    ///    TypeError: If the value is not an `int`.
    Integer => PyIntegerSerializer => "IntegerSerializer",

    /// Serializes numeric values as bare SQL numeric literals.
    /// Accepts `float`, `int`, and `decimal.Decimal`. Finite floats are formatted
    /// so that `1.0` stays distinct from `1`. `str` is not accepted.
    ///
    /// Raises:
    ///     TypeError: If the value is not a `float`, `int`, or `decimal.Decimal`.
    ///     ValueError: If the value is a non-finite float (`nan`, `inf`, `-inf`).
    Float => PyFloatSerializer => "FloatSerializer",

    /// Serializes Python booleans as dialect boolean literals.
    /// Accepts Python `bool` only. The dialect chooses the spelling
    /// (`TRUE`/`FALSE` or `1`/`0`).
    ///
    /// Raises:
    ///     TypeError: If the value is not a `bool`.
    Boolean => PyBooleanSerializer => "BooleanSerializer",

    /// Serializes text as a quoted, escaped SQL string literal.
    /// Accepts `str`, or an `enum.Enum` member whose `.value` is a `str`. The
    /// dialect supplies quoting and escaping.
    ///
    /// Raises:
    ///     TypeError: If the value is not a `str` or a string-valued enum member.
    String => PyStringSerializer => "StringSerializer",

    /// Serializes UUIDs as quoted SQL string literals.
    /// Accepts `uuid.UUID` or `str`.
    /// Raises:
    ///     TypeError: If the value is not a `uuid.UUID` or `str`.
    Uuid => PyUuidSerializer => "UuidSerializer",

    /// Serializes binary data as a dialect blob/bytea literal.
    /// Accepts exact `bytes` (not `bytearray` or `memoryview`).
    ///
    /// Raises:
    ///     TypeError: If the value is not `bytes`.
    Bytes => PyBytesSerializer => "BytesSerializer",

    /// Serializes bit strings as dialect bit-string literals.
    /// Accepts a `str` consisting only of `'0'` and `'1'` characters.
    /// Supported on PostgreSQL, DuckDB, MySQL, and Teradata.
    ///
    /// Raises:
    ///     TypeError: If the value is not a `str`.
    ///     ValueError: If the string is empty or contains characters other than `'0'` and `'1'`.
    ///     NotImplementedError: If the dialect has no native bit-string literal.
    Bit => PyBitSerializer => "BitSerializer",

    /// Serializes a JSON-encodable object as a quoted SQL string literal.
    /// Accepts any object that the JSON encoder can dump. Uses `orjson` when it
    /// is installed, otherwise the stdlib `json` module. The JSON text is then
    /// wrapped with the dialect's string quoting. With `orjson`, datetime,
    /// UUID, enum.StrEnum, and dataclass are encoded natively; stdlib
    /// `json` rejects those.
    ///
    /// Raises:
    ///     TypeError: If the value cannot be JSON-encoded.
    Json => PyJSONSerializer => "JSONSerializer",

    /// Serializes dates as quoted `YYYY-MM-DD` literals.
    /// Accepts exact `datetime.date` (not `datetime.datetime`).
    ///
    /// Raises:
    ///     TypeError: If the value is not a `datetime.date`.
    Date => PyDateSerializer => "DateSerializer",

    /// Serializes times as quoted `HH:MM:SS[.ffffff]` literals.
    /// Accepts `datetime.time`.
    ///
    /// Raises:
    ///     TypeError: If the value is not a `datetime.time`.
    Time => PyTimeSerializer => "TimeSerializer",

    /// Serializes timestamps as quoted SQL timestamp literals.
    /// Accepts naive or aware `datetime.datetime`, or `int`/`float` UTC epoch
    /// seconds. Naive values keep wall time. Aware values keep their original
    /// offset (they are not converted to UTC).
    ///
    /// Raises:
    ///     TypeError: If the value is not a `datetime.datetime`, `int`, or `float`.
    ///     OverflowError: If an epoch value is out of range for a timestamp.
    Datetime => PyDatetimeSerializer => "DatetimeSerializer",

    /// Serializes IP addresses and networks as quoted SQL literals.
    /// Accepts `str` or `ipaddress` objects (`IPv4Address`, `IPv6Address`,
    /// `IPv4Network`, `IPv6Network`, and related types whose `str()` is a valid
    /// address or CIDR). Supported on PostgreSQL and DuckDB.
    ///
    /// Raises:
    ///     ValueError: If the value is not a valid IPv4/IPv6 address or network.
    ///     NotImplementedError: If the dialect has no native INET type.
    Inet => PyInetSerializer => "InetSerializer",

    /// Serializes MAC addresses as quoted SQL literals.
    /// Accepts a `str` using `:` or `-` as the separator.
    /// Supported on PostgreSQL.
    ///
    /// Raises:
    ///     ValueError: If the value is not a valid MAC address.
    ///     NotImplementedError: If the dialect has no native MACADDR type.
    MacAddress => PyMacAddressSerializer => "MacAddressSerializer",

    /// Inserts `str(obj)` into SQL with no quoting or type check.
    /// Used when no serializer could be inferred from the type name. The result
    /// is not a safe SQL value; prefer an explicit serializer.
    Unspecified => PyUnspecifiedSerializer => "UnspecifiedSerializer",
}

/// Serializes sequences as `ARRAY[elem, ...]` literals.
///
/// Each element is serialized independently with `element`. Accepts `list`
/// or `tuple`. Nested arrays are expressed by wrapping another
/// `ArraySerializer`. Supported on PostgreSQL, DuckDB, and BigQuery.
///
/// Raises:
///     TypeError: If the value is not a `list` or `tuple`.
///     NotImplementedError: If the dialect has no `ARRAY[…]` constructor.
#[rapidquery_macros::pyclass("rapidquery.data_type.ArraySerializer", extends = PySerializer)]
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
    #[pyo3(signature = (element, prefix=None, suffix=None))]
    fn __new__(
        element: PySerializer,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> pyo3::PyClassInitializer<Self> {
        let type_ = SerializerType::Array(Box::new(element.0));
        let inner = SerializerInner::new(type_, prefix.map(Into::into), suffix.map(Into::into));

        pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self)
    }

    /// Returns the serializer used for each array element.
    ///
    /// Returns:
    ///     The `Serializer` passed to the constructor.
    #[getter]
    fn get_element(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let base = slf.as_super();
        match &base.0.r#type {
            SerializerType::Array(element) => {
                PySerializer((**element).clone()).into_subclasses(slf.py())
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

/// Serializes values by calling a user-supplied Python function.
///
/// `function` has the form `fn(dialect_name: str, obj) -> str`. The return
/// value is inserted into SQL as-is (not re-quoted or escaped). It is the
/// caller's responsibility to produce a safe fragment.
#[rapidquery_macros::pyclass("rapidquery.data_type.CustomSerializer", extends = PySerializer)]
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
    #[pyo3(signature = (function, prefix=None, suffix=None))]
    fn __new__(
        function: alias::PyObject,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        unsafe {
            if pyo3::ffi::PyCallable_Check(function.as_ptr()) != 1 {
                return rapidquery_macros::pyerr!(
                    Err(PyTypeError),
                    "the 'function' parameter must be callable"
                );
            }
        }
        let type_ = SerializerType::Custom(function);
        let inner = SerializerInner::new(type_, prefix.map(Into::into), suffix.map(Into::into));

        Ok(pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self))
    }

    /// Returns the callable passed to the constructor.
    ///
    /// Returns:
    ///     The function given as `function` to `__new__`.
    #[getter]
    fn get_function(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        let base = slf.as_super();
        match &base.0.r#type {
            SerializerType::Custom(function) => function.clone(),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

/// `Serializer` instance, or a `DataType` (its serializer only).
///
/// Use [`PySerializer`] itself as an argument if you wanna accept ONLY `PySerializer` types.
pub struct SerializerExtractor(pub SerializerInner);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for SerializerExtractor {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();

        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
            // interned in `initialize_typeref` before any extractor runs.
            if crate::typeref::check(ptr, crate::typeref::SERIALIZER_TYPE) {
                let serializer = obj.cast_unchecked::<PySerializer>();
                return Ok(Self(serializer.get().0.clone()));
            }

            if crate::typeref::check_exact(ptr, crate::typeref::DATATYPE_TYPE) {
                let data_type = obj.cast_unchecked::<crate::data_type::PyDataType>();
                return Ok(Self(data_type.get().serializer.clone()));
            }
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected Serializer or DataType, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
        )
    }
}
