//! All SQL types with their serialization and deserialization methods.
//!
//! With support for custom types for Python.

mod abstracts;
mod binary;
mod datetimes;
mod json;
mod others;
mod primitives;
mod vector;

pub use abstracts::*;
pub use binary::*;
pub use datetimes::*;
pub use json::*;
pub use others::*;
pub use primitives::*;
pub use vector::*;

#[pyo3::pymodule(name = "sqltypes")]
pub mod sqltypes_module {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    // sqltypes::abstracts
    #[pymodule_export]
    use super::PySQLTypeAbstract;

    // sqltypes::binary
    #[pymodule_export]
    use super::PyBinaryType;
    #[pymodule_export]
    use super::PyBitType;
    #[pymodule_export]
    use super::PyBlobType;
    #[pymodule_export]
    use super::PyVarBinaryType;
    #[pymodule_export]
    use super::PyVarBitType;

    // sqltypes::datetimes
    #[pymodule_export]
    use super::PyDateTimeType;
    #[pymodule_export]
    use super::PyDateType;
    #[pymodule_export]
    use super::PyTimeType;
    #[pymodule_export]
    use super::PyTimestampType;

    // sqltypes::json
    #[pymodule_export]
    use super::PyJSONBinaryType;
    #[pymodule_export]
    use super::PyJSONType;

    // sqltypes::vector
    #[pymodule_export]
    use super::PyVectorType;

    // sqltypes::others
    #[pymodule_export]
    use super::PyArrayType;
    #[pymodule_export]
    use super::PyDecimalType;
    #[pymodule_export]
    use super::PyEnumType;
    #[pymodule_export]
    use super::PyINETType;
    #[pymodule_export]
    use super::PyMacAddressType;
    #[pymodule_export]
    use super::PyUUIDType;

    // sqltypes::primitives
    #[pymodule_export]
    use super::PyBigIntegerType;
    #[pymodule_export]
    use super::PyBigUnsignedType;
    #[pymodule_export]
    use super::PyBooleanType;
    #[pymodule_export]
    use super::PyCharType;
    #[pymodule_export]
    use super::PyDoubleType;
    #[pymodule_export]
    use super::PyFloatType;
    #[pymodule_export]
    use super::PyIntegerType;
    #[pymodule_export]
    use super::PySmallIntegerType;
    #[pymodule_export]
    use super::PySmallUnsignedType;
    #[pymodule_export]
    use super::PyStringType;
    #[pymodule_export]
    use super::PyTextType;
    #[pymodule_export]
    use super::PyTinyIntegerType;
    #[pymodule_export]
    use super::PyTinyUnsignedType;
    #[pymodule_export]
    use super::PyUnsignedType;
}
