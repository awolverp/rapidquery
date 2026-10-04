//! SQL data types: interned names and [`DataType`].
//!
//! A [`DataTypeName`] is a canonical SQL spelling. Known names are interned.
//! Serializing a Python value is [`crate::serializers`], not this module.

mod name;

#[cfg(test)]
mod tests;

pub use name::DataTypeName;

use crate::internal::repr::ReprFormatter;
use crate::internal::utils;

/// Represents a SQL data type.
///
/// A data type is a SQL type name. The name can be any string; it is not
/// limited to a set of predefined values.
///
/// Known names are interned and canonicalized (`"int"` becomes `"INT"`).
/// Unknown names and parameterized spellings (`VARCHAR(255)`) are kept as
/// given. Interning applies to the **name string**, not to `DataType`
/// instances: `DataType("int")` is not a singleton.
///
/// [`WriteSql`] emits the type name (CAST, column definitions). A Python
/// value is serialized by a `rapidquery.serializers` serializer on `Value`.
#[rapidquery_macros::pyclass("rapidquery.data_type.DataType", from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDataType {
    pub name: DataTypeName,
}

impl PyDataType {
    #[inline]
    pub fn from_name(name: impl Into<DataTypeName>) -> Self {
        Self { name: name.into() }
    }
}

impl crate::sql::WriteSql for PyDataType {
    #[inline]
    fn write_sql<W: crate::sql::SqlWriter>(
        &self,
        dialect: &crate::dialect::PyDialect,
        out: &mut W,
    ) -> crate::sql::PrepareResult<()> {
        self.name.write_sql(dialect, out)
    }
}

#[pyo3::pymethods]
impl PyDataType {
    /// Creates a new SQL data type.
    ///
    /// Args:
    ///     name: The SQL type spelling. Not limited to a predefined set.
    ///             Known names are interned and canonicalized (`"int"`
    ///             becomes `"INT"`). Unknown names and parameterized
    ///             spellings (`VARCHAR(255)`) are kept as given.
    #[new]
    #[pyo3(signature = (name, /))]
    fn __new__(name: String) -> Self {
        PyDataType::from_name(name)
    }

    /// Returns the type name.
    ///
    /// Returns:
    ///     The canonical type name. For interned names this may differ in
    ///     casing or spacing from the string passed to the constructor.
    #[getter]
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn __repr__(&self) -> String {
        let mut fmt = ReprFormatter::new_call("DataType");
        fmt.positional_debug(self.name.as_str());
        fmt.into_string()
    }
}

/// An extractor for DataTypeName, which accepts `str` and `DataType`
pub struct DataTypeNameExtractor(pub DataTypeName);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for DataTypeNameExtractor {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();

        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
            // interned in `initialize_typeref` before any extractor runs.
            if crate::typeref::check(ptr, crate::typeref::DATATYPE_TYPE) {
                let dt = obj.cast_unchecked::<PyDataType>();
                return Ok(Self(dt.get().name.clone()));
            }

            if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
                let borrowed = utils::borrow_py_str_buffer(obj.py(), ptr)?;
                return Ok(Self(DataTypeName::from(borrowed)));
            }
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected str or DataType, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
        )
    }
}

#[pyo3::pymodule(name = "data_type", gil_used = false)]
pub mod data_type_mod {
    #[pymodule_export]
    use super::PyDataType;
}
