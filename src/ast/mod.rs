pub mod data_type;

#[pyo3::pymodule(name = "ast")]
pub mod ast_module {
    #[pymodule_export]
    pub use super::data_type::data_type_module;
}
