pub mod alter_table;
pub mod base;
pub mod index;
pub mod table;
pub mod table_operations;

#[pyo3::pymodule(name = "schema")]
pub mod schema_module {
    use pyo3::types::PyModuleMethods;

    #[pymodule_export]
    use super::base::PySchemaStatement;

    #[pymodule_export]
    use super::alter_table::PyAlterTable;
    #[pymodule_export]
    use super::alter_table::PyAlterTableAddColumnOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableAddForeignKeyOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableBaseOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableDropColumnOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableDropForeignKeyOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableModifyColumnOption;
    #[pymodule_export]
    use super::alter_table::PyAlterTableRenameColumnOption;

    #[pymodule_export]
    use super::index::PyDropIndex;
    #[pymodule_export]
    use super::index::PyIndex;
    #[pymodule_export]
    use super::index::PyIndexColumn;

    #[pymodule_export]
    use super::table::PyTable;

    #[pymodule_export]
    use super::table_operations::PyDropTable;
    #[pymodule_export]
    use super::table_operations::PyRenameTable;
    #[pymodule_export]
    use super::table_operations::PyTruncateTable;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add(
            "__stub_imports__",
            vec!["from .common import Column, ColumnRef, TableName, ForeignKey, Expr"],
        )
    }
}
