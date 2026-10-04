import pytest
from rapidquery._core import _feature_only_stdlib_json

from rapidquery.dialect import (
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
    Dialect,
    Sqlite,
    Teradata,
)

try:
    import orjson

    USING_ORJSON = not _feature_only_stdlib_json
except ImportError:
    USING_ORJSON = False


# Builtin dialects only. GenericDialect needs every field as a keyword argument;
# tests that want one call `generic_dialect()`.
ALL_DIALECT_CLASSES: list[type[Dialect]] = [
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
ALL_DIALECT_NAMES = [cls.__name__ for cls in ALL_DIALECT_CLASSES]
ALL_DIALECTS = tuple(cls() for cls in ALL_DIALECT_CLASSES)


@pytest.fixture(params=ALL_DIALECT_CLASSES, ids=ALL_DIALECT_NAMES)
def dialect_cls(request: pytest.FixtureRequest) -> type[Dialect]:
    """Every builtin SqlDialect subclass (not GenericDialect)."""
    return request.param


@pytest.fixture(params=ALL_DIALECT_CLASSES, ids=ALL_DIALECT_NAMES)
def dialect_instance(request: pytest.FixtureRequest) -> Dialect:
    """An instance of every builtin SqlDialect subclass."""
    return request.param()
