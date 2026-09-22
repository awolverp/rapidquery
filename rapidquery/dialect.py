"""
SQL dialect markers.
"""

from ._core.dialect import (
    BigQuery,
    Databricks,
    DuckDb,
    GenericDialect,
    Hive,
    MsSql,
    MySql,
    Oracle,
    PostgreSql,
    Redshift,
    Snowflake,
    Spark,
    SqlDialect,
    Sqlite,
    Teradata,
)

__all__ = [
    "BigQuery",
    "Databricks",
    "DuckDb",
    "GenericDialect",
    "Hive",
    "MsSql",
    "MySql",
    "Oracle",
    "PostgreSql",
    "Redshift",
    "Snowflake",
    "Spark",
    "SqlDialect",
    "Sqlite",
    "Teradata",
]
