import typing

OPT_PLACEHOLDER_SIMPLE: typing.Final[int]
OPT_PLACEHOLDER_NUMBERED: typing.Final[int]
OPT_NULL_TREATMENT_UNSUPPORTED: typing.Final[int]
OPT_NULL_TREATMENT_INSIDE_PARENS: typing.Final[int]
OPT_NULL_TREATMENT_POSTFIX: typing.Final[int]
OPT_LIKE_ESCAPE_UNSUPPORTED: typing.Final[int]
OPT_LIKE_ESCAPE_SCALAR: typing.Final[int]
OPT_LIKE_ESCAPE_INCLUDING_ANY: typing.Final[int]
OPT_LIKE_ANY_OR_EXPANSION: typing.Final[int]
OPT_LIKE_ANY_ARRAY_CONSTRUCTOR: typing.Final[int]
OPT_LIKE_ANY_PATTERN_LIST: typing.Final[int]
OPT_ILIKE_EXPRESSION_STANDARD: typing.Final[int]
OPT_ILIKE_EXPRESSION_USE_LIKE_LOWER: typing.Final[int]
OPT_ILIKE_EXPRESSION_USE_LIKE: typing.Final[int]
OPT_RLIKE_EXPRESSION_UNSUPPORTED: typing.Final[int]
OPT_RLIKE_EXPRESSION_STANDARD: typing.Final[int]
OPT_RLIKE_EXPRESSION_TILDE: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_LIKE: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_CONTAINS: typing.Final[int]
OPT_RLIKE_EXPRESSION_REGEXP_MATCHES: typing.Final[int]
OPT_STRING_CONCAT_PIPES: typing.Final[int]
OPT_STRING_CONCAT_PLUS: typing.Final[int]
OPT_STRING_CONCAT_CONCAT_FUNCTION: typing.Final[int]
OPT_BITXOR_UNSUPPORTED: typing.Final[int]
OPT_BITXOR_CARET: typing.Final[int]
OPT_BITXOR_HASH: typing.Final[int]
OPT_BITXOR_XOR_FUNCTION: typing.Final[int]
OPT_BITXOR_BIT_XOR_FUNCTION: typing.Final[int]
OPT_MODULO_PERCENT: typing.Final[int]
OPT_MODULO_FUNCTION: typing.Final[int]
OPT_MODULO_INFIX: typing.Final[int]
OPT_IS_DISTINCT_FROM_UNSUPPORTED: typing.Final[int]
OPT_IS_DISTINCT_FROM_STANDARD: typing.Final[int]
OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL: typing.Final[int]
OPT_TRY_CAST_UNSUPPORTED: typing.Final[int]
OPT_TRY_CAST_TRY_CAST: typing.Final[int]
OPT_TRY_CAST_TRY_CAST_WORD: typing.Final[int]
OPT_TRY_CAST_SAFE_CAST: typing.Final[int]
OPT_TRIM_SQL_FROM: typing.Final[int]
OPT_TRIM_FUNCTION: typing.Final[int]
OPT_TRIM_SPACES_ONLY: typing.Final[int]
OPT_COLLATE_UNSUPPORTED: typing.Final[int]
OPT_COLLATE_INFIX_IDENT: typing.Final[int]
OPT_COLLATE_FUNCTION_STRING: typing.Final[int]
OPT_BITWISE_UNSUPPORTED: typing.Final[int]
OPT_BITWISE_INFIX: typing.Final[int]
OPT_BITWISE_INFIX_AND_SHIFT: typing.Final[int]
OPT_BITWISE_INFIX_NO_SHIFT: typing.Final[int]
OPT_BITWISE_BIT_SHIFT_FUNCTIONS: typing.Final[int]
OPT_BITWISE_SHIFT_FUNCTIONS: typing.Final[int]
OPT_BITWISE_BIT_AND_ONLY: typing.Final[int]
OPT_TUPLE_UNSUPPORTED: typing.Final[int]
OPT_TUPLE_ROW: typing.Final[int]
OPT_TUPLE_ROW_REJECT_EMPTY: typing.Final[int]
OPT_TUPLE_STRUCT: typing.Final[int]
OPT_TUPLE_PARENTHESES: typing.Final[int]
OPT_BIT_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_BIT_FORMAT_PREFIX_BIG_B: typing.Final[int]
OPT_BIT_FORMAT_PREFIX_SMALL_B: typing.Final[int]
OPT_BIT_FORMAT_POSTFIX_BIG_B: typing.Final[int]
OPT_BOOL_FORMAT_KEYWORD: typing.Final[int]
OPT_BOOL_FORMAT_INTEGER: typing.Final[int]
OPT_IDENTIFIER_FORMAT_DOUBLE_QUOTE: typing.Final[int]
OPT_IDENTIFIER_FORMAT_BACKTICK: typing.Final[int]
OPT_IDENTIFIER_FORMAT_SQUARE_BRACKET: typing.Final[int]
OPT_IDENTIFIER_ESCAPE_DOUBLE: typing.Final[int]
OPT_IDENTIFIER_ESCAPE_BACKSLASH: typing.Final[int]
OPT_STRING_FORMAT_SINGLE_QUOTE: typing.Final[int]
OPT_STRING_FORMAT_PREFIX_BIG_E: typing.Final[int]
OPT_STRING_FORMAT_PREFIX_BIG_N: typing.Final[int]
OPT_STRING_ESCAPE_QUOTE: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_HEX: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_MYSQL: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_UNICODE: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_HEX_ZERO: typing.Final[int]
OPT_STRING_ESCAPE_BACKSLASH_UNICODE_QUOTE: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_BIG_X: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_ZERO_X: typing.Final[int]
OPT_BYTES_FORMAT_SLASH_X_BYTEA: typing.Final[int]
OPT_BYTES_FORMAT_PREFIX_BIG_B_SLASH_X: typing.Final[int]
OPT_BYTES_FORMAT_HEX_TO_RAW: typing.Final[int]
OPT_BYTES_FORMAT_UNHEX: typing.Final[int]
OPT_BYTES_FORMAT_TO_VARBYTE: typing.Final[int]
OPT_BYTES_FORMAT_POSTFIX_XBV: typing.Final[int]
OPT_BYTES_FORMAT_TO_BINARY: typing.Final[int]
OPT_BYTES_FORMAT_SLASH_X_BLOB: typing.Final[int]
OPT_FUNCTION_FILTER_UNSUPPORTED: typing.Final[int]
OPT_FUNCTION_FILTER_FILTER: typing.Final[int]
OPT_FUNCTION_FILTER_FILTER_WHERE: typing.Final[int]
OPT_FUNCTION_FILTER_INSIDE_WHERE: typing.Final[int]
OPT_ARRAY_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_ARRAY_FORMAT_CONSTRUCTOR: typing.Final[int]
OPT_ARRAY_FORMAT_LIST: typing.Final[int]
OPT_ARRAY_FORMAT_FUNCTION: typing.Final[int]
OPT_DATE_FORMAT_QUOTED: typing.Final[int]
OPT_DATE_FORMAT_KEYWORD: typing.Final[int]
OPT_TIME_FORMAT_QUOTED: typing.Final[int]
OPT_TIME_FORMAT_KEYWORD: typing.Final[int]
OPT_TIMESTAMP_FORMAT_QUOTED: typing.Final[int]
OPT_TIMESTAMP_FORMAT_TIMESTAMPTZ: typing.Final[int]
OPT_TIMESTAMP_FORMAT_WITH_TIMEZONE: typing.Final[int]
OPT_INET_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_INET_FORMAT_QUOTED: typing.Final[int]
OPT_INET_FORMAT_CAST: typing.Final[int]
OPT_MACADDR_FORMAT_UNSUPPORTED: typing.Final[int]
OPT_MACADDR_FORMAT_QUOTED: typing.Final[int]
OPT_MACADDR_FORMAT_CAST: typing.Final[int]
OPT_UUID_FORMAT_QUOTED: typing.Final[int]
OPT_UUID_FORMAT_UUID_KEYWORD: typing.Final[int]
OPT_UUID_FORMAT_CAST_UUID: typing.Final[int]
OPT_JSON_FORMAT_QUOTED: typing.Final[int]
OPT_JSON_FORMAT_JSON_KEYWORD: typing.Final[int]
OPT_JSON_FORMAT_PARSE_JSON: typing.Final[int]

class Dialect:
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
        bitxor: int = 0,
        modulo: int = 0,
        is_distinct_from: int = 0,
        try_cast: int = 0,
        trim: int = 0,
        collate: int = 0,
        bitwise: int = 0,
        tuple: int = 0,
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
    ) -> typing.Self: ...
    @property
    def placeholder_character(self) -> str: ...
    @property
    def placeholder(self) -> int: ...
    @property
    def null_treatment(self) -> int: ...
    @property
    def like_escape(self) -> int: ...
    @property
    def like_any(self) -> int: ...
    @property
    def ilike_expression(self) -> int: ...
    @property
    def rlike_expression(self) -> int: ...
    @property
    def string_concat(self) -> int: ...
    @property
    def bitxor(self) -> int: ...
    @property
    def modulo(self) -> int: ...
    @property
    def is_distinct_from(self) -> int: ...
    @property
    def try_cast(self) -> int: ...
    @property
    def trim(self) -> int: ...
    @property
    def collate(self) -> int: ...
    @property
    def bitwise(self) -> int: ...
    @property
    def tuple(self) -> int: ...
    @property
    def bit_format(self) -> int: ...
    @property
    def bool_format(self) -> int: ...
    @property
    def identifier_format(self) -> int: ...
    @property
    def identifier_escape(self) -> int: ...
    @property
    def string_format(self) -> int: ...
    @property
    def string_escape(self) -> int: ...
    @property
    def bytes_format(self) -> int: ...
    @property
    def function_filter(self) -> int: ...
    @property
    def array_format(self) -> int: ...
    @property
    def date_format(self) -> int: ...
    @property
    def time_format(self) -> int: ...
    @property
    def timestamp_format(self) -> int: ...
    @property
    def inet_format(self) -> int: ...
    @property
    def macaddr_format(self) -> int: ...
    @property
    def uuid_format(self) -> int: ...
    @property
    def json_format(self) -> int: ...
    @property
    def supports_similar_to(self) -> bool: ...
    @property
    def parenthesize_in_between(self) -> bool: ...
    @property
    def supports_double_colon(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

BigQuery: typing.Final[Dialect] = ...
Databricks: typing.Final[Dialect] = ...
DuckDb: typing.Final[Dialect] = ...
Hive: typing.Final[Dialect] = ...
MsSql: typing.Final[Dialect] = ...
MySql: typing.Final[Dialect] = ...
Oracle: typing.Final[Dialect] = ...
PostgreSql: typing.Final[Dialect] = ...
Redshift: typing.Final[Dialect] = ...
Snowflake: typing.Final[Dialect] = ...
Spark: typing.Final[Dialect] = ...
Sqlite: typing.Final[Dialect] = ...
Teradata: typing.Final[Dialect] = ...
