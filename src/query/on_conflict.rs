use pyo3::types::{PyAnyMethods, PyDictMethods, PyTupleMethods};
use sea_query::IntoIden;

use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, RefBoundObject, ToSeaQuery};

#[derive(Debug, Clone)]
pub enum OnConflictUpdate {
    Column(sea_query::DynIden),
    Expr(sea_query::DynIden, PyExpr),
}

#[derive(Debug, Clone, Default)]
pub enum OnConflictAction {
    #[default]
    None,
    DoNothing(Vec<sea_query::DynIden>),
    DoUpdate(Vec<OnConflictUpdate>),
}

crate::implement_pyclass! {
    /// Specifies conflict resolution behavior for INSERT statements.
    ///
    /// Handles situations where an INSERT would violate a unique constraint
    /// or primary key.
    ///
    /// This corresponds to INSERT ... ON CONFLICT in PostgreSQL and
    /// INSERT ... ON DUPLICATE KEY UPDATE in MySQL.
    #[derive(Debug, Clone)]
    mutable [subclass] PyOnConflict(OnConflictState) as "OnConflict" {
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
                let val = PyExpr::try_from(&val)?;

                let action = OnConflictUpdate::Expr(
                    sea_query::Alias::new(key.extract::<String>().unwrap_unchecked()).into_iden(),
                    val,
                );
                actions.push(action);
            }
        }

        match &mut self.action {
            OnConflictAction::DoNothing(_) | OnConflictAction::None => {
                self.action = OnConflictAction::DoUpdate(actions);
            }
            OnConflictAction::DoUpdate(x) => {
                x.append(&mut actions);
            }
        }

        Ok(())
    }

    #[inline]
    fn update_from_tuple(&mut self, args: BoundArgs<'_>) -> pyo3::PyResult<()> {
        let mut actions = Vec::with_capacity(args.len());

        for key in args.iter() {
            let column_ref = PyColumnRef::try_from(&key)?;
            match column_ref.name {
                Some(x) => actions.push(OnConflictUpdate::Column(x)),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "OnConflict cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        match &mut self.action {
            OnConflictAction::DoNothing(_) | OnConflictAction::None => {
                self.action = OnConflictAction::DoUpdate(actions);
            }
            OnConflictAction::DoUpdate(x) => {
                x.append(&mut actions);
            }
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyOnConflict {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(*targets))]
    fn __init__(&self, targets: BoundArgs<'_>) -> pyo3::PyResult<()> {
        if targets.is_empty() {
            let state = OnConflictState {
                targets: vec![],
                action: OnConflictAction::None,
                target_where: None,
                action_where: None,
            };

            self.0.set(state);
            return Ok(());
        }

        let mut normalized_targets = Vec::with_capacity(targets.len());
        for item in targets.iter() {
            let column_ref = PyColumnRef::try_from(&item)?;
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
        self.0.set(state);
        Ok(())
    }

    /// Specify DO NOTHING action for conflicts.
    ///
    /// When a conflict occurs, the conflicting row will be skipped.
    ///
    /// `keys` parameter provides primary keys if you are using MySQL, for MySQL specific polyfill.
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
            let column_ref = PyColumnRef::try_from(&item)?;
            match column_ref.name {
                Some(x) => normalized_keys.push(x),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "OnConflict cannot accept asterisk '*' as column",
                    ))
                }
            }
        }

        {
            let mut lock = slf.0.lock();

            match &mut lock.action {
                OnConflictAction::DoUpdate(_) | OnConflictAction::None => {
                    lock.action = OnConflictAction::DoNothing(normalized_keys);
                }
                OnConflictAction::DoNothing(x) => {
                    x.append(&mut normalized_keys);
                }
            }
        }

        Ok(slf)
    }

    /// Specify DO UPDATE action for conflicts using column names, or with explicit values.
    #[pyo3(signature=(*args, **kwds))]
    fn do_update<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: BoundArgs<'a>,
        kwds: Option<BoundKwargs<'a>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !PyTupleMethods::is_empty(args) {
            let mut lock = slf.0.lock();
            lock.update_from_tuple(args)?;
        }
        if let Some(kwds) = kwds {
            let mut lock = slf.0.lock();
            lock.update_from_dictionary(kwds.clone())?;
        }

        Ok(slf)
    }

    /// Add a WHERE clause to the conflict target (partial unique index).
    fn target_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr, got {}",
                    crate::internal::get_type_name(slf.py(), condition.as_ptr())
                );
            }

            let mut lock = slf.0.lock();
            lock.target_where = Some(condition.cast_unchecked::<PyExpr>().get().clone());
        }

        Ok(slf)
    }

    /// Add a WHERE clause to the conflict action (conditional update).
    fn action_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr, got {}",
                    crate::internal::get_type_name(slf.py(), condition.as_ptr())
                );
            }

            let mut lock = slf.0.lock();
            lock.action_where = Some(condition.cast_unchecked::<PyExpr>().get().clone());
        }

        Ok(slf)
    }

    pub fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf);

        fmt.vec("targets", true)
            .quote_iter(lock.targets.iter().map(|x| x.to_string()))
            .finish(&mut fmt);

        match &lock.action {
            OnConflictAction::DoNothing(_) => {
                fmt.quote("action", "DO NOTHING");
            }
            OnConflictAction::DoUpdate(_) => {
                fmt.quote("action", "DO UPDATE");
            }
            OnConflictAction::None => (),
        }

        fmt.optional_map("target_where", lock.target_where.as_ref(), |x| x.__repr__())
            .optional_map("action_where", lock.action_where.as_ref(), |x| x.__repr__())
            .finish()
    }
}
