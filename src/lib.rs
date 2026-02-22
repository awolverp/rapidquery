#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::too_many_arguments)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(likely_unlikely)]
#![feature(optimize_attribute)]
#![feature(once_cell_try)]

#[macro_use]
mod macros;

mod column;
mod common;
mod expression;
mod foreign_key;
mod parameters;
mod sqltypes;
mod typeref;
mod value;

/// RapidQuery core module written in Rust
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    use pyo3::types::PyModuleMethods;

    // sqltypes::abstracts
    #[pymodule_export]
    use crate::sqltypes::PySQLTypeAbstract;

    // sqltypes::binary
    #[pymodule_export]
    use crate::sqltypes::PyBinaryType;
    #[pymodule_export]
    use crate::sqltypes::PyBitType;
    #[pymodule_export]
    use crate::sqltypes::PyBlobType;
    #[pymodule_export]
    use crate::sqltypes::PyVarBinaryType;
    #[pymodule_export]
    use crate::sqltypes::PyVarBitType;

    // sqltypes::datetimes
    #[pymodule_export]
    use crate::sqltypes::PyDateTimeType;
    #[pymodule_export]
    use crate::sqltypes::PyDateType;
    #[pymodule_export]
    use crate::sqltypes::PyTimeType;
    #[pymodule_export]
    use crate::sqltypes::PyTimestampType;

    // sqltypes::json
    #[pymodule_export]
    use crate::sqltypes::PyJSONBinaryType;
    #[pymodule_export]
    use crate::sqltypes::PyJSONType;

    // sqltypes::vector
    #[pymodule_export]
    use crate::sqltypes::PyVectorType;

    // sqltypes::others
    #[pymodule_export]
    use crate::sqltypes::PyArrayType;
    #[pymodule_export]
    use crate::sqltypes::PyDecimalType;
    #[pymodule_export]
    use crate::sqltypes::PyEnumType;
    #[pymodule_export]
    use crate::sqltypes::PyINETType;
    #[pymodule_export]
    use crate::sqltypes::PyMacAddressType;
    #[pymodule_export]
    use crate::sqltypes::PyUUIDType;

    // sqltypes::primitives
    #[pymodule_export]
    use crate::sqltypes::PyBigIntegerType;
    #[pymodule_export]
    use crate::sqltypes::PyBigUnsignedType;
    #[pymodule_export]
    use crate::sqltypes::PyBooleanType;
    #[pymodule_export]
    use crate::sqltypes::PyCharType;
    #[pymodule_export]
    use crate::sqltypes::PyDoubleType;
    #[pymodule_export]
    use crate::sqltypes::PyFloatType;
    #[pymodule_export]
    use crate::sqltypes::PyIntegerType;
    #[pymodule_export]
    use crate::sqltypes::PySmallIntegerType;
    #[pymodule_export]
    use crate::sqltypes::PySmallUnsignedType;
    #[pymodule_export]
    use crate::sqltypes::PyStringType;
    #[pymodule_export]
    use crate::sqltypes::PyTextType;
    #[pymodule_export]
    use crate::sqltypes::PyTinyIntegerType;
    #[pymodule_export]
    use crate::sqltypes::PyTinyUnsignedType;
    #[pymodule_export]
    use crate::sqltypes::PyUnsignedType;

    // value
    #[pymodule_export]
    use crate::value::PyValue;

    // common
    #[pymodule_export]
    use crate::common::PyColumnRef;
    #[pymodule_export]
    use crate::common::PyQueryStatement;
    #[pymodule_export]
    use crate::common::PySchemaStatement;
    #[pymodule_export]
    use crate::common::PyTableName;
    #[pymodule_export]
    use crate::common::Py_AsteriskType;

    // expression
    #[pymodule_export]
    use crate::expression::all;
    #[pymodule_export]
    use crate::expression::any;
    #[pymodule_export]
    use crate::expression::not_;
    #[pymodule_export]
    use crate::expression::PyExpr;
    #[pymodule_export]
    use crate::expression::PyFunc;

    // column
    #[pymodule_export]
    use crate::column::PyColumn;

    // foreign_key
    #[pymodule_export]
    use crate::foreign_key::PyForeignKey;

    /// @constant
    #[pymodule_export]
    const ASTERISK: Py_AsteriskType = Py_AsteriskType;

    /// @constant
    #[pymodule_export]
    const COLUMN_OPT_PRIMARY_KEY: u8 = crate::column::COLUMN_OPT_PRIMARY_KEY;

    /// @constant
    #[pymodule_export]
    const COLUMN_OPT_UNIQUE_KEY: u8 = crate::column::COLUMN_OPT_UNIQUE_KEY;

    /// @constant
    #[pymodule_export]
    const COLUMN_OPT_NULLABLE: u8 = crate::column::COLUMN_OPT_NULLABLE;

    /// @constant
    #[pymodule_export]
    const COLUMN_OPT_AUTO_INCREMENT: u8 = crate::column::COLUMN_OPT_AUTO_INCREMENT;

    /// @constant
    #[pymodule_export]
    const COLUMN_OPT_STORED_GENERATED: u8 = crate::column::COLUMN_OPT_STORED_GENERATED;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add(
            "__stub_imports__",
            vec!["import decimal", "import uuid", "import datetime", "import enum"],
        )?;

        crate::typeref::initialize_typeref(m.py());
        Ok(())
    }
}
