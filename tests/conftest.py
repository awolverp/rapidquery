import pytest

from rapidquery.dialect import (
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

ALL_DIALECTS = [
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
    SQLite,
    Teradata,
]
ALL_DIALECT_NAMES = [cls.__name__ for cls in ALL_DIALECTS]


@pytest.fixture(params=ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def dialect_cls(request: pytest.FixtureRequest) -> type[SqlDialect]:
    """Every concrete SqlDialect subclass."""
    return request.param


@pytest.fixture(params=ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def dialect_instance(request: pytest.FixtureRequest) -> SqlDialect:
    """An instance of every concrete SqlDialect subclass."""
    return request.param()
