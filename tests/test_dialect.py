import pytest

from rapidquery.dialect import (
    OPT_BIT_FORMAT_POSTFIX_BIG_B,
    OPT_BITWISE_CALL_SYNTAX_INFIX,
    OPT_BITWISE_OPERATIONS_AND,
    OPT_BITWISE_OPERATIONS_NOT,
    OPT_BITWISE_OPERATIONS_OR,
    OPT_BITWISE_OPERATIONS_SHL,
    OPT_BITWISE_OPERATIONS_SHR,
    OPT_BITWISE_XOR_SPELLING_CARET,
    OPT_BOOL_FORMAT_INTEGER,
    OPT_BYTES_FORMAT_SLASH_X_BYTEA,
    OPT_IDENTIFIER_ESCAPE_BACKSLASH,
    OPT_IDENTIFIER_FORMAT_BACKTICK,
    OPT_LIKE_ESCAPE_ANY,
    OPT_LIKE_ESCAPE_SCALAR,
    OPT_PLACEHOLDER_SYNTAX_NUMBERED,
    OPT_PLACEHOLDER_SYNTAX_SIMPLE,
    OPT_STRING_ESCAPE_BACKSLASH_HEX,
    OPT_STRING_FORMAT_PREFIX_BIG_E,
    Dialect,
    PostgreSql,
)


def test_getters_round_trip():
    dialect = Dialect(
        placeholder_character="$",
        placeholder_syntax=OPT_PLACEHOLDER_SYNTAX_NUMBERED,
        like_escape=OPT_LIKE_ESCAPE_SCALAR | OPT_LIKE_ESCAPE_ANY,
        bool_format=OPT_BOOL_FORMAT_INTEGER,
        cast_double_colon=True,
    )
    assert dialect.placeholder_character == "$"
    assert dialect.placeholder_syntax == OPT_PLACEHOLDER_SYNTAX_NUMBERED
    assert dialect.like_escape == OPT_LIKE_ESCAPE_SCALAR | OPT_LIKE_ESCAPE_ANY
    assert dialect.bool_format == OPT_BOOL_FORMAT_INTEGER
    assert dialect.cast_double_colon is True
    assert dialect.supports_similar_to is False

    default = Dialect()
    assert default.placeholder_character == "?"
    assert default.placeholder_syntax == OPT_PLACEHOLDER_SYNTAX_SIMPLE
    assert default.cast_double_colon is False
    assert default.bitwise_operations == (
        OPT_BITWISE_OPERATIONS_AND
        | OPT_BITWISE_OPERATIONS_OR
        | OPT_BITWISE_OPERATIONS_NOT
        | OPT_BITWISE_OPERATIONS_SHL
        | OPT_BITWISE_OPERATIONS_SHR
    )
    assert default.bitwise_call_syntax == OPT_BITWISE_CALL_SYNTAX_INFIX
    assert default.bitwise_xor_spelling == OPT_BITWISE_XOR_SPELLING_CARET
    assert default.like_escape == 1
    assert default.tuple_arity == 3

    with pytest.raises(ValueError, match="bitwise_operations"):
        Dialect(bitwise_operations=1 << 7)


def test_builtin_getters():
    assert PostgreSql.placeholder_character == "$"
    assert PostgreSql.placeholder_syntax == OPT_PLACEHOLDER_SYNTAX_NUMBERED
    assert PostgreSql.cast_double_colon is True
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

    assert default.write_identifier("lov`e") == "`lov\\`e`"

    assert default.write_bits("10101") == "'10101'B"

    with pytest.raises(ValueError):
        _ = default.write_bits("123")

    assert default.write_bytes(b"love") == "'\\x6C6F7665'::bytea"
