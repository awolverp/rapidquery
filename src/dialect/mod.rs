mod base;
mod escape;

pub use base::*;

#[pyo3::pymodule(name = "dialect", gil_used = false)]
pub mod dialect_mod {
    #[pymodule_export]
    use super::AggregateConfig;
    #[pymodule_export]
    use super::BitwiseConfig;
    #[pymodule_export]
    use super::CastConfig;
    #[pymodule_export]
    use super::LikeConfig;
    #[pymodule_export]
    use super::LiteralConfig;
    #[pymodule_export]
    use super::OperatorConfig;
    #[pymodule_export]
    use super::PyDialect;
    #[pymodule_export]
    use super::SchemaConfig;
    #[pymodule_export]
    use super::StringConfig;
    #[pymodule_export]
    use super::TemporalConfig;
    #[pymodule_export]
    use super::TupleConfig;

    #[pymodule_init]
    fn init(module: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        super::export_options(module)
    }
}
