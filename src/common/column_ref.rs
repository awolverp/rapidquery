use std::str::FromStr;

use pyo3::types::{PyAnyMethods, PyStringMethods};
use sea_query::IntoIden;

use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundKwargs, RefBoundObject};

crate::implement_pyclass! {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    /// Represents a reference to a database column with optional table and schema qualification.
    ///
    /// This class is used to uniquely identify columns in SQL queries, supporting
    /// schema-qualified and table-qualified column references.
    ///
    /// NOTE: this class is immutable and frozen.
    #[derive(Debug, Clone)]
    [] PyColumnRef as "ColumnRef" {
        /// Name of the referenced column. [`Option::None`] means '*'.
        pub name: Option<sea_query::DynIden>,
        pub table: Option<sea_query::DynIden>,
        pub schema: Option<sea_query::DynIden>,
    }
}

impl sea_query::IntoColumnRef for PyColumnRef {
    fn into_column_ref(self) -> sea_query::ColumnRef {
        if let Some(name) = self.name {
            match (self.table, self.schema) {
                (Some(table), Some(schema)) => {
                    sea_query::ColumnRef::SchemaTableColumn(schema, table, name)
                }
                (Some(table), None) => sea_query::ColumnRef::TableColumn(table, name),
                _ => sea_query::ColumnRef::Column(name),
            }
        } else if let Some(table) = self.table {
            sea_query::ColumnRef::TableAsterisk(table)
        } else {
            sea_query::ColumnRef::Asterisk
        }
    }
}

impl FromStr for PyColumnRef {
    type Err = pyo3::PyErr;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let string = string.trim().to_owned();

        if string.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "cannot parse an empty string",
            ));
        }

        // Possible formats:
        //    name
        //    table.name
        //    schema.table.name
        let mut string: Vec<String> = string.split('.').map(String::from).collect();

        if string.len() > 3 {
            return Err(pyo3::exceptions::PyValueError::new_err("invalid format"));
        }

        let name = string.pop().unwrap();
        let table = string.pop().map(|x| sea_query::Alias::new(x).into_iden());
        let schema = string.pop().map(|x| sea_query::Alias::new(x).into_iden());

        Ok(Self {
            name: if name == "*" {
                None
            } else {
                Some(sea_query::Alias::new(name).into_iden())
            },
            table,
            schema,
        })
    }
}

impl TryFrom<RefBoundObject<'_>> for PyColumnRef {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject<'_>) -> Result<Self, Self::Error> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            if let Ok(x) = value.cast_exact::<pyo3::types::PyString>() {
                return Self::from_str(x.to_str()?);
            }

            if let Some(result) = Self::try_from_property(value)? {
                return Ok(result);
            }

            crate::new_error!(
                PyTypeError,
                "expected ColumnRef, str, or object.to_column_ref property, got {}",
                crate::internal::get_type_name(value.py(), value.as_ptr())
            )
        }
    }
}

impl PyColumnRef {
    #[inline]
    pub fn try_from_property(value: RefBoundObject) -> pyo3::PyResult<Option<Self>> {
        const PROPERTY_NAME: &std::ffi::CStr = c"__column_ref__";

        let property = match value.getattr(PROPERTY_NAME) {
            Ok(x) => Ok(Some(x)),
            Err(err) if err.is_instance_of::<pyo3::exceptions::PyAttributeError>(value.py()) => {
                Ok(None)
            }
            Err(err) => Err(err),
        };
        let property = property?;

        if property.is_none() {
            return Ok(None);
        }

        unsafe {
            let property = property.unwrap_unchecked();

            if pyo3::ffi::Py_TYPE(property.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = property.cast_unchecked::<Self>();
                return Ok(Some(casted_value.get().clone()));
            }

            if let Ok(x) = property.extract::<String>() {
                return Self::from_str(&x).map(Some);
            }

            crate::new_error!(
                PyTypeError,
                "__column_ref__ property returns something other than ColumnRef or str; returns {}",
                crate::internal::get_type_name(property.py(), property.as_ptr())
            )
        }
    }
}

#[pyo3::pymethods]
impl PyColumnRef {
    #[new]
    #[pyo3(signature=(name, table=None, schema=None))]
    fn __new__(
        name: String,
        table: Option<String>,
        schema: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let name = unsafe {
            if name == "*" {
                None
            } else {
                Some(name)
            }
        };

        Ok(Self {
            name: name.map(|x| sea_query::Alias::new(x).into_iden()),
            table: table.map(|x| sea_query::Alias::new(x).into_iden()),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
        })
    }

    #[getter]
    fn name(&self) -> String {
        match &self.name {
            None => String::from("*"),
            Some(x) => x.to_string(),
        }
    }

    #[getter]
    fn table(&self) -> Option<String> {
        self.table.as_ref().map(|x| x.to_string())
    }

    #[getter]
    fn schema(&self) -> Option<String> {
        self.schema.as_ref().map(|x| x.to_string())
    }

    /// Parse a string representation of a column reference.
    ///
    /// Supports formats like:
    /// - "column_name"
    /// - "table.column_name"
    /// - "schema.table.column_name"
    #[classmethod]
    fn parse(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, string: String) -> pyo3::PyResult<Self> {
        Self::from_str(&string)
    }

    #[pyo3(signature=(**kwds))]
    fn copy_with(&self, kwds: Option<BoundKwargs<'_>>) -> pyo3::PyResult<Self> {
        use pyo3::types::PyDictMethods;

        let mut cloned = self.clone();
        if kwds.is_none() {
            return Ok(cloned);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };

        for (key, val) in kwds.iter() {
            unsafe {
                let key = key.extract::<String>().unwrap_unchecked();

                let val: Option<String> = unsafe {
                    if pyo3::ffi::Py_IsNone(val.as_ptr()) == 1 {
                        None
                    } else if pyo3::ffi::PyUnicode_CheckExact(val.as_ptr()) == 1 {
                        Some(val.extract::<String>().unwrap_unchecked())
                    } else {
                        return crate::new_error!(
                            PyTypeError,
                            "expected str or None, got {}",
                            crate::internal::get_type_name(val.py(), val.as_ptr())
                        );
                    }
                };

                if key == "name" {
                    if val.is_none() {
                        return crate::new_error!(PyTypeError, "expected str for name, got None");
                    }

                    let val_str = unsafe { val.unwrap_unchecked() };
                    if val_str == "*" {
                        cloned.name = None;
                    } else {
                        cloned.name = Some(sea_query::Alias::new(val_str).into_iden());
                    }
                } else if key == "table" {
                    cloned.table = val.map(|x| sea_query::Alias::new(x).into_iden());
                } else if key == "schema" {
                    cloned.schema = val.map(|x| sea_query::Alias::new(x).into_iden());
                } else {
                    return crate::new_error!(
                        PyTypeError,
                        "got an unexpected keyword argument '{}'",
                        key
                    );
                }
            }
        }

        Ok(cloned)
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        slf.name == other.name && slf.schema == other.schema && slf.table == other.table
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return false;
        }

        slf.name != other.name || slf.schema != other.schema || slf.table != other.table
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut state = std::hash::DefaultHasher::new();

        if let Some(x) = &self.name {
            x.to_string().hash(&mut state);
        }
        if let Some(x) = &self.table {
            x.to_string().hash(&mut state);
        }
        if let Some(x) = &self.schema {
            x.to_string().hash(&mut state);
        }

        state.finish()
    }

    /// Shorthand for `Expr(self)`
    fn to_expr(&self) -> super::expression::PyExpr {
        super::expression::PyExpr(sea_query::Expr::column(self.clone()))
    }

    pub fn __repr__(&self) -> String {
        let mut fmt = ReprFormatter::new("ColumnRef");

        match &self.name {
            Some(x) => {
                fmt.iden("name", x);
            }
            None => {
                fmt.pair("name", "*");
            }
        }
        fmt.optional_iden("table", self.table.as_ref());
        fmt.optional_iden("schema", self.schema.as_ref());

        fmt.finish()
    }
}
