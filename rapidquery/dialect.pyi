"""
SQL dialects and the integer flags that configure one.
"""

import typing

OPT_ARRAY_FORMAT_CONSTRUCTOR: typing.Final[int]
OPT_ARRAY_FORMAT_FUNCTION: typing.Final[int]
OPT_ARRAY_FORMAT_LIST: typing.Final[int]
OPT_ARRAY_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_BIT_FORMAT_POSTFIX_BIG_B: typing.Final[int]
OPT_BIT_FORMAT_PREFIX_BIG_B: typing.Final[int]
OPT_BIT_FORMAT_PREFIX_SMALL_B: typing.Final[int]
OPT_BIT_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_BITWISE_CALL_SYNTAX_BIT_FUNCTION: typing.Final[int]
OPT_BITWISE_CALL_SYNTAX_INFIX: typing.Final[int]
OPT_BITWISE_CALL_SYNTAX_SHIFT_FUNCTION: typing.Final[int]
OPT_BITWISE_OPERATIONS_AND: typing.Final[int]
OPT_BITWISE_OPERATIONS_NOT: typing.Final[int]
OPT_BITWISE_OPERATIONS_OR: typing.Final[int]
OPT_BITWISE_OPERATIONS_SHL: typing.Final[int]
OPT_BITWISE_OPERATIONS_SHR: typing.Final[int]
OPT_BITWISE_XOR_SPELLING_BIT_FUNCTION: typing.Final[int]
OPT_BITWISE_XOR_SPELLING_CARET: typing.Final[int]
OPT_BITWISE_XOR_SPELLING_FUNCTION: typing.Final[int]
OPT_BITWISE_XOR_SPELLING_HASH: typing.Final[int]
OPT_BITWISE_XOR_SPELLING_UNSUPPORTED: typing.Final[int]
OPT_BOOL_FORMAT_INTEGER: typing.Final[int]
OPT_BOOL_FORMAT_KEYWORD: typing.Final[int]
OPT_BYTES_FORMAT_HEX_TO_RAW: typing.Final[int]
OPT_BYTES_FORMAT_POSTFIX_XBV: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_BIG_B_SLASH_X: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_BIG_X: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_ZERO_X: typing.Final[int]
OPT_BYTES_FORMAT_SLASH_X_BLOB: typing.Final[int]
OPT_BYTES_FORMAT_SLASH_X_BYTEA: typing.Final[int]
OPT_BYTES_FORMAT_TO_BINARY: typing.Final[int]
OPT_BYTES_FORMAT_TO_VARBYTE: typing.Final[int]
OPT_BYTES_FORMAT_UNHEX: typing.Final[int]
OPT_CAST_TRY_CAST_SAFE_CAST: typing.Final[int]
OPT_CAST_TRY_CAST_TRY_CAST: typing.Final[int]
OPT_CAST_TRY_CAST_TRY_CAST_WORD: typing.Final[int]
OPT_CAST_TRY_CAST_UNSUPPORTED: typing.Final[int]
OPT_COLLATE_SPELLING_FUNCTION_STRING: typing.Final[int]
OPT_COLLATE_SPELLING_INFIX_IDENT: typing.Final[int]
OPT_COLLATE_SPELLING_UNSUPPORTED: typing.Final[int]
OPT_DATE_FORMAT_KEYWORD: typing.Final[int]
OPT_DATE_FORMAT_QUOTED: typing.Final[int]
OPT_FUNCTION_FILTER_FILTER: typing.Final[int]
OPT_FUNCTION_FILTER_FILTER_WHERE: typing.Final[int]
OPT_FUNCTION_FILTER_INSIDE_WHERE: typing.Final[int]
OPT_FUNCTION_FILTER_UNSUPPORTED: typing.Final[int]
OPT_IDENTIFIER_ESCAPE_BACKSLASH: typing.Final[int]
OPT_IDENTIFIER_ESCAPE_DOUBLE: typing.Final[int]
OPT_IDENTIFIER_FORMAT_BACKTICK: typing.Final[int]
OPT_IDENTIFIER_FORMAT_DOUBLE_QUOTE: typing.Final[int]
OPT_IDENTIFIER_FORMAT_SQUARE_BRACKET: typing.Final[int]
OPT_ILIKE_EXPRESSION_STANDARD: typing.Final[int]
OPT_ILIKE_EXPRESSION_USE_LIKE: typing.Final[int]
OPT_ILIKE_EXPRESSION_USE_LIKE_LOWER: typing.Final[int]
OPT_INET_FORMAT_CAST: typing.Final[int]
OPT_INET_FORMAT_QUOTED: typing.Final[int]
OPT_INET_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_IS_DISTINCT_FROM_SPELLING_NULL_SAFE_EQUAL: typing.Final[int]
OPT_IS_DISTINCT_FROM_SPELLING_STANDARD: typing.Final[int]
OPT_IS_DISTINCT_FROM_SPELLING_UNSUPPORTED: typing.Final[int]
OPT_JSON_FORMAT_JSON_KEYWORD: typing.Final[int]
OPT_JSON_FORMAT_PARSE_JSON: typing.Final[int]
OPT_JSON_FORMAT_QUOTED: typing.Final[int]
OPT_LIKE_ANY_ARRAY_CONSTRUCTOR: typing.Final[int]
OPT_LIKE_ANY_OR_EXPANSION: typing.Final[int]
OPT_LIKE_ANY_PATTERN_LIST: typing.Final[int]
OPT_LIKE_ESCAPE_ANY: typing.Final[int]
OPT_LIKE_ESCAPE_SCALAR: typing.Final[int]
OPT_MACADDR_FORMAT_CAST: typing.Final[int]
OPT_MACADDR_FORMAT_QUOTED: typing.Final[int]
OPT_MACADDR_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_MODULO_SPELLING_FUNCTION: typing.Final[int]
OPT_MODULO_SPELLING_INFIX: typing.Final[int]
OPT_MODULO_SPELLING_PERCENT: typing.Final[int]
OPT_NULL_TREATMENT_INSIDE_PARENS: typing.Final[int]
OPT_NULL_TREATMENT_POSTFIX: typing.Final[int]
OPT_NULL_TREATMENT_UNSUPPORTED: typing.Final[int]
OPT_PLACEHOLDER_SYNTAX_NUMBERED: typing.Final[int]
OPT_PLACEHOLDER_SYNTAX_SIMPLE: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_CONTAINS: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_LIKE: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_MATCHES: typing.Final[int]
OPT_RLIKE_EXPRESSION_STANDARD: typing.Final[int]
OPT_RLIKE_EXPRESSION_TILDE: typing.Final[int]
OPT_RLIKE_EXPRESSION_UNSUPPORTED: typing.Final[int]
OPT_SCHEMA_AUTHORIZATION_STYLE_OPTIONAL: typing.Final[int]
OPT_SCHEMA_AUTHORIZATION_STYLE_UNSUPPORTED: typing.Final[int]
OPT_SCHEMA_FEATURES_EMBEDDED_DDL: typing.Final[int]
OPT_SCHEMA_FEATURES_IF_NOT_EXISTS: typing.Final[int]
OPT_SCHEMA_FEATURES_IS_DATABASE: typing.Final[int]
OPT_SCHEMA_FEATURES_DROP_CASCADE: typing.Final[int]
OPT_SCHEMA_FEATURES_DROP_RESTRICT: typing.Final[int]
OPT_SCHEMA_FEATURES_UNSUPPORTED: typing.Final[int]
OPT_SCHEMA_LOCATION_STYLE_EXTERNAL_VOLUME: typing.Final[int]
OPT_SCHEMA_LOCATION_STYLE_LOCATION_KEYWORD: typing.Final[int]
OPT_SCHEMA_LOCATION_STYLE_MANAGED_LOCATION_KEYWORD: typing.Final[int]
OPT_SCHEMA_LOCATION_STYLE_UNSUPPORTED: typing.Final[int]
OPT_STRING_CONCAT_CONCAT_FUNCTION: typing.Final[int]
OPT_STRING_CONCAT_PIPES: typing.Final[int]
OPT_STRING_CONCAT_PLUS: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_HEX: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_HEX_ZERO: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_MYSQL: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_UNICODE: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_UNICODE_QUOTE: typing.Final[int]
OPT_STRING_ESCAPE_QUOTE: typing.Final[int]
OPT_STRING_FORMAT_PREFIX_BIG_E: typing.Final[int]
OPT_STRING_FORMAT_PREFIX_BIG_N: typing.Final[int]
OPT_STRING_FORMAT_SINGLE_QUOTE: typing.Final[int]
OPT_TIME_FORMAT_KEYWORD: typing.Final[int]
OPT_TIME_FORMAT_QUOTED: typing.Final[int]
OPT_TIMESTAMP_FORMAT_QUOTED: typing.Final[int]
OPT_TIMESTAMP_FORMAT_TIMESTAMPTZ: typing.Final[int]
OPT_TIMESTAMP_FORMAT_WITH_TIMEZONE: typing.Final[int]
OPT_TRIM_SPELLING_FUNCTION: typing.Final[int]
OPT_TRIM_SPELLING_SPACES_ONLY: typing.Final[int]
OPT_TRIM_SPELLING_SQL_FROM: typing.Final[int]
OPT_TUPLE_ARITY_EMPTY: typing.Final[int]
OPT_TUPLE_ARITY_ONE: typing.Final[int]
OPT_TUPLE_SYNTAX_PARENTHESES: typing.Final[int]
OPT_TUPLE_SYNTAX_ROW: typing.Final[int]
OPT_TUPLE_SYNTAX_STRUCT: typing.Final[int]
OPT_TUPLE_SYNTAX_UNSUPPORTED: typing.Final[int]
OPT_UUID_FORMAT_CAST_UUID: typing.Final[int]
OPT_UUID_FORMAT_QUOTED: typing.Final[int]
OPT_UUID_FORMAT_UUID_KEYWORD: typing.Final[int]

class Dialect:
    """Quoting, literal spelling, and which SQL constructs are legal."""

    def __new__(
        cls,
        *,
        placeholder_character: str = "?",
        placeholder_syntax: int = ...,
        like_escape: int = ...,
        like_any: int = ...,
        ilike_expression: int = ...,
        rlike_expression: int = ...,
        supports_similar_to: bool = False,
        null_treatment: int = ...,
        function_filter: int = ...,
        bitwise_operations: int = ...,
        bitwise_call_syntax: int = ...,
        bitwise_xor_spelling: int = ...,
        identifier_format: int = ...,
        identifier_escape: int = ...,
        string_format: int = ...,
        string_escape: int = ...,
        string_concat: int = ...,
        cast_try_cast: int = ...,
        cast_double_colon: bool = False,
        tuple_syntax: int = ...,
        tuple_arity: int = ...,
        date_format: int = ...,
        time_format: int = ...,
        timestamp_format: int = ...,
        bool_format: int = ...,
        bit_format: int = ...,
        bytes_format: int = ...,
        inet_format: int = ...,
        macaddr_format: int = ...,
        uuid_format: int = ...,
        json_format: int = ...,
        array_format: int = ...,
        modulo_spelling: int = ...,
        is_distinct_from_spelling: int = ...,
        trim_spelling: int = ...,
        collate_spelling: int = ...,
        schema_features: int = ...,
        schema_authorization_style: int = ...,
        schema_location_style: int = ...,
    ) -> typing.Self:
        """
        Build a dialect from flags.

        An exclusive spelling is one `OPT_*` constant. A mask is several
        `OPT_*` constants combined with `|`.
        """

    @property
    def array_format(self) -> int: ...
    @property
    def bit_format(self) -> int: ...
    @property
    def bitwise_call_syntax(self) -> int: ...
    @property
    def bitwise_operations(self) -> int: ...
    @property
    def bitwise_xor_spelling(self) -> int: ...
    @property
    def bool_format(self) -> int: ...
    @property
    def bytes_format(self) -> int: ...
    @property
    def cast_double_colon(self) -> bool: ...
    @property
    def cast_try_cast(self) -> int: ...
    @property
    def collate_spelling(self) -> int: ...
    @property
    def date_format(self) -> int: ...
    @property
    def function_filter(self) -> int: ...
    @property
    def identifier_escape(self) -> int: ...
    @property
    def identifier_format(self) -> int: ...
    @property
    def ilike_expression(self) -> int: ...
    @property
    def inet_format(self) -> int: ...
    @property
    def is_distinct_from_spelling(self) -> int: ...
    @property
    def json_format(self) -> int: ...
    @property
    def like_any(self) -> int: ...
    @property
    def like_escape(self) -> int: ...
    @property
    def macaddr_format(self) -> int: ...
    @property
    def modulo_spelling(self) -> int: ...
    @property
    def null_treatment(self) -> int: ...
    @property
    def placeholder_character(self) -> str: ...
    @property
    def placeholder_syntax(self) -> int: ...
    @property
    def rlike_expression(self) -> int: ...
    @property
    def string_concat(self) -> int: ...
    @property
    def string_escape(self) -> int: ...
    @property
    def string_format(self) -> int: ...
    @property
    def supports_similar_to(self) -> bool: ...
    @property
    def time_format(self) -> int: ...
    @property
    def timestamp_format(self) -> int: ...
    @property
    def trim_spelling(self) -> int: ...
    @property
    def tuple_arity(self) -> int: ...
    @property
    def tuple_syntax(self) -> int: ...
    @property
    def uuid_format(self) -> int: ...
    @property
    def schema_features(self) -> int: ...
    @property
    def schema_authorization_style(self) -> int: ...
    @property
    def schema_location_style(self) -> int: ...
    def write_boolean(self, value: bool) -> str:
        """
        Write `value` as this dialect's boolean literal.

        Args:
            value: Python boolean.

        Returns:
            `TRUE`, `FALSE`, `1`, or `0`.
        """

    def write_identifier(self, value: str) -> str:
        """Quote `value` with this dialect's identifier delimiters.

        Args:
            value: Raw identifier body.

        Returns:
            The delimited identifier.
        """

    def write_string_literal(self, value: str) -> str:
        """
        Escape `value` and wrap it as a complete string literal.

        Args:
            value: Text to embed. A NUL character is rejected.

        Returns:
            The SQL string literal.

        Raises:
            ValueError: `value` contains a NUL character.
        """

    def write_bits(self, value: str) -> str:
        """
        Write a bit string of `0` and `1` characters.

        Args:
            value: Bit characters. Empty is legal. Any other character is not.

        Returns:
            The dialect's bit literal.

        Raises:
            ValueError: `value` contains a character other than `0` or `1`.
            NotImplementedError: This dialect has no bit-string literal.
        """

    def write_bytes(self, value: bytes) -> str:
        """Write `value` as this dialect's blob or bytea literal.

        Args:
            value: Raw bytes.

        Returns:
            The SQL literal.

        Raises:
            NotImplementedError: This dialect has no byte literal of the
                configured form.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

BigQuery: typing.Final[Dialect] = ...
"""BigQuery standard SQL."""

Databricks: typing.Final[Dialect] = ...
"""Databricks SQL."""

DuckDb: typing.Final[Dialect] = ...
"""DuckDB SQL."""

Hive: typing.Final[Dialect] = ...
"""HiveQL."""

MsSql: typing.Final[Dialect] = ...
"""Microsoft SQL Server."""

MySql: typing.Final[Dialect] = ...
"""MySQL."""

Oracle: typing.Final[Dialect] = ...
"""Oracle SQL."""

PostgreSql: typing.Final[Dialect] = ...
"""PostgreSQL."""

Redshift: typing.Final[Dialect] = ...
"""Amazon Redshift."""

Snowflake: typing.Final[Dialect] = ...
"""Snowflake SQL."""

Spark: typing.Final[Dialect] = ...
"""Spark SQL."""

Sqlite: typing.Final[Dialect] = ...
"""SQLite."""

Teradata: typing.Final[Dialect] = ...
"""Teradata SQL."""
