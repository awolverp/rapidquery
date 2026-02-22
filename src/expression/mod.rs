mod expr;
mod func;

pub use expr::*;
pub use func::*;

/// Create a logical AND condition that is true only if all conditions are true.
///
/// This is equivalent to SQL's AND operator applied to multiple expressions.
///
/// @signature (arg1: Expr, *args: Expr) -> Expr
#[pyo3::pyfunction]
#[pyo3(signature=(arg1, *args))]
pub fn all(
    arg1: pyo3::Bound<'_, PyExpr>,
    args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
    let py = arg1.py();
    let mut expr = arg1.unbind();

    for m in args {
        let m = m.cast_into_exact::<PyExpr>()?;

        let result = sea_query::ExprTrait::and(expr.get().0.clone(), m.get().0.clone());
        expr = pyo3::Py::new(py, PyExpr(result))?;
    }

    Ok(expr.into_any())
}

/// Create a logical OR condition that is true if any condition is true.
///
/// This is equivalent to SQL's OR operator applied to multiple expressions.
///
/// @signature (arg1: Expr, *args: Expr) -> Expr
#[pyo3::pyfunction]
#[pyo3(signature=(arg1, *args))]
pub fn any(
    arg1: pyo3::Bound<'_, PyExpr>,
    args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
    let py = arg1.py();
    let mut expr = arg1.unbind();

    for m in args {
        let m = m.cast_into_exact::<PyExpr>()?;

        let result = sea_query::ExprTrait::or(expr.get().0.clone(), m.get().0.clone());
        expr = pyo3::Py::new(py, PyExpr(result))?;
    }

    Ok(expr.into_any())
}

/// Create a logical NOT.
///
/// @signature (arg: Expr) -> Expr
#[pyo3::pyfunction]
pub fn not_(arg: &pyo3::Bound<'_, PyExpr>) -> PyExpr {
    sea_query::ExprTrait::not(arg.get().0.clone()).into()
}
