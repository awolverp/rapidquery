use pyo3::types::{PyAnyMethods, PyDictMethods, PyTupleMethods};
use sea_query::IntoIden;

use crate::{expression::PyExpr, utils::ToSeaQuery};

#[derive(Debug)]
pub enum OnConflictUpdate {
    Column(sea_query::DynIden),
    Expr(sea_query::DynIden, PyExpr),
}

#[derive(Debug)]
pub enum OnConflictAction {
    None,
    DoNothing(Vec<sea_query::DynIden>),
    DoUpdate(Vec<OnConflictUpdate>),
}

implement_state_pyclass! {
    /// Specifies conflict resolution behavior for INSERT statements.
    ///
    /// Handles situations where an INSERT would violate a unique constraint
    /// or primary key. Supports various strategies:
    /// - DO NOTHING: Skip the conflicting row
    /// - DO UPDATE: Update the existing row with new values
    ///
    /// This corresponds to INSERT ... ON CONFLICT in PostgreSQL and
    /// INSERT ... ON DUPLICATE KEY UPDATE in MySQL.
    ///
    /// @signature (*targets: Column | ColumnRef | str)
    #[derive(Debug)]
    pub struct [] PyOnConflict(OnConflictState) as "OnConflict" {
        targets: Vec<sea_query::DynIden>,
        action: OnConflictAction,
        target_where: Option<PyExpr>,
        action_where: Option<PyExpr>,
    }
}

impl ToSeaQuery<sea_query::OnConflict> for OnConflictState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::OnConflict {
        let mut stmt = sea_query::OnConflict::columns(self.targets.clone());

        match &self.action {
            OnConflictAction::None => (),
            OnConflictAction::DoNothing(x) => {
                if x.is_empty() {
                    stmt.do_nothing();
                } else {
                    stmt.do_nothing_on(x.iter().cloned());
                }
            }
            OnConflictAction::DoUpdate(x) => {
                let mut columns = Vec::new();
                let mut exprs = Vec::new();

                for val in x.iter() {
                    match val {
                        OnConflictUpdate::Column(name) => columns.push(name.clone()),
                        OnConflictUpdate::Expr(name, expr) => {
                            exprs.push((name.clone(), expr.0.clone()));
                        }
                    }
                }

                stmt.update_columns(columns);
                stmt.values(exprs);
            }
        }

        if let Some(x) = &self.action_where {
            stmt.action_and_where(x.0.clone());
        }
        if let Some(x) = &self.target_where {
            stmt.target_and_where(x.0.clone());
        }

        stmt
    }
}

impl OnConflictState {
    #[inline]
    fn update_from_dictionary(
        &mut self,
        kwds: pyo3::Bound<'_, pyo3::types::PyDict>,
    ) -> pyo3::PyResult<()> {
        let mut actions = Vec::with_capacity(kwds.len());

        for (key, val) in kwds.iter() {
            unsafe {
                let val = crate::expression::PyExpr::try_from(&val)?;

                let action = OnConflictUpdate::Expr(
                    sea_query::Alias::new(key.extract::<String>().unwrap_unchecked()).into_iden(),
                    val,
                );
                actions.push(action);
            }
        }

        self.action = OnConflictAction::DoUpdate(actions);
        Ok(())
    }

    #[inline]
    fn update_from_tuple(
        &mut self,
        args: pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<()> {
        let mut actions = Vec::with_capacity(args.len());

        for key in args.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&key)?;
            match column_ref.name {
                Some(x) => actions.push(OnConflictUpdate::Column(x)),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "OnConflict cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        self.action = OnConflictAction::DoUpdate(actions);
        Ok(())
    }
}

#[pyo3::pymethods]
impl PyOnConflict {
    #[new]
    #[pyo3(signature=(*targets))]
    fn __new__(targets: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<Self> {
        if targets.is_empty() {
            let state = OnConflictState {
                targets: vec![],
                action: OnConflictAction::None,
                target_where: None,
                action_where: None,
            };
            return Ok(state.into());
        }

        let mut normalized_targets = Vec::with_capacity(targets.len());
        for item in targets.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&item)?;
            match column_ref.name {
                Some(x) => normalized_targets.push(x),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "OnConflict cannot accept asterisk '*' as target",
                    ))
                }
            }
        }

        let state = OnConflictState {
            targets: normalized_targets,
            action: OnConflictAction::None,
            target_where: None,
            action_where: None,
        };
        Ok(state.into())
    }

    /// Specify DO NOTHING action for conflicts.
    ///
    /// When a conflict occurs, the conflicting row will be skipped.
    ///
    /// `keys` parameter provides primary keys if you are using MySQL, for MySQL specific polyfill.
    ///
    /// @signature (self, *keys: Column | ColumnRef | str) -> typing.Self
    #[pyo3(signature=(*keys))]
    fn do_nothing<'a>(
        slf: pyo3::PyRef<'a, Self>,
        keys: &pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if keys.is_empty() {
            let mut lock = slf.0.lock();
            lock.action = OnConflictAction::DoNothing(vec![]);
            drop(lock);

            return Ok(slf);
        }

        let mut normalized_keys: Vec<sea_query::DynIden> = Vec::with_capacity(keys.len());
        for item in keys.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&item)?;
            match column_ref.name {
                Some(x) => normalized_keys.push(x),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "OnConflict cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        let mut lock = slf.0.lock();
        lock.action = OnConflictAction::DoNothing(normalized_keys);
        drop(lock);

        return Ok(slf);
    }

    /// Specify DO UPDATE action for conflicts using column names, or with explicit values.
    ///
    /// @overload (self, *args: Column | ColumnRef | str) -> typing.Self
    /// @overload (self, **kwds: object) -> typing.Self
    /// @signature (self, *args: Column | ColumnRef | str, **kwds: object) -> typing.Self
    #[pyo3(signature=(*args, **kwds))]
    fn do_update<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&'a pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !PyTupleMethods::is_empty(args) && kwds.is_some() {
            return Err(typeerror!(
                "cannot use both args and kwargs at the same time",
            ));
        }

        if !PyTupleMethods::is_empty(args) {
            let mut lock = slf.0.lock();
            lock.update_from_tuple(args.clone())?;
        } else if kwds.is_some() {
            let mut lock = slf.0.lock();
            lock.update_from_dictionary(kwds.unwrap().clone())?;
        } else {
            return Err(typeerror!("no arguments provided"));
        }

        Ok(slf)
    }

    /// Add a WHERE clause to the conflict target (partial unique index).
    ///
    /// @signature (self, condition: Expr) -> typing.Self
    fn target_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return Err(typeerror!(
                    "expected Expr, got {:?}",
                    condition.py(),
                    condition.as_ptr()
                ));
            }

            let mut lock = slf.0.lock();
            lock.target_where = Some(
                condition
                    .cast_unchecked::<crate::expression::PyExpr>()
                    .get()
                    .clone(),
            );
        }

        Ok(slf)
    }

    /// Add a WHERE clause to the conflict action (conditional update).
    ///
    /// @signature (self, condition: Expr) -> typing.Self
    fn action_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return Err(typeerror!(
                    "expected Expr, got {:?}",
                    condition.py(),
                    condition.as_ptr()
                ));
            }

            let mut lock = slf.0.lock();
            lock.action_where = Some(
                condition
                    .cast_unchecked::<crate::expression::PyExpr>()
                    .get()
                    .clone(),
            );
        }

        Ok(slf)
    }

    pub fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<OnConflict targets=[").unwrap();

        let n = lock.targets.len();
        for (index, tg) in lock.targets.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", tg.to_string()).unwrap();
            } else {
                write!(s, "{}, ", tg.to_string()).unwrap();
            }
        }
        write!(s, "]").unwrap();

        match &lock.action {
            OnConflictAction::DoNothing(_) => {
                write!(s, " (DO NOTHING)").unwrap();
            }
            OnConflictAction::DoUpdate(_) => {
                write!(s, " (DO UPDATE)").unwrap();
            }
            OnConflictAction::None => (),
        }

        if let Some(x) = &lock.target_where {
            write!(s, " target_where={}", x.__repr__()).unwrap();
        }
        if let Some(x) = &lock.action_where {
            write!(s, " action_where={}", x.__repr__()).unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
