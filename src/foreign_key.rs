#[derive(Debug, Clone, Copy)]
pub struct ForeignKeyActionAlias(sea_query::ForeignKeyAction);

implement_state_pyclass! {
    /// Specifies a foreign key relationship between tables.
    ///
    /// Defines referential integrity constraints including:
    /// - Source columns (in the child table)
    /// - Target columns (in the parent table)
    /// - Actions for updates and deletes (CASCADE, RESTRICT, SET NULL, etc.)
    /// - Optional naming for the constraint
    ///
    /// Foreign keys ensure data consistency by requiring that values in the
    /// child table's columns match existing values in the parent table's columns.
    ///
    /// @signature (
    ///     from_columns: typing.Sequence[str | ColumnRef | Column],
    ///     to_table: TableName | str,
    ///     to_columns: typing.Sequence[str | ColumnRef | Column],
    ///     name: str | None = None,
    ///     *,
    ///     on_delete: str | None = None,
    ///     on_update: str | None = None,
    /// )
    pub struct [] PyForeignKey(ForeignKeyState) as "ForeignKey" {
        /// Foreign key constraint name
        pub name: String,

        /// To table
        pub to_table: pyo3::Py<crate::common::PyTableName>,

        /// To columns
        pub to_columns: Vec<String>,

        /// From table
        pub from_table: Option<pyo3::Py<crate::common::PyTableName>>,

        /// From columns
        pub from_columns: Vec<String>,

        /// On delete action
        pub on_delete: Option<ForeignKeyActionAlias>,

        /// On update action
        pub on_update: Option<ForeignKeyActionAlias>,
    }
}
