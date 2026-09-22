"""Python API for SqlDialect, the 13 builtins, and GenericDialect.

Escape and format algorithms are tested in Rust (`src/dialect/inner.rs`).
These tests cover property wiring, Python string conversion, and a smoke
check of each public method.
"""

import pytest

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

from .conftest import ALL_DIALECT_CLASSES, ALL_DIALECT_NAMES, generic_dialect

# Expected DialectConfig for each builtin (src/dialect/builtin.rs).
BUILTIN_CONFIG = {
    BigQuery: dict(
        placeholder_sigil="@",
        bit_format=None,
        supports_array=True,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="InsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Keyword",
        identifier_format="Backtick",
        identifier_escape="SameAsString",
        string_format="SingleQuote",
        string_escape="BackslashHex",
        bytes_format="PrefixBigBSlashX",
    ),
    Databricks: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="PatternList",
        bool_format="Keyword",
        identifier_format="Backtick",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="BackslashUnicodeQuote",
        bytes_format="PrefixBigX",
    ),
    DuckDb: dict(
        placeholder_sigil="$",
        bit_format="PrefixBigB",
        supports_array=True,
        supports_inet=True,
        supports_mac_address=False,
        null_treatment_syntax="InsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Keyword",
        identifier_format="DoubleQuote",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="Quote",
        bytes_format="PrefixBigX",
    ),
    Hive: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Keyword",
        identifier_format="Backtick",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="BackslashUnicode",
        bytes_format="Unhex",
    ),
    MsSql: dict(
        placeholder_sigil="@",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Int",
        identifier_format="SquareBracket",
        identifier_escape="Delimiters",
        string_format="PrefixBigN",
        string_escape="QuoteStripNul",
        bytes_format="PrefixZeroX",
    ),
    MySql: dict(
        placeholder_sigil=":",
        bit_format="PrefixSmallB",
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Int",
        identifier_format="Backtick",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="BackslashMysql",
        bytes_format="PrefixBigX",
    ),
    Oracle: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Int",
        identifier_format="DoubleQuote",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="QuoteStripNul",
        bytes_format="HexToRaw",
    ),
    PostgreSql: dict(
        placeholder_sigil=":",
        bit_format="PrefixBigB",
        supports_array=True,
        supports_inet=True,
        supports_mac_address=True,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="ArrayConstructor",
        bool_format="Keyword",
        identifier_format="DoubleQuote",
        identifier_escape="Delimiters",
        string_format="PrefixBigE",
        string_escape="BackslashHex",
        bytes_format="SlashXBytea",
    ),
    Redshift: dict(
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
        string_escape="QuoteStripNul",
        bytes_format="ToVarbyte",
    ),
    Snowflake: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="PatternList",
        bool_format="Keyword",
        identifier_format="DoubleQuote",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="BackslashHexZero",
        bytes_format="PrefixBigX",
    ),
    Spark: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="PatternList",
        bool_format="Keyword",
        identifier_format="Backtick",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="BackslashUnicodeQuote",
        bytes_format="PrefixBigX",
    ),
    Sqlite: dict(
        placeholder_sigil=":",
        bit_format=None,
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Int",
        identifier_format="DoubleQuote",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="QuoteStripNul",
        bytes_format="PrefixBigX",
    ),
    Teradata: dict(
        placeholder_sigil=":",
        bit_format="PostfixBigB",
        supports_array=False,
        supports_inet=False,
        supports_mac_address=False,
        null_treatment_syntax="OutsideParens",
        like_any_syntax="OrExpansion",
        bool_format="Keyword",
        identifier_format="SingleQuote",
        identifier_escape="Delimiters",
        string_format="SingleQuote",
        string_escape="QuoteStripNul",
        bytes_format="PostfixXbv",
    ),
}

_STRING_WRAP = {
    "SingleQuote": "'{}'",
    "PrefixBigE": "E'{}'",
    "PrefixBigN": "N'{}'",
}
_BOOL_TRUE = {"Keyword": "TRUE", "Int": "1"}
_BOOL_FALSE = {"Keyword": "FALSE", "Int": "0"}
_BIT_WRAP = {
    "PrefixBigB": "B'{}'",
    "PrefixSmallB": "b'{}'",
    "PostfixBigB": "'{}'B",
}
_BYTES_DEAD = {
    "PrefixBigX": "X'DEAD'",
    "PrefixZeroX": "0xDEAD",
    "SlashXBytea": "'\\xDEAD'::bytea",
    "PrefixBigBSlashX": "B'\\xDE\\xAD'",
    "HexToRaw": "HEXTORAW('DEAD')",
    "Unhex": "UNHEX('DEAD')",
    "ToVarbyte": "TO_VARBYTE('DEAD', 'hex')",
    "PostfixXbv": "'DEAD'XBV",
}


class TestSqlDialect:
    def test_base_cannot_be_instantiated(self):
        with pytest.raises(TypeError):
            SqlDialect()

    def test_rejects_a_dialect_class_where_an_instance_is_required(self):
        from rapidquery.data_type import IntegerSerializer

        with pytest.raises(TypeError, match="SqlDialect"):
            IntegerSerializer().serialize(PostgreSql, 1)

    def test_all_expected_dialects_are_exported(self):
        import rapidquery.dialect as dialect_module

        for name in ALL_DIALECT_NAMES:
            assert hasattr(dialect_module, name)
        assert hasattr(dialect_module, "GenericDialect")
        assert hasattr(dialect_module, "SqlDialect")


class TestBuiltinDialects:
    def test_is_subclass_of_sql_dialect(self, dialect_cls: type[SqlDialect]):
        assert issubclass(dialect_cls, SqlDialect)

    def test_instance_is_sql_dialect(self, dialect_instance: SqlDialect):
        assert isinstance(dialect_instance, SqlDialect)

    def test_constructs_with_no_arguments(self, dialect_cls: type[SqlDialect]):
        dialect_cls()

    def test_rejects_unexpected_arguments(self, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            dialect_cls("unexpected")

    def test_cannot_be_subclassed(self, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            type(f"Sub{dialect_cls.__name__}", (dialect_cls,), {})

    def test_frozen(self, dialect_instance: SqlDialect):
        with pytest.raises(AttributeError):
            dialect_instance.placeholder_sigil = "?"

    def test_builtin_config_covers_every_class(self):
        assert set(BUILTIN_CONFIG) == set(ALL_DIALECT_CLASSES)

    def test_config_matches_builtin_table(self, dialect_cls: type[SqlDialect]):
        dialect = dialect_cls()
        for field, expected in BUILTIN_CONFIG[dialect_cls].items():
            assert getattr(dialect, field) == expected, field

    def test_same_dialect_instances_are_equal(self, dialect_cls: type[SqlDialect]):
        assert dialect_cls() == dialect_cls()
        assert hash(dialect_cls()) == hash(dialect_cls())
        assert len({dialect_cls(), dialect_cls()}) == 1

    def test_instance_is_not_equal_to_its_class(self, dialect_cls: type[SqlDialect]):
        assert dialect_cls() != dialect_cls

    def test_distinct_builtins(self):
        assert len(set(ALL_DIALECT_CLASSES)) == len(ALL_DIALECT_CLASSES)
        assert PostgreSql() != MySql()
        assert len({PostgreSql(), MySql()}) == 2

    def test_format_boolean_follows_bool_format(self, dialect_instance: SqlDialect):
        kind = dialect_instance.bool_format
        assert dialect_instance.format_boolean(True) == _BOOL_TRUE[kind]
        assert dialect_instance.format_boolean(False) == _BOOL_FALSE[kind]

    def test_write_string_literal_wraps_plain_text(self, dialect_instance: SqlDialect):
        wrapped = _STRING_WRAP[dialect_instance.string_format].format("hello")
        assert dialect_instance.write_string_literal("hello") == wrapped

    def test_format_bytes_follows_bytes_format(self, dialect_instance: SqlDialect):
        assert (
            dialect_instance.format_bytes(b"\xde\xad")
            == _BYTES_DEAD[dialect_instance.bytes_format]
        )

    def test_format_bits(self, dialect_instance: SqlDialect):
        if dialect_instance.bit_format is None:
            with pytest.raises(ValueError, match="does not support literal bit"):
                dialect_instance.format_bits("10")
            return
        expected = _BIT_WRAP[dialect_instance.bit_format].format("10")
        assert dialect_instance.format_bits("10") == expected


class TestQuoteIdentifier:
    def test_delimiter_escaping(self):
        assert PostgreSql().quote_identifier('a"b') == '"a""b"'
        assert MySql().quote_identifier("a`b") == "`a``b`"
        assert MsSql().quote_identifier("a]b") == "[a]]b]"
        assert Teradata().quote_identifier("a'b") == "'a''b'"

    def test_same_as_string_uses_string_escape(self):
        # BigQuery: backticks + BackslashHex (same as string literals).
        assert BigQuery().quote_identifier("foo'bar\\baz") == "`foo\\'bar\\\\baz`"

    def test_generic_delimiters_and_same_as_string(self):
        doubled = generic_dialect(
            identifier_format="DoubleQuote", identifier_escape="Delimiters"
        )
        assert doubled.quote_identifier('a"b') == '"a""b"'

        same = generic_dialect(
            identifier_format="Backtick",
            identifier_escape="SameAsString",
            string_escape="BackslashHex",
        )
        assert same.quote_identifier("foo'bar") == "`foo\\'bar`"


class TestStringEscape:
    # One representative input per StringEscape variant. Full matrices are in
    # src/dialect/inner.rs.
    @pytest.mark.parametrize(
        "escape, raw, escaped",
        [
            ("Quote", "O'Brien", "O''Brien"),
            ("Quote", "a\0b", "a\0b"),
            ("QuoteStripNul", "O'Brien", "O''Brien"),
            ("QuoteStripNul", "a\0b", "ab"),
            ("BackslashHex", "'", "\\'"),
            ("BackslashHex", "\0", "\\x00"),
            ("BackslashHex", "\x1a", "\\x1A"),
            ("BackslashMysql", "\0", "\\0"),
            ("BackslashMysql", '"', '\\"'),
            ("BackslashMysql", "\x1a", "\\Z"),
            ("BackslashMysql", "\t", "\t"),
            ("BackslashUnicode", "\0", "\\0"),
            ("BackslashUnicode", "\x01", "\\u0001"),
            ("BackslashUnicode", '"', '"'),
            ("BackslashHexZero", "\0", "\\0"),
            ("BackslashHexZero", "\x1a", "\\x1A"),
            ("BackslashUnicodeQuote", '"', '\\"'),
            ("BackslashUnicodeQuote", "\x01", "\\u0001"),
        ],
    )
    def test_escape_string(self, escape: str, raw: str, escaped: str):
        dialect = generic_dialect(string_escape=escape)
        assert dialect.escape_string(raw) == escaped
        assert dialect.write_string_literal(raw) == f"'{escaped}'"


class TestGenericDialect:
    def test_is_sql_dialect(self):
        dialect = generic_dialect()
        assert isinstance(dialect, SqlDialect)
        assert isinstance(dialect, GenericDialect)
        assert issubclass(GenericDialect, SqlDialect)

    def test_cannot_be_subclassed(self):
        with pytest.raises(TypeError):
            type("SubGeneric", (GenericDialect,), {})

    def test_all_arguments_are_required_and_keyword_only(self):
        with pytest.raises(TypeError):
            GenericDialect()
        with pytest.raises(TypeError):
            GenericDialect(":")  # pyright: ignore[reportCallIssue]

    def test_unknown_format_name_is_value_error(self):
        with pytest.raises(ValueError, match="string_escape"):
            generic_dialect(string_escape="nope")
        with pytest.raises(ValueError, match="bit_format"):
            generic_dialect(bit_format="nope")

    def test_format_names_are_case_insensitive(self):
        dialect = generic_dialect(bool_format="int", string_escape="BACKSLASHHEX")
        assert dialect.bool_format == "Int"
        assert dialect.string_escape == "BackslashHex"

    def test_placeholder_sigil_must_be_one_character(self):
        with pytest.raises(ValueError, match="length 1"):
            generic_dialect(placeholder_sigil="ab")

    def test_frozen(self):
        dialect = generic_dialect()
        with pytest.raises(AttributeError):
            dialect.placeholder_sigil = "$"

    def test_equality_is_by_config(self):
        a = generic_dialect(placeholder_sigil="$")
        b = generic_dialect(placeholder_sigil="$")
        c = generic_dialect(placeholder_sigil=":")
        assert a == b
        assert hash(a) == hash(b)
        assert a != c
        assert len({a, b, c}) == 2

    def test_equal_to_builtin_with_the_same_config(self):
        # Equality compares the inner Dialect, not the Python class.
        assert generic_dialect(**BUILTIN_CONFIG[PostgreSql]) == PostgreSql()
        assert generic_dialect() != PostgreSql()

    def test_repr_lists_every_field(self):
        dialect = generic_dialect(placeholder_sigil="$", bit_format="PrefixBigB")
        text = repr(dialect)
        assert text.startswith("GenericDialect(")
        assert "placeholder_sigil='$'" in text
        assert 'bit_format="PrefixBigB"' in text
        assert "supports_array=False" in text
        assert 'string_escape="Quote"' in text

    def test_methods_honor_overrides(self):
        dialect = generic_dialect(
            bool_format="Int",
            string_format="PrefixBigE",
            string_escape="BackslashHex",
            bit_format="PrefixSmallB",
            bytes_format="PrefixZeroX",
        )
        assert dialect.format_boolean(True) == "1"
        assert dialect.write_string_literal("a'b\n") == "E'a\\'b\\n'"
        assert dialect.format_bits("10") == "b'10'"
        assert dialect.format_bytes(b"\x00") == "0x00"
