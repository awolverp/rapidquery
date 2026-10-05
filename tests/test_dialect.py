import pytest

from rapidquery.dialect import (
    OPT_BIT_FORMAT_POSTFIX_BIG_B,
    OPT_BOOL_FORMAT_INTEGER,
    OPT_BYTES_FORMAT_SLASH_X_BYTEA,
    OPT_IDENTIFIER_ESCAPE_BACKSLASH,
    OPT_IDENTIFIER_FORMAT_BACKTICK,
    OPT_LIKE_ESCAPE_INCLUDING_ANY,
    OPT_LIKE_ESCAPE_SCALAR,
    OPT_PLACEHOLDER_NUMBERED,
    OPT_PLACEHOLDER_SIMPLE,
    OPT_STRING_ESCAPE_BACKSLASH_HEX,
    OPT_STRING_FORMAT_PREFIX_BIG_E,
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


# Edge cases are tested in Rust
def test_dialect_methods():
    default = Dialect(
        bool_format=OPT_BOOL_FORMAT_INTEGER,
        string_format=OPT_STRING_FORMAT_PREFIX_BIG_E,
        string_escape=OPT_STRING_ESCAPE_BACKSLASH_HEX,
        identifier_format=OPT_IDENTIFIER_FORMAT_BACKTICK,
        identifier_escape=OPT_IDENTIFIER_ESCAPE_BACKSLASH,
        bit_format=OPT_BIT_FORMAT_POSTFIX_BIG_B,
        bytes_format=OPT_BYTES_FORMAT_SLASH_X_BYTEA,
    )
    assert default.write_boolean(True) == "1"

    assert default.write_string_literal("love") == "E'love'"

    assert default.escape_string("\r") == "\\r"
    assert default.escape_string("\t") == "\\t"
    assert default.escape_string("\\") == "\\\\"

    assert default.write_identifier("lov`e") == "`lov\\`e`"

    assert default.write_bits("10101") == "'10101'B"

    with pytest.raises(ValueError):
        _ = default.write_bits("123")

    assert default.write_bytes(b"love") == "'\\x6C6F7665'::bytea"
