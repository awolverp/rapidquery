#![allow(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "nightly", feature(optimize_attribute))]

pub mod data_type;
pub mod dialect;
pub mod expr;
pub mod internal;
pub mod sql;
pub mod typeref;

// Tasks until being stable (until this part)
// - Add lint/coding-style rules
// - Restore GenericDialect ( very helpful for testing )
// - Test and coverage everything and edge cases
// - Update docstrings / doccomments

// Later tasks:
// - Complete `Expr` methods
// - Test `Expr` methods
// - Benchmark and compare with old version
// - Open PR
// - Start writing query ASTs.

/// RapidQuery native extension.
#[pyo3::pymodule(gil_used = false)]
mod _core {
    #[pymodule_export]
    use super::data_type::data_type_mod;

    #[pymodule_export]
    use super::dialect::dialect_mod;

    #[pymodule_export]
    use super::expr::expr_mod;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        use crate::internal::utils::PyModuleSubmoduleExt;
        use pyo3::types::PyModuleMethods;

        crate::typeref::initialize_typeref(m.py());

        m.add(
            "_feature_only_stdlib_json",
            cfg_select! {
                feature = "only-stdlib-json" => true,
                not(feature = "only-stdlib-json") => false,
            },
        )?;
        m.add(
            "_feature_only_orjson",
            cfg_select! {
                feature = "only-orjson" => true,
                not(feature = "only-orjson") => false,
            },
        )?;

        m.init_submodules()
    }
}
