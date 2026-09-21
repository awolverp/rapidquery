pub mod base;
pub mod enum_;
pub mod extractors;
pub mod func;
pub mod ident;
pub mod nodes;
pub mod value;

#[pyo3::pymodule(name = "expr", gil_used = false)]
pub mod expr_mod {
    #[pymodule_export]
    use super::base::PyExpr;

    #[pymodule_export]
    use super::func::PyFunction;

    #[pymodule_export]
    use super::ident::{PyAsterisk, PyIdent, PyUnquote};

    #[pymodule_export]
    use super::value::PyValue;

    #[pymodule_export]
    use super::nodes::{
        PyBetweenExpr, PyBinaryExpr, PyCaseExpr, PyCastExpr, PyCustomExpr, PyInListExpr,
        PyIsDistinctFromExpr, PyIsExpr, PyIsNormalizedExpr, PyLikeExpr, PyRLikeExpr,
        PySimilarToExpr, PyTupleExpr, PyUnaryExpr,
    };
}
