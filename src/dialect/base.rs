use std::hash::{Hash, Hasher};

use crate::internal::utils;

use super::convert;
use super::inner::Dialect;

/// Abstract SQL dialect.
///
/// Concrete dialects are frozen subclasses (`PostgreSql`, `DuckDb`, ...).
/// Pass an instance wherever a dialect is required.
/// This base class cannot be instantiated.
///
/// Instances of the same dialect compare equal and hash together.
#[rapidquery_macros::pyclass(
    "SqlDialect",
    subclass,
    skip_from_py_object,
    module = "rapidquery._core.dialect"
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySqlDialect(pub Dialect);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PySqlDialect {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `obj` is a borrowed `&Bound<PyAny>`, so `as_ptr()` is a
            // live `PyObject*`.
            if crate::typeref::check(obj.as_ptr(), crate::typeref::SQL_DIALECT_TYPE) {
                let cast = obj.cast_unchecked::<Self>();
                return Ok(Self(cast.get().0));
            }

            rapidquery_macros::pyerr!(
                Err(PyTypeError),
                "expected a SqlDialect, got {}",
                utils::get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

fn collect(write: impl FnOnce(&mut String) -> std::fmt::Result) -> String {
    let mut buf = String::new();
    let _ = write(&mut buf);
    buf
}

#[pyo3::pymethods]
impl PySqlDialect {
    #[getter]
    fn get_positional_parameter(&self) -> String {
        convert::positional_parameter_name(self.0.positional_parameter)
    }

    #[getter]
    fn get_bit_format(&self) -> Option<&'static str> {
        self.0.bit_format.map(convert::bit_format_name)
    }

    #[getter]
    fn get_supports_array(&self) -> bool {
        self.0.supports_array
    }

    #[getter]
    fn get_supports_inet(&self) -> bool {
        self.0.supports_inet
    }

    #[getter]
    fn get_supports_mac_address(&self) -> bool {
        self.0.supports_mac_address
    }

    #[getter]
    fn get_null_treatment_syntax(&self) -> &'static str {
        convert::null_treatment_syntax_name(self.0.null_treatment_syntax)
    }

    #[getter]
    fn get_like_any_syntax(&self) -> &'static str {
        convert::like_any_syntax_name(self.0.like_any_syntax)
    }

    #[getter]
    fn get_bool_format(&self) -> &'static str {
        convert::bool_format_name(self.0.bool_format)
    }

    #[getter]
    fn get_identifier_format(&self) -> &'static str {
        convert::identifier_format_name(self.0.identifier_format)
    }

    #[getter]
    fn get_identifier_escape(&self) -> &'static str {
        convert::identifier_escape_name(self.0.identifier_escape)
    }

    #[getter]
    fn get_string_format(&self) -> &'static str {
        convert::string_format_name(self.0.string_format)
    }

    #[getter]
    fn get_string_escape(&self) -> &'static str {
        convert::string_escape_name(self.0.string_escape)
    }

    #[getter]
    fn get_bytes_format(&self) -> &'static str {
        convert::bytes_format_name(self.0.bytes_format)
    }

    fn format_boolean(&self, value: bool) -> String {
        collect(|buf| self.0.format_boolean(buf, value))
    }

    fn escape_string(&self, string: &str) -> String {
        collect(|buf| self.0.escape_string(buf, string))
    }

    fn quote_identifier(&self, ident: &str) -> String {
        collect(|buf| self.0.quote_identifier(buf, ident))
    }

    fn write_string_literal(&self, raw: &str) -> String {
        collect(|buf| self.0.write_string_literal(buf, raw))
    }

    fn format_bits(&self, bits: &str) -> pyo3::PyResult<String> {
        if self.0.bit_format.is_none() {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "this dialect does not support literal bit values"
            );
        }
        Ok(collect(|buf| self.0.format_bits(buf, bits)))
    }

    fn format_bytes(&self, data: &[u8]) -> String {
        collect(|buf| self.0.format_bytes(buf, data))
    }

    // Bound<Self> so only instances match; the class object is a type.
    // PostgreSql() == PostgreSql is false; PostgreSql() == PostgreSql() is true.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::default();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}
