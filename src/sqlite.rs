/// SQLite-Only functions
///
/// **Comming Soon ...**
#[pyo3::pymodule(name = "sqlite")]
pub mod sqlite_module {
    use pyo3::types::PyModuleMethods;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add("__stub_imports__", vec!["from .common import Expr"])
    }
}
