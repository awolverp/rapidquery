"""
SQL dialect markers.
"""

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
