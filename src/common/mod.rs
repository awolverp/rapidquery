pub mod column;
pub mod column_ref;
pub mod expression;
pub mod foreign_key;
pub mod table_ref;
pub mod value;

#[pyo3::pymodule(name = "common")]
pub mod common_module {
    #[pymodule_export]
    use super::value::PyValue;

    #[pymodule_export]
    use super::column_ref::PyColumnRef;

    #[pymodule_export]
    use super::table_ref::PyTableName;

    #[pymodule_export]
    use super::expression::all;
    #[pymodule_export]
    use super::expression::any;
    #[pymodule_export]
    use super::expression::not_;
    #[pymodule_export]
    use super::expression::PyExpr;
    #[pymodule_export]
    use super::expression::PyFunc;

    #[pymodule_export]
    use super::column::PyColumn;

    #[pymodule_export]
    use super::foreign_key::PyForeignKey;
}
