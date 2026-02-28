use std::str::FromStr;

use pyo3::types::{PyAnyMethods, PyStringMethods};
use sea_query::IntoIden;

#[derive(Debug, Default)]
pub enum ReturningClause {
    #[default]
    None,
    All,
    Columns(Vec<String>),
}

implement_pyclass! {
    (
        /// Asterisk `"*"`
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        pub struct [] Py_AsteriskType as "_AsteriskType";
    )
    (
        /// Subclass of schema statements.
        ///
        /// @alias _BackendName = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]
        #[derive(Debug, Clone, Copy)]
        pub struct [subclass] PySchemaStatement as "SchemaStatement";
    )
    (
        /// Subclass of query statements.
        ///
        /// @alias _BackendName = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]
        #[derive(Debug, Clone, Copy)]
        pub struct [subclass] PyQueryStatement as "QueryStatement";
    )
    (
        /// Represents a reference to a database column with optional table and schema qualification.
        ///
        /// This class is used to uniquely identify columns in SQL queries, supporting
        /// schema-qualified and table-qualified column references.
        ///
        /// NOTE: this class is immutable and frozen.
        ///
        /// @signature (name: str | _AsteriskType, table: str | None = ..., schema: str | None = ...)
        #[derive(Debug, Clone)]
        pub struct [] PyColumnRef as "ColumnRef" {
            /// Name of the referenced column. [`Option::None`] means '*'.
            pub name: Option<sea_query::DynIden>,

            /// Table of the referenced column.
            pub table: Option<sea_query::DynIden>,

            /// Schema of the referenced column.
            pub schema: Option<sea_query::DynIden>,
        }
    )
    (
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
        pub struct [] PyTableName as "TableName" {
            /// Table name
            pub name: sea_query::DynIden,

            /// Table schema
            pub schema: Option<sea_query::DynIden>,

            /// Table database
            pub database: Option<sea_query::DynIden>,

            /// Alias name
            pub alias: Option<sea_query::DynIden>,
        }
    )
}

#[pyo3::pymethods]
impl PySchemaStatement {
    /// Build a SQL string representation.
    ///
    /// @signature (self, backend: _BackendName, /) -> str
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, backend: String) -> pyo3::PyResult<String> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }
}

#[pyo3::pymethods]
impl PyQueryStatement {
    /// Build a SQL string representation.
    ///
    /// **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
    ///
    /// @signature (self, backend: _BackendName, /) -> str
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }

    /// Build the SQL statement with parameter values.
    ///
    /// @signature (self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    fn build<'a>(
        &self,
        py: pyo3::Python<'a>,
        backend: String,
    ) -> pyo3::PyResult<(String, pyo3::Bound<'a, pyo3::PyAny>)> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
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

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for PyColumnRef {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_TYPE {
                let casted_value = value.cast_unchecked::<crate::column::PyColumn>();
                let get_value = casted_value.get();

                return Ok(Self {
                    name: Some(sea_query::Alias::new(&get_value.0.lock().name).into_iden()),
                    table: None,
                    schema: None,
                });
            }

            if let Ok(x) = value.cast_exact::<pyo3::types::PyString>() {
                return Self::from_str(x.to_str()?);
            }

            Err(typeerror!(
                "expected ColumnRef, Column or str, got {:?}",
                value.py(),
                value.as_ptr()
            ))
        }
    }
}

#[pyo3::pymethods]
impl PyColumnRef {
    #[new]
    #[pyo3(signature=(name, table=None, schema=None))]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        table: Option<String>,
        schema: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let name = unsafe {
            if pyo3::ffi::Py_TYPE(name.as_ptr()) == crate::typeref::ASTERISK_TYPE {
                None
            } else if pyo3::ffi::PyUnicode_CheckExact(name.as_ptr()) == 1 {
                Some(name.extract::<String>().unwrap_unchecked())
            } else {
                return Err(typeerror!(
                    "expected str or AsteriskType for name, got {:?}",
                    name.py(),
                    name.as_ptr()
                ));
            }
        };

        Ok(Self {
            name: name.map(|x| sea_query::Alias::new(x).into_iden()),
            table: table.map(|x| sea_query::Alias::new(x).into_iden()),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
        })
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        match &self.name {
            None => String::from("*"),
            Some(x) => x.to_string(),
        }
    }

    /// @signature (self) -> str | None
    #[getter]
    fn table(&self) -> Option<String> {
        self.table.as_ref().map(|x| x.to_string())
    }

    /// @signature (self) -> str | None
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
    ///
    /// @signature (cls, string: str) -> typing.Self
    #[classmethod]
    fn parse(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, string: String) -> pyo3::PyResult<Self> {
        Self::from_str(&string)
    }

    /// @signature (
    ///     self,
    ///     *,
    ///     name: str | _AsteriskType | None = ...,
    ///     table: str | None = ...,
    ///     schema: str | None = ...,
    /// ) -> typing.Self
    #[pyo3(signature=(**kwds))]
    fn copy_with(
        &self,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<Self> {
        use pyo3::types::PyDictMethods;

        let mut cloned = self.clone();
        if kwds.is_none() {
            return Ok(cloned);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };

        for (key, val) in kwds.iter() {
            unsafe {
                let key = key.extract::<String>().unwrap_unchecked();

                if key == "name" {
                    if pyo3::ffi::Py_IsNone(val.as_ptr()) == 1 {
                        // Nothing to do here
                    } else if pyo3::ffi::Py_TYPE(val.as_ptr()) == crate::typeref::ASTERISK_TYPE {
                        cloned.name = None;
                    } else if pyo3::ffi::PyUnicode_CheckExact(val.as_ptr()) == 1 {
                        cloned.name = Some(
                            sea_query::Alias::new(val.extract::<String>().unwrap_unchecked())
                                .into_iden(),
                        );
                    } else {
                        return Err(typeerror!(
                            "expected str or None or _AsteriskType, got {:?}",
                            val.py(),
                            val.as_ptr()
                        ));
                    }

                    continue;
                }

                let val: Option<String> = unsafe {
                    if pyo3::ffi::Py_IsNone(val.as_ptr()) == 1 {
                        None
                    } else if pyo3::ffi::PyUnicode_CheckExact(val.as_ptr()) == 1 {
                        Some(val.extract::<String>().unwrap_unchecked())
                    } else {
                        return Err(typeerror!(
                            "expected str or None, got {:?}",
                            val.py(),
                            val.as_ptr()
                        ));
                    }
                };

                if key == "table" {
                    cloned.table = val.map(|x| sea_query::Alias::new(x).into_iden());
                } else if key == "schema" {
                    cloned.schema = val.map(|x| sea_query::Alias::new(x).into_iden());
                } else {
                    return Err(typeerror!(format!(
                        "got an unexpected keyword argument '{}'",
                        key
                    ),));
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

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::new();

        match &self.name {
            Some(x) => write!(s, "<ColumnRef {:?}", x.to_string()).unwrap(),
            None => write!(s, "<ColumnRef *").unwrap(),
        }

        if let Some(x) = &self.table {
            write!(s, " table={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.schema {
            write!(s, " schema={:?}", x.to_string()).unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
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

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for PyTableName {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::TABLE_TYPE {
                let casted_value = value.cast_unchecked::<crate::table::PyTable>();
                return Ok(casted_value.get().0.lock().name.clone());
            }

            if let Ok(x) = value.extract::<String>() {
                return Self::from_str(&x);
            }

            Err(typeerror!(
                "expected TableName or str, got {:?}",
                value.py(),
                value.as_ptr()
            ))
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
    fn copy_with(
        &self,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<Self> {
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
                    return Err(typeerror!(
                        "expected str or None, got {:?}",
                        val.py(),
                        val.as_ptr()
                    ));
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
                return Err(typeerror!(format!(
                    "got an unexpected keyword argument '{}'",
                    key
                ),));
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
        use std::io::Write;

        let mut s = Vec::new();

        write!(s, "<TableName {:?}", self.name.to_string()).unwrap();
        if let Some(x) = &self.schema {
            write!(s, " schema={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.database {
            write!(s, " database={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.alias {
            write!(s, " alias={:?}", x.to_string()).unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
