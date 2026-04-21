use crate::common::expression::PyExpr;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, RefBoundObject, ToSeaQuery};

crate::implement_pyclass! {
    mutable [subclass] PyCaseStatement(CaseStatementState) as "CaseStatement" {
        pub when: Vec<(PyExpr, PyExpr)>,
        pub r#else: Option<PyExpr>,
    }
}

impl Clone for CaseStatementState {
    fn clone(&self) -> Self {
        Self {
            when: self.when.clone(),
            r#else: self.r#else.clone(),
        }
    }
}

impl ToSeaQuery<sea_query::CaseStatement> for CaseStatementState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::CaseStatement {
        let mut stmt = sea_query::CaseStatement::new();

        for (x, y) in self.when.iter() {
            stmt = stmt.case(x.0.clone(), y.0.clone());
        }

        if let Some(x) = &self.r#else {
            stmt = stmt.finally(x.0.clone());
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyCaseStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    pub fn __init__(&self) -> pyo3::PyResult<()> {
        self.0.set(CaseStatementState {
            when: vec![],
            r#else: None,
        });
        Ok(())
    }

    fn when<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
        result: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr as condition, got {}",
                    crate::internal::get_type_name(condition.py(), condition.as_ptr())
                );
            }

            let condition = condition.cast_unchecked::<PyExpr>().get().clone();
            let result = PyExpr::try_from(result)?;

            slf.0.lock().when.push((condition, result));
        }

        Ok(slf)
    }

    fn else_<'a>(
        slf: pyo3::PyRef<'a, Self>,
        result: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let result = PyExpr::try_from(result)?;
        slf.0.lock().r#else = Some(result);

        Ok(slf)
    }

    /// Shorthand for `Expr(self)`
    fn to_expr(&self, py: pyo3::Python) -> PyExpr {
        let stmt = self.0.lock().to_sea_query(py);
        PyExpr(sea_query::SimpleExpr::Case(Box::new(stmt)))
    }

    /// Shorthand for `SelectLabel(self, alias, window)`
    #[pyo3(signature=(alias, window=None))]
    fn label(
        &self,
        py: pyo3::Python,
        alias: String,
        window: Option<RefBoundObject<'_>>,
    ) -> pyo3::PyResult<crate::query::select::PySelectLabel> {
        let window = match window {
            Some(x) => Some(crate::query::select::SelectLabelWindow::try_from(x)?),
            None => None,
        };
        let expr = self.to_expr(py);

        let state = crate::query::select::SelectLabelState {
            expr,
            alias: Some(alias),
            window,
        };
        Ok(state.into())
    }

    fn __copy__(&self) -> Self {
        let lock = self.0.lock();
        lock.clone().into()
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();
        let mut fmt = ReprFormatter::new_with_pyref(&slf);

        fmt.vec("when", false)
            .display_iter(
                lock.when
                    .iter()
                    .map(|x| format!("{} => {}", x.0.__repr__(), x.1.__repr__())),
            )
            .finish(&mut fmt);

        fmt.optional_map("else", lock.r#else.as_ref(), |x| x.__repr__())
            .finish()
    }
}
