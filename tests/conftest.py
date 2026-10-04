import importlib.util

import pytest
from rapidquery._core import _feature_only_stdlib_json

from rapidquery.dialect import (
    BigQuery,
    Databricks,
    Dialect,
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

USING_ORJSON = importlib.util.find_spec("orjson") is not None and not _feature_only_stdlib_json


ALL_DIALECTS: list[Dialect] = [
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


@pytest.fixture(params=ALL_DIALECTS)
def dialect(request: pytest.FixtureRequest) -> Dialect:
    """Every builtin Dialects."""
    return request.param
