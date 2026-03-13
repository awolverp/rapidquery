use pyo3::types::PyAnyMethods;
use sea_query::IntoIden;
use std::str::FromStr;

use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundKwargs, RefBoundObject};

crate::implement_pyclass! {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    /// Represents a table name reference with optional schema, database, and alias.
    ///
    /// This class encapsulates a table name that can include:
    /// - The base table name
    /// - Optional schema/namespace qualification
    /// - Optional database qualification (for systems that support it)
    ///
    /// The class provides parsing capabilities for string representations
    /// and supports comparison operations.
    ///
    /// NOTE: this class is immutable and frozen.
    ///
    /// @signature (
    ///     name: str,
    ///     schema: str | None = None,
    ///     database: str | None = None,
    ///     alias: str | None = None,
    /// )
    #[derive(Debug, Clone)]
    [] PyTableName as "TableName" {
        pub name: sea_query::DynIden,
        pub schema: Option<sea_query::DynIden>,
        pub database: Option<sea_query::DynIden>,
        pub alias: Option<sea_query::DynIden>,
    }
}

impl sea_query::IntoTableRef for PyTableName {
    fn into_table_ref(self) -> sea_query::TableRef {
        match (self.schema, self.database, self.alias) {
            (Some(schema), Some(database), Some(alias)) => {
                sea_query::TableRef::DatabaseSchemaTableAlias(database, schema, self.name, alias)
            }
            (Some(schema), None, Some(alias)) => {
                sea_query::TableRef::SchemaTableAlias(schema, self.name, alias)
            }
            (Some(schema), Some(database), None) => {
                sea_query::TableRef::DatabaseSchemaTable(database, schema, self.name)
            }
            (Some(schema), None, None) => sea_query::TableRef::SchemaTable(schema, self.name),
            (None, None, Some(alias)) => sea_query::TableRef::TableAlias(self.name, alias),
            _ => sea_query::TableRef::Table(self.name),
        }
    }
}

impl TryFrom<RefBoundObject<'_>> for PyTableName {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject<'_>) -> Result<Self, Self::Error> {
        const PROPERTY_NAME: &std::ffi::CStr = c"__table_name__";

        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::TABLE_TYPE) == 1 {
                let casted_value = value.cast_unchecked::<crate::schema::table::PyTable>();
                return Ok(casted_value.get().0.lock().name.clone());
            };

            if let Ok(x) = value.extract::<String>() {
                return Self::from_str(&x);
            }

            if value.hasattr(PROPERTY_NAME)? {
                let property = value.getattr(PROPERTY_NAME)?;

                if pyo3::ffi::Py_TYPE(property.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                    let casted_property = property.cast_unchecked::<Self>();
                    return Ok(casted_property.get().clone());
                }

                if pyo3::ffi::PyObject_TypeCheck(property.as_ptr(), crate::typeref::TABLE_TYPE) == 1
                {
                    let casted_property =
                        property.cast_unchecked::<crate::schema::table::PyTable>();

                    return Ok(casted_property.get().0.lock().name.clone());
                };

                if let Ok(x) = property.extract::<String>() {
                    return Self::from_str(&x);
                }

                return crate::new_error!(
                    PyTypeError,
                    "__table_name__ property returns something other than TableName or Table or \
                     str; returns {}",
                    crate::internal::get_type_name(property.py(), property.as_ptr())
                );
            }

            crate::new_error!(
                PyTypeError,
                "expected TableName or Table or str or object.__table_name__ property, got {}",
                crate::internal::get_type_name(value.py(), value.as_ptr())
            )
        }
    }
}

impl FromStr for PyTableName {
    type Err = pyo3::PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "cannot parse an empty string",
            ));
        }

        // Possible formats:
        //    name
        //    schema.name
        //    database.schema.name
        let mut s: Vec<String> = s.split('.').map(String::from).collect();

        if s.len() > 3 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "invalid format",
            ));
        }

        let name = s
            .pop()
            .map(|x| sea_query::Alias::new(x).into_iden())
            .unwrap();

        let schema = s.pop().map(|x| sea_query::Alias::new(x).into_iden());
        let database = s.pop().map(|x| sea_query::Alias::new(x).into_iden());

        Ok(Self {
            name,
            schema,
            database,
            alias: None,
        })
    }
}

#[pyo3::pymethods]
impl PyTableName {
    #[new]
    #[pyo3(signature=(name, schema=None, database=None, alias=None))]
    fn new(
        name: String,
        schema: Option<String>,
        database: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self {
            name: sea_query::Alias::new(name).into_iden(),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
            database: database.map(|x| sea_query::Alias::new(x).into_iden()),
            alias: alias.map(|x| sea_query::Alias::new(x).into_iden()),
        }
    }

    /// Parse a string representation of a table name.
    ///
    /// Supports formats like:
    /// - "table_name"
    /// - "schema.table_name"
    /// - "database.schema.table_name"
    ///
    /// @signature (cls, string: str) -> typing.Self
    #[classmethod]
    fn parse(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, string: String) -> pyo3::PyResult<Self> {
        Self::from_str(&string)
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    /// @signature (self) -> str | None
    #[getter]
    fn schema(&self) -> Option<String> {
        self.schema.as_ref().map(|x| x.to_string())
    }

    /// @signature (self) -> str | None
    #[getter]
    fn database(&self) -> Option<String> {
        self.database.as_ref().map(|x| x.to_string())
    }

    /// @signature (self) -> str | None
    #[getter]
    fn alias(&self) -> Option<String> {
        self.alias.as_ref().map(|x| x.to_string())
    }

    /// Create a shallow copy of this TableName.
    ///
    /// @signature (
    ///     self,
    ///     *,
    ///     name: str = ...,
    ///     schema: str | None = ...,
    ///     database: str | None = ...,
    ///     alias: str | None = ...,
    /// ) -> typing.Self
    #[pyo3(signature=(**kwds))]
    fn copy_with(&self, kwds: Option<BoundKwargs<'_>>) -> pyo3::PyResult<Self> {
        use pyo3::types::PyDictMethods;

        let mut cloned = self.clone();
        if kwds.is_none() {
            return Ok(cloned);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };

        for (key, val) in kwds.iter() {
            #[cfg(debug_assertions)]
            let key = key.extract::<String>().unwrap();

            #[cfg(not(debug_assertions))]
            let key = unsafe { key.extract::<String>().unwrap_unchecked() };

            // All of values are Option<string>
            let val = unsafe {
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
                if let Some(x) = val {
                    // Ignore name=None
                    cloned.name = sea_query::Alias::new(x).into_iden();
                }
            } else if key == "database" {
                cloned.database = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else if key == "schema" {
                cloned.schema = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else if key == "alias" {
                cloned.alias = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else {
                return crate::new_error!(
                    PyTypeError,
                    "got an unexpected keyword argument '{}'",
                    key
                );
            }
        }

        Ok(cloned)
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let other = other.get();
        Ok(slf.name == other.name && slf.database == other.database && slf.schema == other.schema)
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let other = other.get();
        Ok(slf.name != other.name || slf.database != other.database || slf.schema != other.schema)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    pub fn __repr__(&self) -> String {
        ReprFormatter::new("TableName")
            .iden("name", &self.name)
            .optional_iden("schema", self.schema.as_ref())
            .optional_iden("database", self.database.as_ref())
            .optional_iden("alias", self.alias.as_ref())
            .finish()
    }
}
