use super::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, RefBoundObject, ToSeaQuery};

#[inline]
fn map_str_to_foreign_key_action(value: String) -> pyo3::PyResult<sea_query::ForeignKeyAction> {
    match value.to_lowercase().as_str() {
        "cascade" => Ok(sea_query::ForeignKeyAction::Cascade),
        "no action" => Ok(sea_query::ForeignKeyAction::NoAction),
        "restrict" => Ok(sea_query::ForeignKeyAction::Restrict),
        "set default" => Ok(sea_query::ForeignKeyAction::SetDefault),
        "set null" => Ok(sea_query::ForeignKeyAction::SetNull),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown foreign key action: {value}",
        ))),
    }
}

#[inline]
fn map_foreign_key_action_to_str(value: sea_query::ForeignKeyAction) -> String {
    match value {
        sea_query::ForeignKeyAction::Cascade => String::from("CASCADE"),
        sea_query::ForeignKeyAction::NoAction => String::from("NO ACTION"),
        sea_query::ForeignKeyAction::Restrict => String::from("RESTRICT"),
        sea_query::ForeignKeyAction::SetDefault => String::from("SET DEFAULT"),
        sea_query::ForeignKeyAction::SetNull => String::from("SET NULL"),
    }
}

crate::implement_pyclass! {
    /// Specifies a foreign key relationship between tables.
    ///
    /// Defines referential integrity constraints including:
    /// - Source columns (in the child table)
    /// - Target columns (in the parent table)
    /// - Actions for updates and deletes (CASCADE, RESTRICT, SET NULL, etc.)
    /// - Optional naming for the constraint
    ///
    /// Foreign keys ensure data consistency by requiring that values in the
    /// child table's columns match existing values in the parent table's columns.
    #[derive(Debug, Clone)]
    mutable [subclass] PyForeignKey(ForeignKeyState) as "ForeignKey" {
        /// Foreign key constraint name
        pub name: String,

        /// To table
        pub to_table: PyTableName,

        /// To columns
        pub to_columns: Vec<String>,

        /// From table
        pub from_table: Option<PyTableName>,

        /// From columns
        pub from_columns: Vec<String>,

        /// On delete action
        pub on_delete: Option<sea_query::ForeignKeyAction>,

        /// On update action
        pub on_update: Option<sea_query::ForeignKeyAction>,
    }
}

impl ToSeaQuery<sea_query::ForeignKeyCreateStatement> for ForeignKeyState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::ForeignKeyCreateStatement {
        let mut stmt = sea_query::ForeignKeyCreateStatement::new();
        stmt.name(&self.name);
        stmt.to_tbl(self.to_table.clone());

        for c in &self.from_columns {
            stmt.from_col(sea_query::Alias::new(c));
        }
        for c in &self.to_columns {
            stmt.to_col(sea_query::Alias::new(c));
        }

        if let Some(x) = &self.from_table {
            stmt.from_tbl(x.clone());
        }
        if let Some(x) = self.on_delete {
            stmt.on_delete(x);
        }
        if let Some(x) = self.on_update {
            stmt.on_update(x);
        }

        stmt
    }
}

impl ToSeaQuery<sea_query::TableForeignKey> for ForeignKeyState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::TableForeignKey {
        let mut stmt = sea_query::TableForeignKey::new();
        stmt.name(&self.name);
        stmt.to_tbl(self.to_table.clone());

        for c in &self.from_columns {
            stmt.from_col(sea_query::Alias::new(c));
        }
        for c in &self.to_columns {
            stmt.to_col(sea_query::Alias::new(c));
        }

        if let Some(x) = &self.from_table {
            stmt.from_tbl(x.clone());
        }
        if let Some(x) = self.on_delete {
            stmt.on_delete(x);
        }
        if let Some(x) = self.on_update {
            stmt.on_update(x);
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyForeignKey {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    #[
        pyo3(
            signature=(
                from_columns,
                to_table,
                to_columns,
                from_table=None,
                name=None,
                *,
                on_delete=None,
                on_update=None
            )
        )
    ]
    fn __init__(
        &self,
        from_columns: Vec<BoundObject<'_>>,
        to_table: RefBoundObject<'_>,
        to_columns: Vec<BoundObject<'_>>,
        from_table: Option<RefBoundObject<'_>>,
        name: Option<String>,
        on_delete: Option<String>,
        on_update: Option<String>,
    ) -> pyo3::PyResult<()> {
        // Validate & convert actions
        let on_delete = match on_delete {
            None => None,
            Some(x) => Some(map_str_to_foreign_key_action(x)?),
        };
        let on_update = match on_update {
            None => None,
            Some(x) => Some(map_str_to_foreign_key_action(x)?),
        };

        // Validate & convert tables
        let from_table = match from_table {
            None => None,
            Some(x) => Some(PyTableName::try_from(x)?),
        };
        let to_table = PyTableName::try_from(to_table)?;

        // Validate from/to_columns
        if from_columns.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "from_columns is empty",
            ));
        }
        if to_columns.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "to_columns is empty",
            ));
        }
        if from_columns.len() != to_columns.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "from_columns and to_columns must have same length ({} != {})",
                from_columns.len(),
                to_columns.len()
            )));
        }

        // Convert from_columns
        let mut from_columns_str = Vec::with_capacity(from_columns.len());

        for col in from_columns.into_iter() {
            let col_ref = super::column_ref::PyColumnRef::try_from(&col)?;

            match col_ref.name {
                Some(x) => from_columns_str.push(x.to_string()),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ForeignKey cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        // Convert to_columns
        let mut to_columns_str = Vec::with_capacity(to_columns.len());

        for col in to_columns.into_iter() {
            let col_ref = super::column_ref::PyColumnRef::try_from(&col)?;

            match col_ref.name {
                Some(x) => to_columns_str.push(x.to_string()),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ForeignKey cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        let name = match name {
            Some(x) => x,
            None => {
                let mut s = format!("fk_{}", to_table.name.to_string());

                for col in from_columns_str.iter() {
                    s.push('_');
                    s += col;
                }

                s.push('_');

                for col in to_columns_str.iter() {
                    s.push('_');
                    s += col;
                }

                s
            }
        };

        let result = ForeignKeyState {
            name,
            to_table,
            to_columns: to_columns_str,
            from_table,
            from_columns: from_columns_str,
            on_delete,
            on_update,
        };
        self.0.set(result);
        Ok(())
    }

    /// Foreign key constraint name
    #[getter]
    fn name(&self) -> String {
        self.0.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.0.lock();
        lock.name = val;
    }

    /// Key table, if specified.
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_table(&self) -> Option<PyTableName> {
        self.0.lock().from_table.as_ref().cloned()
    }

    #[setter]
    fn set_from_table(&self, value: Option<RefBoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        lock.from_table = match value {
            None => None,
            Some(x) => Some(super::table_ref::PyTableName::try_from(x)?),
        };
        Ok(())
    }

    /// Referencing table.
    #[getter]
    fn to_table(&self) -> PyTableName {
        self.0.lock().to_table.clone()
    }

    #[setter]
    fn set_to_table(&self, value: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        lock.to_table = super::table_ref::PyTableName::try_from(value)?;
        Ok(())
    }

    /// Key columns.
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_columns(&self) -> Vec<String> {
        self.0.lock().from_columns.clone()
    }

    #[setter]
    fn set_from_columns(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();

        if val.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "from_columns is empty",
            ));
        }
        if val.len() != lock.to_columns.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "from_columns and to_columns must have same length ({} != {})",
                val.len(),
                lock.to_columns.len()
            )));
        }

        let mut from_columns_str = Vec::with_capacity(val.len());
        for col in val.into_iter() {
            let col_ref = super::column_ref::PyColumnRef::try_from(&col)?;

            match col_ref.name {
                Some(x) => from_columns_str.push(x.to_string()),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ForeignKey cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        lock.from_columns = from_columns_str;
        Ok(())
    }

    /// Referencing columns.
    #[getter]
    fn to_columns(&self) -> Vec<String> {
        self.0.lock().to_columns.clone()
    }

    #[setter]
    fn set_to_columns(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();

        if val.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "to_columns is empty",
            ));
        }
        if val.len() != lock.from_columns.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "to_columns and to_columns must have same length ({} != {})",
                val.len(),
                lock.to_columns.len()
            )));
        }

        let mut to_columns_str = Vec::with_capacity(val.len());
        for col in val.into_iter() {
            let col_ref = super::column_ref::PyColumnRef::try_from(&col)?;

            match col_ref.name {
                Some(x) => to_columns_str.push(x.to_string()),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ForeignKey cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        lock.to_columns = to_columns_str;
        Ok(())
    }

    /// ON DELETE action.
    #[getter]
    fn on_delete(&self) -> Option<String> {
        self.0.lock().on_delete.map(map_foreign_key_action_to_str)
    }

    #[setter]
    fn set_on_delete(&self, value: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        lock.on_delete = match value {
            None => None,
            Some(x) => Some(map_str_to_foreign_key_action(x)?),
        };
        Ok(())
    }

    /// ON UPDATE action.
    #[getter]
    fn on_update(&self) -> Option<String> {
        self.0.lock().on_update.map(map_foreign_key_action_to_str)
    }

    #[setter]
    fn set_on_update(&self, value: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        lock.on_update = match value {
            None => None,
            Some(x) => Some(map_str_to_foreign_key_action(x)?),
        };
        Ok(())
    }

    fn __copy__(&self) -> Self {
        let lock = self.0.lock();
        lock.clone().into()
    }

    pub fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        ReprFormatter::new_with_pyref(&slf)
            .quote("name", &lock.name)
            .map("to_table", &lock.to_table, |x| x.__repr__())
            .debug("to_columns", &lock.to_columns)
            .debug("from_columns", &lock.from_columns)
            .optional_map("from_table", lock.from_table.as_ref(), |x| x.__repr__())
            .optional_map("on_delete", lock.on_delete, map_foreign_key_action_to_str)
            .optional_map("on_update", lock.on_update, map_foreign_key_action_to_str)
            .finish()
    }
}
