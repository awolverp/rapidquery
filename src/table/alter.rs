use crate::{common::PySchemaStatement, utils::ToSeaQuery};
use pyo3::{types::PyAnyMethods, PyTypeInfo};

implement_pyclass! {
    (
        /// Base class for all ALTER TABLE operation types.
        ///
        /// This abstract base class represents the different types of modifications
        /// that can be made to an existing table structure, such as adding/dropping
        /// columns, modifying column definitions, or managing foreign keys.
        #[derive(Debug, Clone, Copy)]
        pub struct [subclass] PyAlterTableBaseOption as "AlterTableBaseOption";
    )
    (
        /// ALTER TABLE operation to add a new column.
        ///
        /// Adds a column to an existing table with optional IF NOT EXISTS clause
        /// to prevent errors if the column already exists.
        ///
        /// @signature (column: Column, if_not_exists: bool = False)
        #[derive(Debug)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableAddColumnOption
        as "AlterTableAddColumnOption" {
            column: crate::column::PyColumn,
            if_not_exists: bool,
        }
    )
    (
        /// ALTER TABLE operation to add a foreign key constraint.
        ///
        /// Adds referential integrity between tables by creating a foreign key
        /// relationship on an existing table.
        ///
        /// @signature (foreign_key: ForeignKey)
        #[derive(Debug)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableAddForeignKeyOption
        as "AlterTableAddForeignKeyOption" {
            foreign_key: crate::foreign_key::PyForeignKey,
        }
    )
    (
        /// ALTER TABLE operation to drop an existing column.
        ///
        /// Removes a column from the table. This operation may fail if the column
        /// is referenced by other database objects.
        ///
        /// @signature (name: Column | ColumnRef | str)
        #[derive(Debug, Clone)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableDropColumnOption
        as "AlterTableDropColumnOption" {
            name: String,
        }
    )
    (
        /// ALTER TABLE operation to drop a foreign key constraint.
        ///
        /// Removes a foreign key relationship by its constraint name.
        ///
        /// @signature (name: ForeignKey | str)
        #[derive(Debug, Clone)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableDropForeignKeyOption
        as "AlterTableDropForeignKeyOption" {
            name: String,
        }
    )
    (
        /// ALTER TABLE operation to modify a column definition.
        ///
        /// Changes properties of an existing column such as type, nullability,
        /// default value, or other constraints.
        ///
        /// @signature (column: Column)
        #[derive(Debug)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableModifyColumnOption
        as "AlterTableModifyColumnOption" {
            column: crate::column::PyColumn,
        }
    )
    (
        /// ALTER TABLE operation to rename a column.
        ///
        /// Changes the name of an existing column without modifying its type
        /// or constraints.
        ///
        /// @signature (from_name: Column | ColumnRef | str, to_name: Column | ColumnRef | str)
        #[derive(Debug, Clone)]
        pub struct [extends=PyAlterTableBaseOption] PyAlterTableRenameColumnOption
        as "AlterTableRenameColumnOption" {
            from_name: String,
            to_name: String,
        }
    )
}
implement_state_pyclass! {
    /// Represents an ALTER TABLE SQL statement.
    ///
    /// Provides a flexible way to modify existing table structures by applying
    /// one or more alteration operations such as adding/dropping columns,
    /// modifying column definitions, or managing constraints.
    ///
    /// Multiple operations can be batched together in a single ALTER TABLE
    /// statement for efficiency.
    ///
    /// @signature (name: Table | TableName | str, options: typing.Iterable[AlterTableBaseOption] = ())
    #[derive(Debug)]
    pub struct [extends=PySchemaStatement] PyAlterTable(AlterTableState) as "AlterTable" {
        name: crate::common::PyTableName,
        options: Vec<pyo3::Py<pyo3::PyAny>>,
    }
}

#[pyo3::pymethods]
impl PyAlterTableAddColumnOption {
    #[new]
    #[pyo3(signature = (column, if_not_exists=false))]
    fn __new__(
        column: &pyo3::Bound<'_, pyo3::PyAny>,
        if_not_exists: bool,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let column = column.cast::<crate::column::PyColumn>()?;
        let get_column = column.get();

        let result = Self {
            column: crate::column::PyColumn(std::sync::Arc::clone(&get_column.0)),
            if_not_exists,
        };
        Ok((result, PyAlterTableBaseOption))
    }

    /// @signature (self) -> Column
    #[getter]
    fn column(&self) -> crate::column::PyColumn {
        crate::column::PyColumn(std::sync::Arc::clone(&self.column.0))
    }

    /// @signature (self) -> bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    fn __repr__(&self) -> String {
        let result = format!("<AlterTableAddColumnOption {}", self.column.__repr__());

        if self.if_not_exists {
            result + " if_not_exists=True>"
        } else {
            result + ">"
        }
    }
}

#[pyo3::pymethods]
impl PyAlterTableAddForeignKeyOption {
    #[new]
    #[pyo3(signature = (foreign_key))]
    fn __new__(
        foreign_key: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let foreign_key = foreign_key.cast::<crate::foreign_key::PyForeignKey>()?;
        let get_foreign_key = foreign_key.get();

        let result = Self {
            foreign_key: crate::foreign_key::PyForeignKey(std::sync::Arc::clone(
                &get_foreign_key.0,
            )),
        };
        Ok((result, PyAlterTableBaseOption))
    }

    /// @signature (self) -> ForeignKey
    #[getter]
    fn foreign_key(&self) -> crate::foreign_key::PyForeignKey {
        crate::foreign_key::PyForeignKey(std::sync::Arc::clone(&self.foreign_key.0))
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableAddForeignKeyOption {}>",
            self.foreign_key.__repr__()
        )
    }
}

#[pyo3::pymethods]
impl PyAlterTableDropColumnOption {
    #[new]
    #[pyo3(signature = (name))]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let column_ref = crate::common::PyColumnRef::try_from(name)?;

        match column_ref.name {
            None => Err(pyo3::exceptions::PyValueError::new_err(
                "AlterTableDropColumnOption cannot accept asterisk '*' as name",
            )),
            Some(x) => {
                let result = Self {
                    name: x.to_string(),
                };
                Ok((result, PyAlterTableBaseOption))
            }
        }
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableDropColumnOption {}>", self.name)
    }
}

#[pyo3::pymethods]
impl PyAlterTableDropForeignKeyOption {
    #[new]
    #[pyo3(signature = (name))]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let name_string = unsafe {
            if pyo3::ffi::Py_TYPE(name.as_ptr()) == crate::typeref::FOREIGN_KEY_TYPE {
                let fk = name.cast_unchecked::<crate::foreign_key::PyForeignKey>();
                fk.get().0.lock().name.clone()
            } else if let Ok(x) = name.extract::<String>() {
                x
            } else {
                return Err(typeerror!(
                    "expected ForeignKey or str, got {:?}",
                    name.py(),
                    name.as_ptr()
                ));
            }
        };

        let result = Self { name: name_string };
        Ok((result, PyAlterTableBaseOption))
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableDropForeignKeyOption {}>", self.name)
    }
}

#[pyo3::pymethods]
impl PyAlterTableModifyColumnOption {
    #[new]
    #[pyo3(signature = (column))]
    fn __new__(
        column: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let column = column.cast::<crate::column::PyColumn>()?;
        let get_column = column.get();

        let result = Self {
            column: crate::column::PyColumn(std::sync::Arc::clone(&get_column.0)),
        };
        Ok((result, PyAlterTableBaseOption))
    }

    /// @signature (self) -> Column
    #[getter]
    fn column(&self) -> crate::column::PyColumn {
        crate::column::PyColumn(std::sync::Arc::clone(&self.column.0))
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableModifyColumnOption {}>", self.column.__repr__())
    }
}

#[pyo3::pymethods]
impl PyAlterTableRenameColumnOption {
    #[new]
    #[pyo3(signature = (from_name, to_name))]
    fn __new__(
        from_name: &pyo3::Bound<'_, pyo3::PyAny>,
        to_name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PyAlterTableBaseOption)> {
        let from_column_ref = crate::common::PyColumnRef::try_from(from_name)?;
        let to_column_ref = crate::common::PyColumnRef::try_from(to_name)?;

        if from_column_ref.name.is_none() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "AlterTableRenameColumnOption cannot accept asterisk '*' as from_name",
            ));
        }
        if to_column_ref.name.is_none() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "AlterTableRenameColumnOption cannot accept asterisk '*' as to_name",
            ));
        }

        unsafe {
            let result = Self {
                from_name: from_column_ref.name.unwrap_unchecked().to_string(),
                to_name: to_column_ref.name.unwrap_unchecked().to_string(),
            };
            Ok((result, PyAlterTableBaseOption))
        }
    }

    /// @signature (self) -> str
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_name(&self) -> String {
        self.from_name.clone()
    }

    /// @signature (self) -> str
    #[getter]
    fn to_name(&self) -> String {
        self.to_name.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableRenameColumnOption {} {}>",
            self.from_name, self.to_name
        )
    }
}

impl ToSeaQuery<sea_query::TableAlterStatement> for AlterTableState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::TableAlterStatement {
        let mut stmt = sea_query::TableAlterStatement::new();
        stmt.table(self.name.clone());

        for op in self.options.iter() {
            unsafe {
                let op_type = pyo3::ffi::Py_TYPE(op.as_ptr());

                if op_type == PyAlterTableAddColumnOption::type_object_raw(py) {
                    let add_column_opt = op.cast_bound_unchecked::<PyAlterTableAddColumnOption>(py);
                    let get_add_column_opt = add_column_opt.get();

                    let column_def: sea_query::ColumnDef =
                        get_add_column_opt.column.0.lock().to_sea_query(py);

                    if get_add_column_opt.if_not_exists {
                        stmt.add_column_if_not_exists(column_def);
                    } else {
                        stmt.add_column(column_def);
                    }
                } else if op_type == PyAlterTableAddForeignKeyOption::type_object_raw(py) {
                    let add_fk = op.cast_bound_unchecked::<PyAlterTableAddForeignKeyOption>(py);
                    let get_add_fk = add_fk.get();

                    let table_fk: sea_query::TableForeignKey =
                        get_add_fk.foreign_key.0.lock().to_sea_query(py);

                    stmt.add_foreign_key(&table_fk);
                } else if op_type == PyAlterTableDropColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropColumnOption>(py);
                    let x = bound.get();

                    stmt.drop_column(sea_query::Alias::new(&x.name));
                } else if op_type == PyAlterTableDropForeignKeyOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropForeignKeyOption>(py);
                    let x = bound.get();

                    stmt.drop_foreign_key(sea_query::Alias::new(&x.name));
                } else if op_type == PyAlterTableModifyColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableModifyColumnOption>(py);
                    let x = bound.get();

                    let column_def: sea_query::ColumnDef = x.column.0.lock().to_sea_query(py);

                    stmt.modify_column(column_def);
                } else if op_type == PyAlterTableRenameColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableRenameColumnOption>(py);
                    let x = bound.get();

                    stmt.rename_column(
                        sea_query::Alias::new(&x.from_name),
                        sea_query::Alias::new(&x.to_name),
                    );
                }
            }
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyAlterTable {
    #[new]
    #[pyo3(signature = (name, options=Vec::new()))]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        options: Vec<pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let name = crate::common::PyTableName::try_from(name)?;

        for opt in options.iter() {
            if !opt.is_instance_of::<PyAlterTableBaseOption>() {
                return Err(typeerror!(
                    "expected instance of AlterTableBaseOption, got {:?}",
                    opt.py(),
                    opt.as_ptr()
                ));
            }
        }

        let state = AlterTableState {
            name,
            options: options.into_iter().map(|x| x.unbind()).collect(),
        };
        Ok((state.into(), PySchemaStatement))
    }

    /// The name of the table to alter.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    fn name(&self) -> crate::common::PyTableName {
        self.0.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = crate::common::PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.name = val;
        Ok(())
    }

    /// The list of alteration operations to apply.
    ///
    /// @signature (self) -> typing.Sequence[AlterTableBaseOption]
    /// @setter typing.Iterable[AlterTableBaseOption]
    #[getter]
    fn options(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        self.0
            .lock()
            .options
            .iter()
            .map(|x| x.clone_ref(py))
            .collect()
    }

    #[setter]
    fn set_options(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        for opt in val.iter() {
            if !opt.is_instance_of::<PyAlterTableBaseOption>() {
                return Err(typeerror!(
                    "expected instance of AlterTableBaseOption, got {:?}",
                    opt.py(),
                    opt.as_ptr()
                ));
            }
        }

        let mut lock = self.0.lock();
        lock.options = val.into_iter().map(|x| x.unbind()).collect();
        Ok(())
    }

    /// Add an alteration operation to this ALTER TABLE statement.
    ///
    /// @signature (self, opt: AlterTableBaseOption) -> typing.Self
    fn add_option<'a>(
        slf: pyo3::PyRef<'a, Self>,
        opt: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !opt.is_instance_of::<PyAlterTableBaseOption>() {
            return Err(typeerror!(
                "expected instance of AlterTableBaseOption, got {:?}",
                opt.py(),
                opt.as_ptr()
            ));
        }

        {
            let mut lock = slf.0.lock();
            lock.options.push(opt.unbind());
        }
        Ok(slf)
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        build_schema_statement!(backend, stmt)
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();

        let new_state = AlterTableState {
            name: lock.name.clone(),
            options: lock.options.iter().map(|x| x.clone_ref(py)).collect(),
        };

        pyo3::Py::new(py, (new_state.into(), PySchemaStatement))
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<AlterTable name={} options=[", lock.name.__repr__()).unwrap();

        let n = lock.options.len().saturating_sub(1);
        for (index, op) in lock.options.iter().enumerate() {
            if index == n {
                write!(s, "{op}").unwrap();
            } else {
                write!(s, "{op}, ").unwrap();
            }
        }
        write!(s, "]>").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
