"""SQL dialect markers.

Pass either the class or an instance wherever a dialect is required:

    from rapidquery.dialect import DuckDB, PostgreSQL

    query.prepare(PostgreSQL)
    query.prepare(DuckDB())
"""

from ._core.dialect import (
    BigQuery,
    Databricks,
    DuckDB,
    Hive,
    MSSQL,
    MySql,
    Oracle,
    PostgreSQL,
    Redshift,
    Snowflake,
    Spark,
    SqlDialect,
    SQLite,
    Teradata,
)

__all__ = [
    "BigQuery",
    "Databricks",
    "DuckDB",
    "Hive",
    "MSSQL",
    "MySql",
    "Oracle",
    "PostgreSQL",
    "Redshift",
    "Snowflake",
    "Spark",
    "SqlDialect",
    "SQLite",
    "Teradata",
]
