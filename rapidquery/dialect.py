"""
SQL dialect markers.
"""

# TODO: rename PostgreSQL to PostgreSql, SQLite to Sqlite, MSSQL to MsSql, DuckDB to DuckDb

from ._core.dialect import (
    MSSQL,
    BigQuery,
    Databricks,
    DuckDB,
    Hive,
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
    "MSSQL",
    "BigQuery",
    "Databricks",
    "DuckDB",
    "Hive",
    "MySql",
    "Oracle",
    "PostgreSQL",
    "Redshift",
    "SQLite",
    "Snowflake",
    "Spark",
    "SqlDialect",
    "Teradata",
]
