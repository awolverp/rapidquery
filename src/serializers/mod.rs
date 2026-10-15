//! Serializers

mod functions;
mod inner;

pub use inner::SerializerType;

use crate::internal::alias;
use crate::internal::repr;

pub fn serializer_repr(serializer: &SerializerType) -> repr::ReprFormatter {
    let mut fmt = repr::ReprFormatter::new_call(serializer_type_to_pyname(serializer));

    if let SerializerType::Array(element) = serializer {
        fmt.positional(serializer_repr(element));
    }

    fmt.close();
    fmt
}

/// Turns a Python object into a SQL literal for one dialect.
#[rapidquery_macros::pyclass("rapidquery.serializers.Serializer", subclass, from_py_object, generic)]
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PySerializer(pub SerializerType);

#[pyo3::pymethods]
impl PySerializer {
    /// Renders `object` as a SQL literal for `dialect`.
    fn serialize(&self, dialect: crate::dialect::PyDialect, object: alias::BoundObject<'_>) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.serialize(&dialect, &object, &mut buf)?;
        Ok(buf)
    }

    fn __repr__(&self) -> String {
        serializer_repr(&self.0).into_string()
    }

    // Bound<Self> so only instances match.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;

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
            #[rapidquery_macros::pyclass("rapidquery.serializers" + $pyname, extends = PySerializer)]
            #[derive(Debug, Default)]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                #[new]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(PySerializer(SerializerType::$variant))
                        .add_subclass(Self)
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
                match &self.0 {
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
    /// Bare SQL integer. Accepts `int`, not `bool`.
    Integer => PyIntegerSerializer => "IntegerSerializer",

    /// Bare SQL number for `float`, `int`, or `Decimal`.
    Float => PyFloatSerializer => "FloatSerializer",

    /// Dialect boolean literal for a Python `bool`.
    Boolean => PyBooleanSerializer => "BooleanSerializer",

    /// Quoted SQL string for `str` or a string enum member.
    String => PyStringSerializer => "StringSerializer",

    /// UUID literal for `uuid.UUID` or `str`.
    Uuid => PyUuidSerializer => "UuidSerializer",

    /// Blob or bytea literal for exact `bytes`.
    Bytes => PyBytesSerializer => "BytesSerializer",

    /// Bit-string literal for a non-empty `0`/`1` string.
    Bit => PyBitSerializer => "BitSerializer",

    /// JSON literal for an object the JSON encoder can dump.
    Json => PyJSONSerializer => "JSONSerializer",

    /// Date literal for `datetime.date`, not `datetime.datetime`.
    Date => PyDateSerializer => "DateSerializer",

    /// Time literal for `datetime.time`.
    Time => PyTimeSerializer => "TimeSerializer",

    /// Timestamp literal for `datetime` or a UTC epoch number.
    Datetime => PyDatetimeSerializer => "DatetimeSerializer",

    /// Inet literal for an IP address, network, or address string.
    Inet => PyInetSerializer => "InetSerializer",

    /// MAC address literal for a `:` or `-` separated string.
    MacAddress => PyMacAddressSerializer => "MacAddressSerializer",

    /// Inserts `str(object)` with no quoting and no type check.
    Unspecified => PyUnspecifiedSerializer => "UnspecifiedSerializer",
}

/// `ARRAY[…]` serializer. Each item is rendered by the element serializer.
#[rapidquery_macros::pyclass("rapidquery.serializers.ArraySerializer", extends = PySerializer)]
#[derive(Debug, Default)]
pub struct PyArraySerializer;

#[pyo3::pymethods]
impl PyArraySerializer {
    /// Build an array serializer from the serializer used for each item.
    #[new]
    #[inline]
    fn __new__(element: PySerializer) -> pyo3::PyClassInitializer<Self> {
        let inner = SerializerType::Array(Box::new(element.0));
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
            SerializerType::Array(element) => PySerializer((**element).clone()).into_subclasses(slf.py()),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

/// Serializer that inserts the `str` returned by a callable. The text is not escaped.
#[rapidquery_macros::pyclass("rapidquery.serializers.CustomSerializer", extends = PySerializer)]
#[derive(Debug, Default)]
pub struct PyCustomSerializer;

#[pyo3::pymethods]
impl PyCustomSerializer {
    /// Build a serializer. `function(dialect, object)` must return `str`.
    #[new]
    fn __new__(function: alias::PyObject) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        unsafe {
            if pyo3::ffi::PyCallable_Check(function.as_ptr()) != 1 {
                return rapidquery_macros::pyerr!(Err(PyTypeError), "the 'function' parameter must be callable");
            }
        }
        let inner = SerializerType::Custom(function);
        Ok(pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self))
    }

    /// The callable passed to the constructor.
    #[getter]
    fn get_function(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        let base = slf.as_super();
        match &base.0 {
            SerializerType::Custom(function) => function.clone(),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

/// Python `Serializer` extracted into a [`SerializerType`].
///
/// Use this as a `FromPyObject` argument when a method accepts only serializer
/// instances. Other objects fail extraction.
///
/// ```
/// use rapidquery::serializers::SerializerExtractor;
/// use rapidquery::serializers::SerializerType;
///
/// let extracted = SerializerExtractor(SerializerType::Integer);
/// assert!(matches!(extracted.0, SerializerType::Integer));
/// ```
pub struct SerializerExtractor(pub SerializerType);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for SerializerExtractor {
    type Error = pyo3::PyErr;

    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();

        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
            // interned in `initialize_typeref` before any extractor runs.
            if crate::typeref::check(ptr, crate::typeref::SERIALIZER_TYPE) {
                let serializer = obj.cast_unchecked::<PySerializer>();
                return Ok(Self(serializer.get().0.clone()));
            }
        }

        rapidquery_macros::pyerr!(Err(PyTypeError), "expected Serializer, got {}", unsafe {
            crate::internal::utils::get_type_name(obj.py(), obj.as_ptr())
        })
    }
}

#[pyo3::pymodule(name = "serializers", gil_used = false)]
pub mod serializers_mod {
    #[pymodule_export]
    use super::PyArraySerializer;
    #[pymodule_export]
    use super::PyBitSerializer;
    #[pymodule_export]
    use super::PyBooleanSerializer;
    #[pymodule_export]
    use super::PyBytesSerializer;
    #[pymodule_export]
    use super::PyCustomSerializer;
    #[pymodule_export]
    use super::PyDateSerializer;
    #[pymodule_export]
    use super::PyDatetimeSerializer;
    #[pymodule_export]
    use super::PyFloatSerializer;
    #[pymodule_export]
    use super::PyInetSerializer;
    #[pymodule_export]
    use super::PyIntegerSerializer;
    #[pymodule_export]
    use super::PyJSONSerializer;
    #[pymodule_export]
    use super::PyMacAddressSerializer;
    #[pymodule_export]
    use super::PySerializer;
    #[pymodule_export]
    use super::PyStringSerializer;
    #[pymodule_export]
    use super::PyTimeSerializer;
    #[pymodule_export]
    use super::PyUnspecifiedSerializer;
    #[pymodule_export]
    use super::PyUuidSerializer;
}
