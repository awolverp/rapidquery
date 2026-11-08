import importlib.util
import os

import pytest
from hypothesis import settings
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

settings.register_profile("default", max_examples=50, deadline=None)
settings.register_profile("slow", max_examples=500, deadline=None)
settings.load_profile(os.environ.get("HYPOTHESIS_PROFILE", "default"))


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
