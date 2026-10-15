"""
SQL dialects and the integer flags that configure one.

Use a builtin (`PostgreSql`, `Sqlite`, `MySql`, and the others) when rendering
an expression. Build a `Dialect` only when you need a different combination of
`OPT_*` flags. Each flag selects a spelling; it does not invent syntax the
dialect lacks.

Examples:
    Quote an identifier and write a boolean the way MySQL does:

    >>> from rapidquery.dialect import (
    ...     OPT_BOOL_FORMAT_INTEGER,
    ...     OPT_IDENTIFIER_FORMAT_BACKTICK,
    ...     Dialect,
    ... )
    >>> dialect = Dialect(
    ...     bool_format=OPT_BOOL_FORMAT_INTEGER,
    ...     identifier_format=OPT_IDENTIFIER_FORMAT_BACKTICK,
    ... )
    >>> dialect.write_boolean(True)
    '1'
    >>> dialect.write_identifier("id")
    '`id`'
"""

import typing

OPT_PLACEHOLDER_SIMPLE: typing.Final[int]
"""Placeholders are a bare `?`."""

OPT_PLACEHOLDER_NUMBERED: typing.Final[int]
"""Placeholders are numbered, such as `$1`."""

OPT_NULL_TREATMENT_UNSUPPORTED: typing.Final[int]
"""`IGNORE NULLS` and `RESPECT NULLS` are rejected."""

OPT_NULL_TREATMENT_INSIDE_PARENS: typing.Final[int]
"""Null treatment is written inside the call parentheses."""

OPT_NULL_TREATMENT_POSTFIX: typing.Final[int]
"""Null treatment is written after the call."""

OPT_LIKE_ESCAPE_SCALAR: typing.Final[int]
"""`ESCAPE` is legal on a scalar `LIKE` and on an `OR` expansion."""

OPT_LIKE_ESCAPE_ANY: typing.Final[int]
"""`ESCAPE` is legal on native `LIKE ANY`. Combine with `OPT_LIKE_ESCAPE_SCALAR`."""

OPT_LIKE_ANY_OR_EXPANSION: typing.Final[int]
"""`LIKE ANY` is expanded to `OR` of scalar `LIKE` predicates."""

OPT_LIKE_ANY_ARRAY_CONSTRUCTOR: typing.Final[int]
"""`LIKE ANY` uses `ANY (ARRAY[…])`."""

OPT_LIKE_ANY_PATTERN_LIST: typing.Final[int]
"""`LIKE ANY` uses a parenthesized pattern list."""

OPT_ILIKE_EXPRESSION_STANDARD: typing.Final[int]
"""Case-insensitive match is the `ILIKE` operator."""

OPT_ILIKE_EXPRESSION_USE_LIKE_LOWER: typing.Final[int]
"""Case-insensitive match lowers both sides and uses `LIKE`."""

OPT_ILIKE_EXPRESSION_USE_LIKE: typing.Final[int]
"""Case-insensitive match falls back to `LIKE`."""

OPT_RLIKE_EXPRESSION_UNSUPPORTED: typing.Final[int]
"""Regex predicates are rejected."""

OPT_RLIKE_EXPRESSION_STANDARD: typing.Final[int]
"""Regex match is the `RLIKE` operator."""

OPT_RLIKE_EXPRESSION_TILDE: typing.Final[int]
"""Regex match is `~`."""

OPT_RLIKE_EXPRESSION_REGEXP_LIKE: typing.Final[int]
"""Regex match is `REGEXP_LIKE`."""

OPT_RLIKE_EXPRESSION_REGEXP_CONTAINS: typing.Final[int]
"""Regex match is `REGEXP_CONTAINS`."""

OPT_RLIKE_EXPRESSION_REGEXP_MATCHES: typing.Final[int]
"""Regex match is `regexp_matches`."""

OPT_STRING_CONCAT_PIPES: typing.Final[int]
"""String concatenation is `||`."""

OPT_STRING_CONCAT_PLUS: typing.Final[int]
"""String concatenation is `+`."""

OPT_STRING_CONCAT_CONCAT_FUNCTION: typing.Final[int]
"""String concatenation is `CONCAT`."""

OPT_BIT_AND: typing.Final[int]
"""Bitwise AND is legal. Combine with the other `OPT_BIT_*` masks."""

OPT_BIT_OR: typing.Final[int]
"""Bitwise OR is legal."""

OPT_BIT_NOT: typing.Final[int]
"""Bitwise NOT is legal."""

OPT_BIT_SHL: typing.Final[int]
"""Left shift is legal."""

OPT_BIT_SHR: typing.Final[int]
"""Right shift is legal."""

OPT_BIT_XOR: typing.Final[int]
"""Bitwise XOR is legal. The token comes from `xor`."""

OPT_BIT_CALL_INFIX: typing.Final[int]
"""AND, OR, NOT, and shifts are `&`, `|`, `~`, `<<`, and `>>`."""

OPT_BIT_CALL_BIT_FUNCTION: typing.Final[int]
"""AND, OR, NOT, and shifts are `BITAND`, `BITOR`, `BITNOT`, `BITSHIFTLEFT`, and `BITSHIFTRIGHT`."""

OPT_BIT_CALL_SHIFT_FUNCTION: typing.Final[int]
"""AND, OR, and NOT are `BIT*` functions. Shifts are `SHIFTLEFT` and `SHIFTRIGHT`."""

OPT_XOR_CARET: typing.Final[int]
"""Bitwise XOR is `^` when `OPT_BIT_XOR` is set."""

OPT_XOR_HASH: typing.Final[int]
"""Bitwise XOR is `#` when `OPT_BIT_XOR` is set."""

OPT_XOR_FUNCTION: typing.Final[int]
"""Bitwise XOR is `xor()` when `OPT_BIT_XOR` is set."""

OPT_XOR_BIT_FUNCTION: typing.Final[int]
"""Bitwise XOR is `BITXOR()` when `OPT_BIT_XOR` is set."""

OPT_MODULO_PERCENT: typing.Final[int]
"""Modulo is `%`."""

OPT_MODULO_FUNCTION: typing.Final[int]
"""Modulo is a function call."""

OPT_MODULO_INFIX: typing.Final[int]
"""Modulo is the dialect's infix operator."""

OPT_IS_DISTINCT_FROM_UNSUPPORTED: typing.Final[int]
"""`IS DISTINCT FROM` is rejected."""

OPT_IS_DISTINCT_FROM_STANDARD: typing.Final[int]
"""`IS DISTINCT FROM` uses the SQL standard spelling."""

OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL: typing.Final[int]
"""`IS DISTINCT FROM` uses the dialect's null-safe equality operator."""

OPT_TRY_CAST_UNSUPPORTED: typing.Final[int]
"""`TRY_CAST` is rejected."""

OPT_TRY_CAST_TRY_CAST: typing.Final[int]
"""`TRY_CAST` is written `TRY_CAST`."""

OPT_TRY_CAST_TRY_CAST_WORD: typing.Final[int]
"""`TRY_CAST` is written as one word, `TRYCAST`."""

OPT_TRY_CAST_SAFE_CAST: typing.Final[int]
"""`TRY_CAST` is written `SAFE_CAST`."""

OPT_TRIM_SQL_FROM: typing.Final[int]
"""`TRIM` uses the SQL `TRIM([side] chars FROM expr)` form."""

OPT_TRIM_FUNCTION: typing.Final[int]
"""`TRIM` is a function, including `LTRIM` and `RTRIM`."""

OPT_TRIM_SPACES_ONLY: typing.Final[int]
"""`TRIM` can remove spaces only."""

OPT_COLLATE_UNSUPPORTED: typing.Final[int]
"""`COLLATE` is rejected."""

OPT_COLLATE_INFIX_IDENT: typing.Final[int]
"""`COLLATE` is an infix identifier."""

OPT_COLLATE_FUNCTION_STRING: typing.Final[int]
"""`COLLATE` is a function and the name is a string."""

OPT_TUPLE_UNSUPPORTED: typing.Final[int]
"""Tuple values are rejected."""

OPT_TUPLE_ROW: typing.Final[int]
"""Tuples are `ROW(…)`."""

OPT_TUPLE_STRUCT: typing.Final[int]
"""Tuples are `struct(…)`."""

OPT_TUPLE_PARENTHESES: typing.Final[int]
"""Tuples are `(…)`."""

OPT_TUPLE_EMPTY: typing.Final[int]
"""Arity 0 is legal. Combine with `OPT_TUPLE_ONE` when a single element is legal too."""

OPT_TUPLE_ONE: typing.Final[int]
"""Arity 1 is legal. Arity 2 and above stays legal whenever the syntax is not unsupported."""

OPT_BIT_FORMAT_UNSUPPORTED: typing.Final[int]
"""Bit-string literals are rejected."""

OPT_BIT_FORMAT_PREFIX_BIG_B: typing.Final[int]
"""Bit strings are `B'…'`."""

OPT_BIT_FORMAT_PREFIX_SMALL_B: typing.Final[int]
"""Bit strings are `b'…'`."""

OPT_BIT_FORMAT_POSTFIX_BIG_B: typing.Final[int]
"""Bit strings are `'…'B`."""

OPT_BOOL_FORMAT_KEYWORD: typing.Final[int]
"""Booleans are `TRUE` and `FALSE`."""

OPT_BOOL_FORMAT_INTEGER: typing.Final[int]
"""Booleans are `1` and `0`."""

OPT_IDENTIFIER_FORMAT_DOUBLE_QUOTE: typing.Final[int]
"""Quoted identifiers use double quotes."""

OPT_IDENTIFIER_FORMAT_BACKTICK: typing.Final[int]
"""Quoted identifiers use backticks."""

OPT_IDENTIFIER_FORMAT_SQUARE_BRACKET: typing.Final[int]
"""Quoted identifiers use square brackets."""

OPT_IDENTIFIER_ESCAPE_DOUBLE: typing.Final[int]
"""A quote inside an identifier is written twice."""

OPT_IDENTIFIER_ESCAPE_BACKSLASH: typing.Final[int]
"""A quote inside an identifier is escaped with a backslash."""

OPT_STRING_FORMAT_SINGLE_QUOTE: typing.Final[int]
"""String literals use single quotes."""

OPT_STRING_FORMAT_PREFIX_BIG_E: typing.Final[int]
"""String literals use the `E'…'` escape prefix."""

OPT_STRING_FORMAT_PREFIX_BIG_N: typing.Final[int]
"""String literals use the national `N'…'` prefix."""

OPT_STRING_ESCAPE_QUOTE: typing.Final[int]
"""A quote inside a string is written twice."""

OPT_STRING_ESCAPE_BACKSLASH_HEX: typing.Final[int]
"""String escapes use backslash, with non-ASCII bytes in hex."""

OPT_STRING_ESCAPE_BACKSLASH_MYSQL: typing.Final[int]
"""String escapes use the MySQL backslash table."""

OPT_STRING_ESCAPE_BACKSLASH_UNICODE: typing.Final[int]
"""String escapes use backslash Unicode sequences."""

OPT_STRING_ESCAPE_BACKSLASH_HEX_ZERO: typing.Final[int]
"""String escapes use backslash hex, and NUL is rejected."""

OPT_STRING_ESCAPE_BACKSLASH_UNICODE_QUOTE: typing.Final[int]
"""String escapes use backslash Unicode, and quotes are escaped too."""

OPT_BYTES_FORMAT_PREFIX_BIG_X: typing.Final[int]
"""Bytes are `X'…'`."""

OPT_BYTES_FORMAT_PREFIX_ZERO_X: typing.Final[int]
"""Bytes are `0x…`."""

OPT_BYTES_FORMAT_SLASH_X_BYTEA: typing.Final[int]
"""Bytes are `'\\x…'::bytea`."""

OPT_BYTES_FORMAT_PREFIX_BIG_B_SLASH_X: typing.Final[int]
"""Bytes are `b'\\x…'`."""

OPT_BYTES_FORMAT_HEX_TO_RAW: typing.Final[int]
"""Bytes are `HEXTORAW('…')`."""

OPT_BYTES_FORMAT_UNHEX: typing.Final[int]
"""Bytes are `UNHEX('…')`."""

OPT_BYTES_FORMAT_TO_VARBYTE: typing.Final[int]
"""Bytes are `TO_VARBYTE(…)`."""

OPT_BYTES_FORMAT_POSTFIX_XBV: typing.Final[int]
"""Bytes use the dialect's postfix binary literal."""

OPT_BYTES_FORMAT_TO_BINARY: typing.Final[int]
"""Bytes are `TO_BINARY('…')`."""

OPT_BYTES_FORMAT_SLASH_X_BLOB: typing.Final[int]
"""Bytes are `'\\x…'::BLOB`."""

OPT_FUNCTION_FILTER_UNSUPPORTED: typing.Final[int]
"""Aggregate `FILTER` is rejected."""

OPT_FUNCTION_FILTER_FILTER: typing.Final[int]
"""`FILTER` omits the `WHERE` keyword."""

OPT_FUNCTION_FILTER_FILTER_WHERE: typing.Final[int]
"""`FILTER` is `FILTER (WHERE …)`."""

OPT_FUNCTION_FILTER_INSIDE_WHERE: typing.Final[int]
"""The filter is a `WHERE` clause inside the call."""

OPT_ARRAY_FORMAT_UNSUPPORTED: typing.Final[int]
"""Array literals are rejected."""

OPT_ARRAY_FORMAT_CONSTRUCTOR: typing.Final[int]
"""Arrays are `ARRAY[…]`."""

OPT_ARRAY_FORMAT_LIST: typing.Final[int]
"""Arrays are a bracket list."""

OPT_ARRAY_FORMAT_FUNCTION: typing.Final[int]
"""Arrays are a function call."""

OPT_DATE_FORMAT_QUOTED: typing.Final[int]
"""Dates are quoted `YYYY-MM-DD` strings."""

OPT_DATE_FORMAT_KEYWORD: typing.Final[int]
"""Dates are `DATE 'YYYY-MM-DD'`."""

OPT_TIME_FORMAT_QUOTED: typing.Final[int]
"""Times are quoted strings."""

OPT_TIME_FORMAT_KEYWORD: typing.Final[int]
"""Times are `TIME '…'`."""

OPT_TIMESTAMP_FORMAT_QUOTED: typing.Final[int]
"""Timestamps are quoted strings."""

OPT_TIMESTAMP_FORMAT_TIMESTAMPTZ: typing.Final[int]
"""Aware timestamps use the `TIMESTAMPTZ` keyword."""

OPT_TIMESTAMP_FORMAT_WITH_TIMEZONE: typing.Final[int]
"""Aware timestamps use `TIMESTAMP WITH TIMEZONE`."""

OPT_INET_FORMAT_UNSUPPORTED: typing.Final[int]
"""Inet literals are rejected."""

OPT_INET_FORMAT_QUOTED: typing.Final[int]
"""Inet values are quoted strings."""

OPT_INET_FORMAT_CAST: typing.Final[int]
"""Inet values are `'…'::inet`."""

OPT_MACADDR_FORMAT_UNSUPPORTED: typing.Final[int]
"""MAC address literals are rejected."""

OPT_MACADDR_FORMAT_QUOTED: typing.Final[int]
"""MAC addresses are quoted strings."""

OPT_MACADDR_FORMAT_CAST: typing.Final[int]
"""MAC addresses are `'…'::macaddr`."""

OPT_UUID_FORMAT_QUOTED: typing.Final[int]
"""UUIDs are quoted strings."""

OPT_UUID_FORMAT_UUID_KEYWORD: typing.Final[int]
"""UUIDs are `UUID '…'`."""

OPT_UUID_FORMAT_CAST_UUID: typing.Final[int]
"""UUIDs are `'…'::uuid`."""

OPT_JSON_FORMAT_QUOTED: typing.Final[int]
"""JSON is a quoted string."""

OPT_JSON_FORMAT_JSON_KEYWORD: typing.Final[int]
"""JSON is `JSON '…'`."""

OPT_JSON_FORMAT_PARSE_JSON: typing.Final[int]
"""JSON is `PARSE_JSON('…')`."""

class Dialect:
    """Quoting, literal spelling, and which SQL constructs are legal.

    Use a builtin dialect when rendering an expression. Construct `Dialect`
    when you need a different mix of `OPT_*` flags. Flags select a spelling
    the renderer already knows. They do not add syntax that has no writer.

    Examples:
        Write MySQL-style booleans and identifiers:

        >>> from rapidquery.dialect import (
        ...     OPT_BOOL_FORMAT_INTEGER,
        ...     OPT_IDENTIFIER_FORMAT_BACKTICK,
        ...     Dialect,
        ... )
        >>> dialect = Dialect(
        ...     bool_format=OPT_BOOL_FORMAT_INTEGER,
        ...     identifier_format=OPT_IDENTIFIER_FORMAT_BACKTICK,
        ... )
        >>> dialect.write_boolean(True)
        '1'
        >>> dialect.write_identifier("id")
        '`id`'
    """

    def __new__(
        cls,
        *,
        placeholder_character: str = "?",
        placeholder: int = 0,
        null_treatment: int = 0,
        like_escape: int = 0,
        like_any: int = 0,
        ilike_expression: int = 0,
        rlike_expression: int = 0,
        string_concat: int = 0,
        bit_ops: int = 31,
        bit_call: int = 0,
        xor: int = 0,
        modulo: int = 0,
        is_distinct_from: int = 0,
        try_cast: int = 0,
        trim: int = 0,
        collate: int = 0,
        tuple: int = 0,
        tuple_arity: int = 0,
        bit_format: int = 0,
        bool_format: int = 0,
        identifier_format: int = 0,
        identifier_escape: int = 0,
        string_format: int = 0,
        string_escape: int = 0,
        bytes_format: int = 0,
        function_filter: int = 0,
        array_format: int = 0,
        date_format: int = 0,
        time_format: int = 0,
        timestamp_format: int = 0,
        inet_format: int = 0,
        macaddr_format: int = 0,
        uuid_format: int = 0,
        json_format: int = 0,
        supports_similar_to: bool = False,
        parenthesize_in_between: bool = False,
        supports_double_colon: bool = False,
    ) -> typing.Self:
        """
        Build a dialect from flags.

        An exclusive spelling is one `OPT_*` constant. A mask is several
        `OPT_*` constants combined with `|`. Omitted spellings use discriminant
        0. Omitted masks are empty, except `bit_ops`, which defaults to infix
        AND, OR, NOT, and both shifts.

        Args:
            placeholder_character: One character written for a parameter.
            placeholder: `OPT_PLACEHOLDER_SIMPLE` or `OPT_PLACEHOLDER_NUMBERED`.
            null_treatment: Where `{IGNORE|RESPECT} NULLS` is written.
            like_escape: Mask of `OPT_LIKE_ESCAPE_SCALAR` and `OPT_LIKE_ESCAPE_ANY`. `0` rejects `ESCAPE`.
            like_any: How `LIKE ANY` is spelled.
            ilike_expression: How a case-insensitive `LIKE` is spelled.
            rlike_expression: How a regex predicate is spelled.
            string_concat: How `||` is spelled.
            bit_ops: Mask of `OPT_BIT_AND`, `OPT_BIT_OR`, `OPT_BIT_NOT`, `OPT_BIT_SHL`,
                `OPT_BIT_SHR`, and `OPT_BIT_XOR`. The default `31` is every operator except XOR.
            bit_call: How AND, OR, NOT, and shifts are spelled.
            xor: How `^` is spelled when `OPT_BIT_XOR` is set. Ignored otherwise.
            modulo: How `%` is spelled.
            is_distinct_from: How `IS DISTINCT FROM` is spelled.
            try_cast: How `TRY_CAST` is spelled.
            trim: How `TRIM` is spelled.
            collate: How `COLLATE` is spelled.
            tuple: How a row value is spelled.
            tuple_arity: Mask of `OPT_TUPLE_EMPTY` and `OPT_TUPLE_ONE`. `0` means at least two elements.
            bit_format: How a bit-string literal is spelled.
            bool_format: `TRUE`/`FALSE` or `1`/`0`.
            identifier_format: Delimiter around a quoted identifier.
            identifier_escape: How an identifier quotes its delimiter.
            string_format: Prefix and quotes of a string literal.
            string_escape: How special characters in a string are escaped.
            bytes_format: How a byte literal is spelled.
            function_filter: How aggregate `FILTER` is spelled.
            array_format: How an array literal is spelled.
            date_format: How a date literal is spelled.
            time_format: How a time literal is spelled.
            timestamp_format: How a timestamp literal is spelled.
            inet_format: How an inet literal is spelled.
            macaddr_format: How a MAC address literal is spelled.
            uuid_format: How a UUID literal is spelled.
            json_format: How a JSON literal is spelled.
            supports_similar_to: Whether `SIMILAR TO` is legal.
            parenthesize_in_between: Wrap `IN` and `BETWEEN` in parentheses.
            supports_double_colon: Whether `expr::type` is legal.

        Raises:
            TypeError: `placeholder_character` is not one character, or a flag
                is not an `int`.
            ValueError: A mask has a bit that is not one of its `OPT_*` flags.
        """

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

    @property
    def placeholder_character(self) -> str:
        """The one character used in placeholders."""

    @property
    def placeholder(self) -> int:
        """`OPT_PLACEHOLDER_SIMPLE` or `OPT_PLACEHOLDER_NUMBERED`."""

    @property
    def null_treatment(self) -> int:
        """Where `{IGNORE|RESPECT} NULLS` is written."""

    @property
    def like_escape(self) -> int:
        """Mask of where `LIKE … ESCAPE` is legal. `0` rejects it."""

    @property
    def like_any(self) -> int:
        """How `LIKE ANY` is spelled."""

    @property
    def ilike_expression(self) -> int:
        """How a case-insensitive match is spelled."""

    @property
    def rlike_expression(self) -> int:
        """How a regex predicate is spelled."""

    @property
    def string_concat(self) -> int:
        """How string concatenation is spelled."""

    @property
    def bit_ops(self) -> int:
        """Mask of bitwise operators this dialect can render."""

    @property
    def bit_call(self) -> int:
        """How AND, OR, NOT, and shifts are spelled."""

    @property
    def xor(self) -> int:
        """How bitwise XOR is spelled when `OPT_BIT_XOR` is set."""

    @property
    def modulo(self) -> int:
        """How modulo is spelled."""

    @property
    def is_distinct_from(self) -> int:
        """How `IS DISTINCT FROM` is spelled."""

    @property
    def try_cast(self) -> int:
        """How `TRY_CAST` is spelled."""

    @property
    def trim(self) -> int:
        """How `TRIM` is spelled."""

    @property
    def collate(self) -> int:
        """How `COLLATE` is spelled."""

    @property
    def tuple(self) -> int:
        """How a row value is spelled."""

    @property
    def tuple_arity(self) -> int:
        """Which tuple arities below 2 are legal."""

    @property
    def bit_format(self) -> int:
        """How a bit-string literal is spelled."""

    @property
    def bool_format(self) -> int:
        """How a boolean literal is spelled."""

    @property
    def identifier_format(self) -> int:
        """Which delimiter wraps a quoted identifier."""

    @property
    def identifier_escape(self) -> int:
        """How an identifier quotes its own delimiter."""

    @property
    def string_format(self) -> int:
        """Which prefix and quotes a string literal uses."""

    @property
    def string_escape(self) -> int:
        """How special characters in a string are escaped."""

    @property
    def bytes_format(self) -> int:
        """How a byte literal is spelled."""

    @property
    def function_filter(self) -> int:
        """How aggregate `FILTER` is spelled."""

    @property
    def array_format(self) -> int:
        """How an array literal is spelled."""

    @property
    def date_format(self) -> int:
        """How a date literal is spelled."""

    @property
    def time_format(self) -> int:
        """How a time literal is spelled."""

    @property
    def timestamp_format(self) -> int:
        """How a timestamp literal is spelled."""

    @property
    def inet_format(self) -> int:
        """How an inet literal is spelled."""

    @property
    def macaddr_format(self) -> int:
        """How a MAC address literal is spelled."""

    @property
    def uuid_format(self) -> int:
        """How a UUID literal is spelled."""

    @property
    def json_format(self) -> int:
        """How a JSON literal is spelled."""

    @property
    def supports_similar_to(self) -> bool:
        """Whether `SIMILAR TO` can be rendered."""

    @property
    def parenthesize_in_between(self) -> bool:
        """Whether `IN` and `BETWEEN` are wrapped in parentheses."""

    @property
    def supports_double_colon(self) -> bool:
        """Whether `expr::type` can be rendered."""

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
