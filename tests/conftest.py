import pytest
from rapidquery._core import _feature_only_stdlib_json

from rapidquery.dialect import (
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

try:
    import orjson

    USING_ORJSON = not _feature_only_stdlib_json
except ImportError:
    USING_ORJSON = False


# Builtin dialects only. GenericDialect needs every field as a keyword argument;
# tests that want one call `generic_dialect()`.
ALL_DIALECT_CLASSES: list[type[SqlDialect]] = [
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

# Defaults match src/dialect/inner.rs `base()` used by the Rust unit tests.
_GENERIC_DEFAULTS = dict(
    placeholder_sigil=":",
    bit_format=None,
    supports_array=False,
    supports_inet=False,
    supports_mac_address=False,
    null_treatment_syntax="OutsideParens",
    like_any_syntax="OrExpansion",
    bool_format="Keyword",
    identifier_format="DoubleQuote",
    identifier_escape="Delimiters",
    string_format="SingleQuote",
    string_escape="Quote",
    bytes_format="PrefixBigX",
)


def generic_dialect(**overrides) -> GenericDialect:
    """Build a GenericDialect; pass only the fields the test cares about."""
    return GenericDialect(**{**_GENERIC_DEFAULTS, **overrides})


@pytest.fixture(params=ALL_DIALECT_CLASSES, ids=ALL_DIALECT_NAMES)
def dialect_cls(request: pytest.FixtureRequest) -> type[SqlDialect]:
    """Every builtin SqlDialect subclass (not GenericDialect)."""
    return request.param


@pytest.fixture(params=ALL_DIALECT_CLASSES, ids=ALL_DIALECT_NAMES)
def dialect_instance(request: pytest.FixtureRequest) -> SqlDialect:
    """An instance of every builtin SqlDialect subclass."""
    return request.param()
