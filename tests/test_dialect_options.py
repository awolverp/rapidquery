from rapidquery.dialect import (
    OPT_BOOL_FORMAT_INTEGER,
    OPT_LIKE_ESCAPE_INCLUDING_ANY,
    OPT_LIKE_ESCAPE_SCALAR,
    OPT_PLACEHOLDER_NUMBERED,
    OPT_PLACEHOLDER_SIMPLE,
    Dialect,
    PostgreSql,
)


def test_getters_round_trip():
    dialect = Dialect(
        placeholder_character="$",
        placeholder=OPT_PLACEHOLDER_NUMBERED,
        like_escape=OPT_LIKE_ESCAPE_INCLUDING_ANY,
        bool_format=OPT_BOOL_FORMAT_INTEGER,
        supports_double_colon=True,
    )
    assert dialect.placeholder_character == "$"
    assert dialect.placeholder == OPT_PLACEHOLDER_NUMBERED
    assert dialect.like_escape == OPT_LIKE_ESCAPE_INCLUDING_ANY
    assert dialect.bool_format == OPT_BOOL_FORMAT_INTEGER
    assert dialect.supports_double_colon is True
    assert dialect.supports_similar_to is False
    assert dialect.parenthesize_in_between is False

    default = Dialect()
    assert default.placeholder_character == "?"
    assert default.placeholder == OPT_PLACEHOLDER_SIMPLE
    assert default.supports_double_colon is False


def test_builtin_getters():
    assert PostgreSql.placeholder_character == "$"
    assert PostgreSql.placeholder == OPT_PLACEHOLDER_NUMBERED
    assert PostgreSql.supports_double_colon is True
    assert PostgreSql.like_escape == OPT_LIKE_ESCAPE_SCALAR
