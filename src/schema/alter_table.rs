use super::base::PySchemaStatement;
use crate::common::column::PyColumn;
use crate::common::column_ref::PyColumnRef;
use crate::common::foreign_key::PyForeignKey;
use crate::common::table_ref::PyTableName;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};

use pyo3::types::PyAnyMethods;
use pyo3::PyTypeInfo;
use sea_query::IntoIden;

crate::implement_pyclass! {
    // Base class for all ALTER TABLE operation types.
    ///
    /// This abstract base class represents the different types of modifications
    /// that can be made to an existing table structure, such as adding/dropping
    /// columns, modifying column definitions, or managing foreign keys.
    #[derive(Debug, Clone, Copy)]
    [subclass] PyAlterTableBaseOption as "AlterTableBaseOption";
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to add a new column.
    ///
    /// Adds a column to an existing table with optional IF NOT EXISTS clause
    /// to prevent errors if the column already exists.
    ///
    /// @signature (self, column: Column, if_not_exists: bool = False)
    #[derive(Debug)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableAddColumnOption(AlterTableAddColumnOptionState)
    as "AlterTableAddColumnOption" {
        /// Always is `PyColumn`
        column: PyObject,
        if_not_exists: bool,
    }
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to add a foreign key constraint.
    ///
    /// Adds referential integrity between tables by creating a foreign key
    /// relationship on an existing table.
    ///
    /// @signature (self, foreign_key: ForeignKey)
    #[derive(Debug)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableAddForeignKeyOption(AlterTableAddForeignKeyOptionState)
    as "AlterTableAddForeignKeyOption" {
        /// Always is `PyForeignKey`
        foreign_key: PyObject,
    }
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to drop an existing column.
    ///
    /// Removes a column from the table. This operation may fail if the column
    /// is referenced by other database objects.
    ///
    /// @signature (self, name: Column | ColumnRef | str)
    #[derive(Debug, Clone)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableDropColumnOption(AlterTableDropColumnOptionState)
    as "AlterTableDropColumnOption" { name: sea_query::DynIden }
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to drop a foreign key constraint.
    ///
    /// Removes a foreign key relationship by its constraint name.
    ///
    /// @signature (self, name: ForeignKey | str)
    #[derive(Debug, Clone)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableDropForeignKeyOption(AlterTableDropForeignKeyOptionState)
    as "AlterTableDropForeignKeyOption" { name: sea_query::DynIden }
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to modify a column definition.
    ///
    /// Changes properties of an existing column such as type, nullability,
    /// default value, or other constraints.
    ///
    /// @signature (self, column: Column)
    #[derive(Debug)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableModifyColumnOption(AlterTableModifyColumnOptionState)
    as "AlterTableModifyColumnOption" {
        /// Always is `PyColumn`
        column: PyObject,
    }
}
crate::implement_pyclass! {
    /// ALTER TABLE operation to rename a column.
    ///
    /// Changes the name of an existing column without modifying its type
    /// or constraints.
    ///
    /// @signature (self, from_name: Column | ColumnRef | str, to_name: Column | ColumnRef | str)
    #[derive(Debug, Clone)]
    immutable [subclass, extends=PyAlterTableBaseOption] PyAlterTableRenameColumnOption(AlterTableRenameColumnOptionState)
    as "AlterTableRenameColumnOption" {
        from_name: sea_query::DynIden,
        to_name: sea_query::DynIden,
    }
}
crate::implement_pyclass! {
    /// Represents an ALTER TABLE SQL statement.
    ///
    /// Provides a flexible way to modify existing table structures by applying
    /// one or more alteration operations such as adding/dropping columns,
    /// modifying column definitions, or managing constraints.
    ///
    /// Multiple operations can be batched together in a single ALTER TABLE
    /// statement for efficiency.
    ///
    /// @signature (self, name: Table | TableName | str, options: typing.Iterable[AlterTableBaseOption] = ())
    #[derive(Debug)]
    mutable [subclass, extends=PySchemaStatement] PyAlterTable(AlterTableState) as "AlterTable" {
        name: PyTableName,
        options: Vec<PyObject>,
    }
}

#[pyo3::pymethods]
impl PyAlterTableAddColumnOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature = (column, if_not_exists=false))]
    fn __init__(&self, column: RefBoundObject<'_>, if_not_exists: bool) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(column.as_ptr(), crate::typeref::COLUMN_TYPE) == 0 {
                return crate::new_error!(
                    PyTypeError,
                    "expected Column, got {}",
                    crate::internal::get_type_name(column.py(), column.as_ptr())
                );
            }
        }

        let result = AlterTableAddColumnOptionState {
            column: column.clone().unbind(),
            if_not_exists,
        };
        unsafe {
            self.0.set(result);
        }
        Ok(())
    }

    /// @signature (self) -> Column
    #[getter]
    fn column(&self, py: pyo3::Python) -> PyObject {
        self.0.as_ref().column.clone_ref(py)
    }

    /// @signature (self) -> bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.as_ref().if_not_exists
    }

    fn __repr__(&self) -> String {
        let inner = self.0.as_ref();
        let result = format!("<AlterTableAddColumnOption {}", inner.column);

        if inner.if_not_exists {
            result + " if_not_exists=True>"
        } else {
            result + ">"
        }
    }
}

#[pyo3::pymethods]
impl PyAlterTableAddForeignKeyOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature = (foreign_key))]
    fn __init__(&self, foreign_key: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(foreign_key.as_ptr(), crate::typeref::FOREIGN_KEY_TYPE)
                == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected ForeignKey, got {}",
                    crate::internal::get_type_name(foreign_key.py(), foreign_key.as_ptr())
                );
            }
        }

        let result = AlterTableAddForeignKeyOptionState {
            foreign_key: foreign_key.clone().unbind(),
        };
        unsafe {
            self.0.set(result);
        }
        Ok(())
    }

    /// @signature (self) -> ForeignKey
    #[getter]
    fn foreign_key(&self, py: pyo3::Python) -> PyObject {
        self.0.as_ref().foreign_key.clone_ref(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableAddForeignKeyOption {}>",
            self.0.as_ref().foreign_key
        )
    }
}

#[pyo3::pymethods]
impl PyAlterTableDropColumnOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature=(name))]
    fn __init__(&self, name: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let column_ref = PyColumnRef::try_from(name)?;

        match column_ref.name {
            Some(x) => {
                let result = AlterTableDropColumnOptionState { name: x };
                unsafe {
                    self.0.set(result);
                }
                Ok(())
            }
            None => crate::new_error!(
                PyValueError,
                "AlterTableDropColumnOption cannot accept asterisk '*' as name"
            ),
        }
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.0.as_ref().name.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableDropColumnOption {:?}>",
            self.0.as_ref().name.to_string()
        )
    }
}

#[pyo3::pymethods]
impl PyAlterTableDropForeignKeyOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature=(name))]
    fn __init__(&self, name: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let name_string = unsafe {
            if pyo3::ffi::PyObject_TypeCheck(name.as_ptr(), crate::typeref::FOREIGN_KEY_TYPE) == 1 {
                let fk = name.cast_unchecked::<PyForeignKey>();
                fk.get().0.lock().name.clone()
            } else if let Ok(x) = name.extract::<String>() {
                x
            } else {
                return crate::new_error!(
                    PyTypeError,
                    "expected expected ForeignKey or str, got {}",
                    crate::internal::get_type_name(name.py(), name.as_ptr())
                );
            }
        };

        let result = AlterTableDropForeignKeyOptionState {
            name: sea_query::Alias::new(name_string).into_iden(),
        };
        unsafe {
            self.0.set(result);
        }
        Ok(())
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.0.as_ref().name.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableDropForeignKeyOption {:?}>",
            self.0.as_ref().name.to_string()
        )
    }
}

#[pyo3::pymethods]
impl PyAlterTableModifyColumnOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature = (column))]
    fn __init__(&self, column: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(column.as_ptr(), crate::typeref::COLUMN_TYPE) == 0 {
                return crate::new_error!(
                    PyTypeError,
                    "expected Column, got {}",
                    crate::internal::get_type_name(column.py(), column.as_ptr())
                );
            }
        }

        let result = AlterTableModifyColumnOptionState {
            column: column.clone().unbind(),
        };
        unsafe {
            self.0.set(result);
        }
        Ok(())
    }

    /// @signature (self) -> Column
    #[getter]
    fn column(&self, py: pyo3::Python) -> PyObject {
        self.0.as_ref().column.clone_ref(py)
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableModifyColumnOption {}>", self.0.as_ref().column)
    }
}

#[pyo3::pymethods]
impl PyAlterTableRenameColumnOption {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: BoundArgs<'_>,
        kwds: Option<BoundKwargs<'_>>,
    ) -> (Self, PyAlterTableBaseOption) {
        (Self::uninit(), PyAlterTableBaseOption)
    }

    #[pyo3(signature = (from_name, to_name))]
    fn __init__(
        &self,
        from_name: RefBoundObject<'_>,
        to_name: RefBoundObject<'_>,
    ) -> pyo3::PyResult<()> {
        let from_column_ref = PyColumnRef::try_from(from_name)?;
        let to_column_ref = PyColumnRef::try_from(to_name)?;

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
            let result = AlterTableRenameColumnOptionState {
                from_name: from_column_ref.name.unwrap_unchecked(),
                to_name: to_column_ref.name.unwrap_unchecked(),
            };
            self.0.set(result);
            Ok(())
        }
    }

    /// @signature (self) -> str
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_name(&self) -> String {
        self.0.as_ref().from_name.clone().to_string()
    }

    /// @signature (self) -> str
    #[getter]
    fn to_name(&self) -> String {
        self.0.as_ref().to_name.clone().to_string()
    }

    fn __repr__(&self) -> String {
        let inner = self.0.as_ref();
        format!(
            "<AlterTableRenameColumnOption {:?} {:?}>",
            inner.from_name.to_string(),
            inner.to_name.to_string()
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
                    let get_add_column_opt = add_column_opt.get().as_ref();
                    let py_column = get_add_column_opt
                        .column
                        .cast_bound_unchecked::<PyColumn>(py)
                        .get()
                        .clone();

                    let column_def: sea_query::ColumnDef = py_column.0.lock().to_sea_query(py);

                    if get_add_column_opt.if_not_exists {
                        stmt.add_column_if_not_exists(column_def);
                    } else {
                        stmt.add_column(column_def);
                    }
                } else if op_type == PyAlterTableAddForeignKeyOption::type_object_raw(py) {
                    let add_fk = op.cast_bound_unchecked::<PyAlterTableAddForeignKeyOption>(py);
                    let get_add_fk = add_fk.get().as_ref();

                    let py_fk = get_add_fk
                        .foreign_key
                        .cast_bound_unchecked::<PyForeignKey>(py)
                        .get()
                        .clone();

                    let table_fk: sea_query::TableForeignKey = py_fk.0.lock().to_sea_query(py);
                    stmt.add_foreign_key(&table_fk);
                } else if op_type == PyAlterTableDropColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropColumnOption>(py);
                    let x = bound.get();

                    stmt.drop_column(x.as_ref().name.clone());
                } else if op_type == PyAlterTableDropForeignKeyOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropForeignKeyOption>(py);
                    let x = bound.get();

                    stmt.drop_foreign_key(x.0.as_ref().name.clone());
                } else if op_type == PyAlterTableModifyColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableModifyColumnOption>(py);
                    let x = bound.get().as_ref();

                    let py_column = x.column.cast_bound_unchecked::<PyColumn>(py).get().clone();

                    let column_def: sea_query::ColumnDef = py_column.0.lock().to_sea_query(py);
                    stmt.modify_column(column_def);
                } else if op_type == PyAlterTableRenameColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableRenameColumnOption>(py);
                    let x = bound.get().as_ref();

                    stmt.rename_column(x.from_name.clone(), x.to_name.clone());
                }
            }
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyAlterTable {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (name, options=Vec::new()))]
    fn __init__(
        &self,
        name: RefBoundObject<'_>,
        options: Vec<BoundObject<'_>>,
    ) -> pyo3::PyResult<()> {
        let name = PyTableName::try_from(name)?;

        unsafe {
            for opt in options.iter() {
                if pyo3::ffi::PyObject_TypeCheck(
                    opt.as_ptr(),
                    crate::typeref::ALTER_TABLE_BASE_OPTION_TYPE,
                ) == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected AlterTableBaseOption, got {}",
                        crate::internal::get_type_name(opt.py(), opt.as_ptr())
                    );
                }
            }
        }

        let state = AlterTableState {
            name,
            options: options.into_iter().map(|x| x.unbind()).collect(),
        };
        self.0.set(state);
        Ok(())
    }

    /// The name of the table to alter.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    fn name(&self) -> PyTableName {
        self.0.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, val: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.name = val;
        Ok(())
    }

    /// The list of alteration operations to apply.
    ///
    /// @signature (self) -> typing.Sequence[AlterTableBaseOption]
    /// @setter typing.Iterable[AlterTableBaseOption]
    #[getter]
    fn options(&self, py: pyo3::Python) -> Vec<PyObject> {
        self.0
            .lock()
            .options
            .iter()
            .map(|x| x.clone_ref(py))
            .collect()
    }

    #[setter]
    fn set_options(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        unsafe {
            for opt in val.iter() {
                if pyo3::ffi::PyObject_TypeCheck(
                    opt.as_ptr(),
                    crate::typeref::ALTER_TABLE_BASE_OPTION_TYPE,
                ) == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected AlterTableBaseOption, got {}",
                        crate::internal::get_type_name(opt.py(), opt.as_ptr())
                    );
                }
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
        opt: BoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                opt.as_ptr(),
                crate::typeref::ALTER_TABLE_BASE_OPTION_TYPE,
            ) == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected AlterTableBaseOption, got {}",
                    crate::internal::get_type_name(opt.py(), opt.as_ptr())
                );
            }
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

        // Prevent "No alter option found" panic
        if lock.options.is_empty() {
            return crate::new_error!(
                PyRuntimeError,
                "No alter option found. You have to specify at least one alter option."
            );
        }

        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_schema_statement!(backend, stmt)
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
