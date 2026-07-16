import pytest

from rapidquery._native.dialect import (
    BigQuery,
    Databricks,
    DuckDb,
    Hive,
    MsSql,
    MySql,
    Oracle,
    PostgreSql,
    Redshift,
    Snowflake,
    Spark,
    Sqlite,
    Teradata,
)

ALL_DIALECTS = [
    BigQuery,
    Databricks,
    DuckDb,
    Hive,
    MsSql,
    MySql,
    Oracle,
    PostgreSql,
    Redshift,
    Snowflake,
    Spark,
    Sqlite,
    Teradata,
]
ALL_DIALECT_NAMES = [
    "BigQuery",
    "Databricks",
    "DuckDb",
    "Hive",
    "MsSql",
    "MySql",
    "Oracle",
    "PostgreSql",
    "Redshift",
    "Snowflake",
    "Spark",
    "Sqlite",
    "Teradata",
]


@pytest.fixture(params=ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def dialect_cls(request):
    """Parametrized fixture yielding every concrete SqlDialect subclass."""
    return request.param


@pytest.fixture(params=ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def dialect_instance(request):
    """Parametrized fixture yielding an *instance* of every SqlDialect subclass."""
    return request.param()


@pytest.fixture
def postgres():
    return PostgreSql


@pytest.fixture
def mysql():
    return MySql
