pub mod errors;
pub mod flags;

mod base;
mod builtin;

pub use base::PyDialect;
pub use builtin::BIGQUERY_DIALECT;
pub use builtin::DATABRICKS_DIALECT;
pub use builtin::DUCKDB_DIALECT;
pub use builtin::HIVE_DIALECT;
pub use builtin::MSSQL_DIALECT;
pub use builtin::MYSQL_DIALECT;
pub use builtin::ORACLE_DIALECT;
pub use builtin::POSTGRESQL_DIALECT;
pub use builtin::REDSHIFT_DIALECT;
pub use builtin::SNOWFLAKE_DIALECT;
pub use builtin::SPARK_DIALECT;
pub use builtin::SQLITE_DIALECT;
pub use builtin::TERADATA_DIALECT;

#[pyo3::pymodule(name = "dialect", gil_used = false)]
pub mod dialect_mod {
    #[pymodule_export]
    use super::PyDialect;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const BigQuery: super::PyDialect = super::BIGQUERY_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Databricks: super::PyDialect = super::DATABRICKS_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const DuckDb: super::PyDialect = super::DUCKDB_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Hive: super::PyDialect = super::HIVE_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const MsSql: super::PyDialect = super::MSSQL_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const MySql: super::PyDialect = super::MYSQL_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Oracle: super::PyDialect = super::ORACLE_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const PostgreSql: super::PyDialect = super::POSTGRESQL_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Redshift: super::PyDialect = super::REDSHIFT_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Snowflake: super::PyDialect = super::SNOWFLAKE_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Spark: super::PyDialect = super::SPARK_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Sqlite: super::PyDialect = super::SQLITE_DIALECT;

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const Teradata: super::PyDialect = super::TERADATA_DIALECT;
}
