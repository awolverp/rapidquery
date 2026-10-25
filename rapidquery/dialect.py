"""SQL dialects and the flags that configure one.

`Dialect` chooses quoting, literal spelling, and which syntax is legal. The
names at the bottom are built-in configurations.

A template key selects an escape (`{hex}`, `{double}`, `{slashx}`, ...). A
positional `{}` is raw text for dates, times, timestamps, UUIDs, inet, and
MAC addresses, so the quotes belong in the template. JSON and schema location
receive an already quoted string.

`placeholder` may be a bare marker (`'?'`) or include the 1-based index
(`'?{}'`, `'${}'`, `':{}'`). `ILIKE` has no case-sensitive `LIKE` fallback:
MsSql, MySql, and Sqlite reject it.
"""

from ._core.dialect import (
    OPT_ANY_ARRAY_CONSTRUCTOR,
    OPT_ANY_OR_EXPANSION,
    OPT_ANY_PATTERN_LIST,
    OPT_ARITY_EMPTY,
    OPT_ARITY_ONE,
    OPT_AUTHORIZATION_OPTIONAL,
    OPT_AUTHORIZATION_UNSUPPORTED,
    OPT_BOOLEAN_INTEGER,
    OPT_BOOLEAN_KEYWORD,
    OPT_COLLATE_FUNCTION_STRING,
    OPT_COLLATE_INFIX_IDENT,
    OPT_COLLATE_UNSUPPORTED,
    OPT_ESCAPE_ANY,
    OPT_ESCAPE_SCALAR,
    OPT_FEATURES_DROP_CASCADE,
    OPT_FEATURES_DROP_RESTRICT,
    OPT_FEATURES_EMBEDDED_DDL,
    OPT_FEATURES_IF_NOT_EXISTS,
    OPT_FEATURES_IS_DATABASE,
    OPT_FEATURES_SUPPORTED,
    OPT_FUNCTION_FILTER_FILTER,
    OPT_FUNCTION_FILTER_FILTER_WHERE,
    OPT_FUNCTION_FILTER_INSIDE_WHERE,
    OPT_FUNCTION_FILTER_UNSUPPORTED,
    OPT_ILIKE_STANDARD,
    OPT_ILIKE_UNSUPPORTED,
    OPT_ILIKE_USE_LIKE_LOWER,
    OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
    OPT_IS_DISTINCT_FROM_STANDARD,
    OPT_IS_DISTINCT_FROM_UNSUPPORTED,
    OPT_MODULO_FUNCTION,
    OPT_MODULO_INFIX,
    OPT_MODULO_PERCENT,
    OPT_NULL_TREATMENT_INSIDE_PARENS,
    OPT_NULL_TREATMENT_POSTFIX,
    OPT_NULL_TREATMENT_UNSUPPORTED,
    OPT_OPERATIONS_AND,
    OPT_OPERATIONS_NOT,
    OPT_OPERATIONS_OR,
    OPT_OPERATIONS_SHL,
    OPT_OPERATIONS_SHR,
    OPT_REGEX_REGEXP_CONTAINS,
    OPT_REGEX_REGEXP_LIKE,
    OPT_REGEX_REGEXP_MATCHES,
    OPT_REGEX_STANDARD,
    OPT_REGEX_TILDE,
    OPT_REGEX_UNSUPPORTED,
    OPT_SYNTAX_BIT_FUNCTION,
    OPT_SYNTAX_INFIX,
    OPT_SYNTAX_PARENTHESES,
    OPT_SYNTAX_ROW,
    OPT_SYNTAX_SHIFT_FUNCTION,
    OPT_SYNTAX_STRUCT,
    OPT_SYNTAX_UNSUPPORTED,
    OPT_TRIM_FUNCTION,
    OPT_TRIM_SPACES_ONLY,
    OPT_TRIM_SQL_FROM,
    OPT_XOR_BIT_FUNCTION,
    OPT_XOR_CARET,
    OPT_XOR_FUNCTION,
    OPT_XOR_HASH,
    OPT_XOR_UNSUPPORTED,
    AggregateConfig,
    BitwiseConfig,
    CastConfig,
    Dialect,
    LikeConfig,
    LiteralConfig,
    OperatorConfig,
    SchemaConfig,
    StringConfig,
    TemporalConfig,
    TupleConfig,
)

_ALL_BITWISE = OPT_OPERATIONS_AND | OPT_OPERATIONS_OR | OPT_OPERATIONS_NOT | OPT_OPERATIONS_SHL | OPT_OPERATIONS_SHR

__all__ = [
    "OPT_ANY_ARRAY_CONSTRUCTOR",
    "OPT_ANY_OR_EXPANSION",
    "OPT_ANY_PATTERN_LIST",
    "OPT_ARITY_EMPTY",
    "OPT_ARITY_ONE",
    "OPT_AUTHORIZATION_OPTIONAL",
    "OPT_AUTHORIZATION_UNSUPPORTED",
    "OPT_BOOLEAN_INTEGER",
    "OPT_BOOLEAN_KEYWORD",
    "OPT_COLLATE_FUNCTION_STRING",
    "OPT_COLLATE_INFIX_IDENT",
    "OPT_COLLATE_UNSUPPORTED",
    "OPT_ESCAPE_ANY",
    "OPT_ESCAPE_SCALAR",
    "OPT_FEATURES_DROP_CASCADE",
    "OPT_FEATURES_DROP_RESTRICT",
    "OPT_FEATURES_EMBEDDED_DDL",
    "OPT_FEATURES_IF_NOT_EXISTS",
    "OPT_FEATURES_IS_DATABASE",
    "OPT_FEATURES_SUPPORTED",
    "OPT_FUNCTION_FILTER_FILTER",
    "OPT_FUNCTION_FILTER_FILTER_WHERE",
    "OPT_FUNCTION_FILTER_INSIDE_WHERE",
    "OPT_FUNCTION_FILTER_UNSUPPORTED",
    "OPT_ILIKE_STANDARD",
    "OPT_ILIKE_UNSUPPORTED",
    "OPT_ILIKE_USE_LIKE_LOWER",
    "OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL",
    "OPT_IS_DISTINCT_FROM_STANDARD",
    "OPT_IS_DISTINCT_FROM_UNSUPPORTED",
    "OPT_MODULO_FUNCTION",
    "OPT_MODULO_INFIX",
    "OPT_MODULO_PERCENT",
    "OPT_NULL_TREATMENT_INSIDE_PARENS",
    "OPT_NULL_TREATMENT_POSTFIX",
    "OPT_NULL_TREATMENT_UNSUPPORTED",
    "OPT_OPERATIONS_AND",
    "OPT_OPERATIONS_NOT",
    "OPT_OPERATIONS_OR",
    "OPT_OPERATIONS_SHL",
    "OPT_OPERATIONS_SHR",
    "OPT_REGEX_REGEXP_CONTAINS",
    "OPT_REGEX_REGEXP_LIKE",
    "OPT_REGEX_REGEXP_MATCHES",
    "OPT_REGEX_STANDARD",
    "OPT_REGEX_TILDE",
    "OPT_REGEX_UNSUPPORTED",
    "OPT_SYNTAX_BIT_FUNCTION",
    "OPT_SYNTAX_INFIX",
    "OPT_SYNTAX_PARENTHESES",
    "OPT_SYNTAX_ROW",
    "OPT_SYNTAX_SHIFT_FUNCTION",
    "OPT_SYNTAX_STRUCT",
    "OPT_SYNTAX_UNSUPPORTED",
    "OPT_TRIM_FUNCTION",
    "OPT_TRIM_SPACES_ONLY",
    "OPT_TRIM_SQL_FROM",
    "OPT_XOR_BIT_FUNCTION",
    "OPT_XOR_CARET",
    "OPT_XOR_FUNCTION",
    "OPT_XOR_HASH",
    "OPT_XOR_UNSUPPORTED",
    "AggregateConfig",
    "BigQuery",
    "BitwiseConfig",
    "CastConfig",
    "Databricks",
    "Dialect",
    "DuckDb",
    "Hive",
    "LikeConfig",
    "LiteralConfig",
    "MsSql",
    "MySql",
    "OperatorConfig",
    "Oracle",
    "PostgreSql",
    "Redshift",
    "SchemaConfig",
    "Snowflake",
    "Spark",
    "Sqlite",
    "StringConfig",
    "TemporalConfig",
    "Teradata",
    "TupleConfig",
]

BigQuery = Dialect(
    placeholder="?",
    like=LikeConfig(
        ilike=OPT_ILIKE_USE_LIKE_LOWER,
        regex=OPT_REGEX_REGEXP_CONTAINS,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_CARET),
    identifier="`{slashbacktick}`",
    string=StringConfig(template="'{hex}'"),
    cast=CastConfig(try_cast="SAFE_CAST({expr} AS {type})"),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="TIME '{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(
        bytes="B'{slashx}'",
        uuid="'{}'",
        json="JSON {}",
        array="ARRAY[{}]",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        trim=OPT_TRIM_FUNCTION,
        collate=OPT_COLLATE_FUNCTION_STRING,
    ),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_INSIDE_PARENS,
        function_filter=OPT_FUNCTION_FILTER_INSIDE_WHERE,
    ),
    schema=SchemaConfig(features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS),
)

Databricks = Dialect(
    placeholder="?",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        any=OPT_ANY_PATTERN_LIST,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode_quote}'"),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_SYNTAX_STRUCT, arity=OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(uuid="'{}'"),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_POSTFIX,
        function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE,
    ),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS,
        location=" LOCATION {}",
    ),
)

DuckDb = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_REGEXP_MATCHES,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_FUNCTION),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_SYNTAX_ROW, arity=OPT_ARITY_EMPTY | OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="TIME '{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMPTZ '{}'",
    ),
    literal=LiteralConfig(
        bit="B'{}'",
        bytes="'{slashx}'::BLOB",
        inet="'{}'::inet",
        uuid="UUID '{}'",
        array="[{}]",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        trim=OPT_TRIM_FUNCTION,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_INSIDE_PARENS,
        function_filter=OPT_FUNCTION_FILTER_FILTER,
    ),
    schema=SchemaConfig(features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS),
)

Hive = Dialect(
    placeholder="?",
    like=LikeConfig(
        ilike=OPT_ILIKE_USE_LIKE_LOWER,
        regex=OPT_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode}'"),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_SYNTAX_STRUCT, arity=OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(
        bytes="UNHEX('{hex}')",
        uuid="'{}'",
        array="array({})",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
        trim=OPT_TRIM_SPACES_ONLY,
    ),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS | OPT_FEATURES_IS_DATABASE,
        location=" LOCATION {}",
    ),
)

MsSql = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_ESCAPE_SCALAR),
    bitwise=BitwiseConfig(
        operations=OPT_OPERATIONS_AND | OPT_OPERATIONS_OR | OPT_OPERATIONS_NOT,
        xor=OPT_XOR_CARET,
    ),
    identifier="[{bracket}]",
    string=StringConfig(template="N'{single}'", concat="{left} + {right}"),
    temporal=TemporalConfig(
        date="N'{}'",
        time="N'{}'",
        datetime_naive="N'{}'",
        datetime_aware="N'{}'",
    ),
    literal=LiteralConfig(
        boolean=OPT_BOOLEAN_INTEGER,
        bytes="0x{hex}",
        uuid="N'{}'",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_EMBEDDED_DDL,
        authorization=OPT_AUTHORIZATION_OPTIONAL,
    ),
)

MySql = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_ESCAPE_SCALAR, regex=OPT_REGEX_STANDARD),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{mysql}'", concat="CONCAT({left}, {right})"),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_SYNTAX_ROW, arity=OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(
        bit="b'{}'",
        boolean=OPT_BOOLEAN_INTEGER,
        uuid="'{}'",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS | OPT_FEATURES_IS_DATABASE,
    ),
)

Oracle = Dialect(
    placeholder=":{}",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        ilike=OPT_ILIKE_USE_LIKE_LOWER,
        regex=OPT_REGEX_REGEXP_LIKE,
    ),
    bitwise=BitwiseConfig(operations=OPT_OPERATIONS_AND, syntax=OPT_SYNTAX_BIT_FUNCTION),
    cast=CastConfig(try_cast=None),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(
        boolean=OPT_BOOLEAN_INTEGER,
        bytes="HEXTORAW('{hex}')",
        uuid="'{}'",
    ),
    operator=OperatorConfig(modulo=OPT_MODULO_FUNCTION, collate=OPT_COLLATE_INFIX_IDENT),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_POSTFIX,
        function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE,
    ),
    schema=SchemaConfig(features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_EMBEDDED_DDL),
)

PostgreSql = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        any=OPT_ANY_ARRAY_CONSTRUCTOR,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_TILDE,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_HASH),
    string=StringConfig(template="E'{hex}'"),
    cast=CastConfig(try_cast=None, supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_SYNTAX_ROW, arity=OPT_ARITY_EMPTY | OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE E'{}'",
        time="TIME E'{}'",
        datetime_naive="TIMESTAMP E'{}'",
        datetime_aware="TIMESTAMPTZ E'{}'",
    ),
    literal=LiteralConfig(
        bit="B'{}'",
        bytes=r"'\x{hex}'::bytea",
        inet="E'{}'::inet",
        macaddr="E'{}'::macaddr",
        uuid="E'{}'::uuid",
        array="ARRAY[{}]",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS | OPT_FEATURES_EMBEDDED_DDL,
        authorization=OPT_AUTHORIZATION_OPTIONAL,
    ),
)

Redshift = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_TILDE,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(operations=OPT_OPERATIONS_AND | OPT_OPERATIONS_SHL | OPT_OPERATIONS_SHR),
    cast=CastConfig(supports_double_colon=True),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(bytes="TO_VARBYTE('{hex}', 'hex')", uuid="'{}'"),
    operator=OperatorConfig(collate=OPT_COLLATE_FUNCTION_STRING),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS | OPT_FEATURES_EMBEDDED_DDL,
        authorization=OPT_AUTHORIZATION_OPTIONAL,
    ),
)

Snowflake = Dialect(
    placeholder=":{}",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR | OPT_ESCAPE_ANY,
        any=OPT_ANY_PATTERN_LIST,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(
        operations=_ALL_BITWISE,
        syntax=OPT_SYNTAX_BIT_FUNCTION,
        xor=OPT_XOR_BIT_FUNCTION,
    ),
    string=StringConfig(template="'{hex}'"),
    cast=CastConfig(supports_double_colon=True),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMPTZ '{}'",
    ),
    literal=LiteralConfig(
        bytes="TO_BINARY('{hex}', 'HEX')",
        uuid="'{}'",
        json="PARSE_JSON({})",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        trim=OPT_TRIM_FUNCTION,
        collate=OPT_COLLATE_FUNCTION_STRING,
    ),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS,
        location=" EXTERNAL_VOLUME = {}",
    ),
)

Spark = Dialect(
    placeholder="?",
    like=LikeConfig(
        escape=OPT_ESCAPE_SCALAR,
        any=OPT_ANY_PATTERN_LIST,
        ilike=OPT_ILIKE_STANDARD,
        regex=OPT_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode_quote}'"),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_SYNTAX_STRUCT, arity=OPT_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(uuid="'{}'", array="array({})"),
    operator=OperatorConfig(is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_POSTFIX,
        function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE,
    ),
    schema=SchemaConfig(
        features=OPT_FEATURES_SUPPORTED | OPT_FEATURES_IF_NOT_EXISTS | OPT_FEATURES_IS_DATABASE,
        location=" LOCATION {}",
    ),
)

Sqlite = Dialect(
    like=LikeConfig(escape=OPT_ESCAPE_SCALAR),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_SYNTAX_PARENTHESES, arity=OPT_ARITY_EMPTY),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(boolean=OPT_BOOLEAN_INTEGER, uuid="'{}'"),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        trim=OPT_TRIM_FUNCTION,
        collate=OPT_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE),
)

Teradata = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_ESCAPE_SCALAR, ilike=OPT_ILIKE_USE_LIKE_LOWER),
    bitwise=BitwiseConfig(
        operations=_ALL_BITWISE,
        syntax=OPT_SYNTAX_SHIFT_FUNCTION,
        xor=OPT_XOR_BIT_FUNCTION,
    ),
    cast=CastConfig(try_cast="TRYCAST({expr} AS {type})"),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(bit="'{}'B", bytes="'{hex}'XBV", uuid="'{}'"),
    operator=OperatorConfig(modulo=OPT_MODULO_INFIX),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_INSIDE_PARENS),
)
