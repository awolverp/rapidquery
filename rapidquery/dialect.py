from ._core.dialect import (
    OPT_BITWISE_OPERATIONS_AND,
    OPT_BITWISE_OPERATIONS_NOT,
    OPT_BITWISE_OPERATIONS_OR,
    OPT_BITWISE_OPERATIONS_SHL,
    OPT_BITWISE_OPERATIONS_SHR,
    OPT_BITWISE_SYNTAX_BIT_FUNCTION,
    OPT_BITWISE_SYNTAX_INFIX,
    OPT_BITWISE_SYNTAX_SHIFT_FUNCTION,
    OPT_BITWISE_XOR_BIT_FUNCTION,
    OPT_BITWISE_XOR_CARET,
    OPT_BITWISE_XOR_FUNCTION,
    OPT_BITWISE_XOR_HASH,
    OPT_BITWISE_XOR_UNSUPPORTED,
    OPT_BOOL_FORMAT_INTEGER,
    OPT_BOOL_FORMAT_KEYWORD,
    OPT_FUNCTION_FILTER_FILTER,
    OPT_FUNCTION_FILTER_FILTER_WHERE,
    OPT_FUNCTION_FILTER_INSIDE_WHERE,
    OPT_FUNCTION_FILTER_UNSUPPORTED,
    OPT_I_LIKE_STRATEGY_STANDARD,
    OPT_I_LIKE_STRATEGY_UNSUPPORTED,
    OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER,
    OPT_INDEX_DROP_FEATURES_CASCADE,
    OPT_INDEX_DROP_FEATURES_RESTRICT,
    OPT_INDEX_DROP_FEATURES_SUPPORTED,
    OPT_INDEX_DROP_TABLE_NAME_ONLY,
    OPT_INDEX_DROP_TABLE_ON,
    OPT_INDEX_DROP_TABLE_QUALIFIED,
    OPT_INDEX_FEATURES_COLUMN_ORDER,
    OPT_INDEX_FEATURES_EXPRESSIONS,
    OPT_INDEX_FEATURES_IF_NOT_EXISTS,
    OPT_INDEX_FEATURES_INCLUDE,
    OPT_INDEX_FEATURES_PARTIAL,
    OPT_INDEX_FEATURES_SUPPORTED,
    OPT_INDEX_FEATURES_UNIQUE,
    OPT_INDEX_TYPE_AFTER_COLUMNS,
    OPT_INDEX_TYPE_BEFORE_COLUMNS,
    OPT_INDEX_TYPE_UNSUPPORTED,
    OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
    OPT_IS_DISTINCT_FROM_STANDARD,
    OPT_IS_DISTINCT_FROM_UNSUPPORTED,
    OPT_LIKE_ANY_ARRAY_CONSTRUCTOR,
    OPT_LIKE_ANY_OR_EXPANSION,
    OPT_LIKE_ANY_PATTERN_LIST,
    OPT_LIKE_ESCAPE_ANY,
    OPT_LIKE_ESCAPE_SCALAR,
    OPT_LIKE_REGEX_REGEXP_CONTAINS,
    OPT_LIKE_REGEX_REGEXP_LIKE,
    OPT_LIKE_REGEX_REGEXP_MATCHES,
    OPT_LIKE_REGEX_STANDARD,
    OPT_LIKE_REGEX_TILDE,
    OPT_LIKE_REGEX_UNSUPPORTED,
    OPT_NULL_TREATMENT_INSIDE_PARENS,
    OPT_NULL_TREATMENT_POSTFIX,
    OPT_NULL_TREATMENT_UNSUPPORTED,
    OPT_OPERATOR_COLLATE_FUNCTION_STRING,
    OPT_OPERATOR_COLLATE_INFIX_IDENT,
    OPT_OPERATOR_COLLATE_UNSUPPORTED,
    OPT_OPERATOR_MODULO_FUNCTION,
    OPT_OPERATOR_MODULO_INFIX,
    OPT_OPERATOR_MODULO_PERCENT,
    OPT_OPERATOR_TRIM_FUNCTION,
    OPT_OPERATOR_TRIM_SPACES_ONLY,
    OPT_OPERATOR_TRIM_SQL_FROM,
    OPT_SCHEMA_AUTHORIZATION_OPTIONAL,
    OPT_SCHEMA_AUTHORIZATION_UNSUPPORTED,
    OPT_SCHEMA_FEATURES_DROP_CASCADE,
    OPT_SCHEMA_FEATURES_DROP_RESTRICT,
    OPT_SCHEMA_FEATURES_EMBEDDED_DDL,
    OPT_SCHEMA_FEATURES_IF_NOT_EXISTS,
    OPT_SCHEMA_FEATURES_IS_DATABASE,
    OPT_SCHEMA_FEATURES_SUPPORTED,
    OPT_TABLE_FEATURES_CHECK,
    OPT_TABLE_FEATURES_EXTERNAL,
    OPT_TABLE_FEATURES_FOREIGN_KEY,
    OPT_TABLE_FEATURES_GENERATED_SUPPORTED,
    OPT_TABLE_FEATURES_IF_NOT_EXISTS,
    OPT_TABLE_FEATURES_ON_COMMIT,
    OPT_TABLE_FEATURES_PRIMARY_KEY,
    OPT_TABLE_FEATURES_REFERENTIAL_ACTION,
    OPT_TABLE_FEATURES_UNIQUE,
    OPT_TUPLE_ARITY_EMPTY,
    OPT_TUPLE_ARITY_ONE,
    OPT_TUPLE_SYNTAX_PARENTHESES,
    OPT_TUPLE_SYNTAX_ROW,
    OPT_TUPLE_SYNTAX_STRUCT,
    OPT_TUPLE_SYNTAX_UNSUPPORTED,
    AggregateConfig,
    BitwiseConfig,
    CastConfig,
    Dialect,
    IndexConfig,
    LikeConfig,
    LiteralConfig,
    OperatorConfig,
    SchemaConfig,
    StringConfig,
    TableConfig,
    TemporalConfig,
    TupleConfig,
)

_ALL_BITWISE = (
    OPT_BITWISE_OPERATIONS_AND
    | OPT_BITWISE_OPERATIONS_OR
    | OPT_BITWISE_OPERATIONS_NOT
    | OPT_BITWISE_OPERATIONS_SHL
    | OPT_BITWISE_OPERATIONS_SHR
)

# Table-level PRIMARY KEY, UNIQUE, FOREIGN KEY, and CHECK, plus ON DELETE / ON UPDATE.
_TABLE_CONSTRAINTS = (
    OPT_TABLE_FEATURES_PRIMARY_KEY
    | OPT_TABLE_FEATURES_UNIQUE
    | OPT_TABLE_FEATURES_FOREIGN_KEY
    | OPT_TABLE_FEATURES_CHECK
    | OPT_TABLE_FEATURES_REFERENTIAL_ACTION
)

# The four standard table constraints, without referential actions.
_TABLE_CONSTRAINTS_NO_ACTION = (
    OPT_TABLE_FEATURES_PRIMARY_KEY
    | OPT_TABLE_FEATURES_UNIQUE
    | OPT_TABLE_FEATURES_FOREIGN_KEY
    | OPT_TABLE_FEATURES_CHECK
)

# PRIMARY KEY, UNIQUE, and FOREIGN KEY. CHECK is omitted.
_TABLE_KEYS = OPT_TABLE_FEATURES_PRIMARY_KEY | OPT_TABLE_FEATURES_UNIQUE | OPT_TABLE_FEATURES_FOREIGN_KEY

__all__ = [
    "OPT_BITWISE_OPERATIONS_AND",
    "OPT_BITWISE_OPERATIONS_NOT",
    "OPT_BITWISE_OPERATIONS_OR",
    "OPT_BITWISE_OPERATIONS_SHL",
    "OPT_BITWISE_OPERATIONS_SHR",
    "OPT_BITWISE_SYNTAX_BIT_FUNCTION",
    "OPT_BITWISE_SYNTAX_INFIX",
    "OPT_BITWISE_SYNTAX_SHIFT_FUNCTION",
    "OPT_BITWISE_XOR_BIT_FUNCTION",
    "OPT_BITWISE_XOR_CARET",
    "OPT_BITWISE_XOR_FUNCTION",
    "OPT_BITWISE_XOR_HASH",
    "OPT_BITWISE_XOR_UNSUPPORTED",
    "OPT_BOOL_FORMAT_INTEGER",
    "OPT_BOOL_FORMAT_KEYWORD",
    "OPT_FUNCTION_FILTER_FILTER",
    "OPT_FUNCTION_FILTER_FILTER_WHERE",
    "OPT_FUNCTION_FILTER_INSIDE_WHERE",
    "OPT_FUNCTION_FILTER_UNSUPPORTED",
    "OPT_INDEX_DROP_FEATURES_CASCADE",
    "OPT_INDEX_DROP_FEATURES_RESTRICT",
    "OPT_INDEX_DROP_FEATURES_SUPPORTED",
    "OPT_INDEX_DROP_TABLE_NAME_ONLY",
    "OPT_INDEX_DROP_TABLE_ON",
    "OPT_INDEX_DROP_TABLE_QUALIFIED",
    "OPT_INDEX_FEATURES_COLUMN_ORDER",
    "OPT_INDEX_FEATURES_EXPRESSIONS",
    "OPT_INDEX_FEATURES_IF_NOT_EXISTS",
    "OPT_INDEX_FEATURES_INCLUDE",
    "OPT_INDEX_FEATURES_PARTIAL",
    "OPT_INDEX_FEATURES_SUPPORTED",
    "OPT_INDEX_FEATURES_UNIQUE",
    "OPT_INDEX_TYPE_AFTER_COLUMNS",
    "OPT_INDEX_TYPE_BEFORE_COLUMNS",
    "OPT_INDEX_TYPE_UNSUPPORTED",
    "OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL",
    "OPT_IS_DISTINCT_FROM_STANDARD",
    "OPT_IS_DISTINCT_FROM_UNSUPPORTED",
    "OPT_I_LIKE_STRATEGY_STANDARD",
    "OPT_I_LIKE_STRATEGY_UNSUPPORTED",
    "OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER",
    "OPT_LIKE_ANY_ARRAY_CONSTRUCTOR",
    "OPT_LIKE_ANY_OR_EXPANSION",
    "OPT_LIKE_ANY_PATTERN_LIST",
    "OPT_LIKE_ESCAPE_ANY",
    "OPT_LIKE_ESCAPE_SCALAR",
    "OPT_LIKE_REGEX_REGEXP_CONTAINS",
    "OPT_LIKE_REGEX_REGEXP_LIKE",
    "OPT_LIKE_REGEX_REGEXP_MATCHES",
    "OPT_LIKE_REGEX_STANDARD",
    "OPT_LIKE_REGEX_TILDE",
    "OPT_LIKE_REGEX_UNSUPPORTED",
    "OPT_NULL_TREATMENT_INSIDE_PARENS",
    "OPT_NULL_TREATMENT_POSTFIX",
    "OPT_NULL_TREATMENT_UNSUPPORTED",
    "OPT_OPERATOR_COLLATE_FUNCTION_STRING",
    "OPT_OPERATOR_COLLATE_INFIX_IDENT",
    "OPT_OPERATOR_COLLATE_UNSUPPORTED",
    "OPT_OPERATOR_MODULO_FUNCTION",
    "OPT_OPERATOR_MODULO_INFIX",
    "OPT_OPERATOR_MODULO_PERCENT",
    "OPT_OPERATOR_TRIM_FUNCTION",
    "OPT_OPERATOR_TRIM_SPACES_ONLY",
    "OPT_OPERATOR_TRIM_SQL_FROM",
    "OPT_SCHEMA_AUTHORIZATION_OPTIONAL",
    "OPT_SCHEMA_AUTHORIZATION_UNSUPPORTED",
    "OPT_SCHEMA_FEATURES_DROP_CASCADE",
    "OPT_SCHEMA_FEATURES_DROP_RESTRICT",
    "OPT_SCHEMA_FEATURES_EMBEDDED_DDL",
    "OPT_SCHEMA_FEATURES_IF_NOT_EXISTS",
    "OPT_SCHEMA_FEATURES_IS_DATABASE",
    "OPT_SCHEMA_FEATURES_SUPPORTED",
    "OPT_TABLE_FEATURES_CHECK",
    "OPT_TABLE_FEATURES_EXTERNAL",
    "OPT_TABLE_FEATURES_FOREIGN_KEY",
    "OPT_TABLE_FEATURES_GENERATED_SUPPORTED",
    "OPT_TABLE_FEATURES_IF_NOT_EXISTS",
    "OPT_TABLE_FEATURES_ON_COMMIT",
    "OPT_TABLE_FEATURES_PRIMARY_KEY",
    "OPT_TABLE_FEATURES_REFERENTIAL_ACTION",
    "OPT_TABLE_FEATURES_UNIQUE",
    "OPT_TUPLE_ARITY_EMPTY",
    "OPT_TUPLE_ARITY_ONE",
    "OPT_TUPLE_SYNTAX_PARENTHESES",
    "OPT_TUPLE_SYNTAX_ROW",
    "OPT_TUPLE_SYNTAX_STRUCT",
    "OPT_TUPLE_SYNTAX_UNSUPPORTED",
    "AggregateConfig",
    "BitwiseConfig",
    "CastConfig",
    "Databricks",
    "Dialect",
    "DuckDb",
    "Hive",
    "IndexConfig",
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
    "TableConfig",
    "TemporalConfig",
    "Teradata",
    "TupleConfig",
]


Databricks = Dialect(
    placeholder="?",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        any=OPT_LIKE_ANY_PATTERN_LIST,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode_quote}'"),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_STRUCT, arity=OPT_TUPLE_ARITY_ONE),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(uuid="'{}'"),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_POSTFIX,
        function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE,
    ),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS,
        location=" LOCATION {}",
    ),
    index=IndexConfig(features=OPT_INDEX_FEATURES_SUPPORTED | OPT_INDEX_FEATURES_IF_NOT_EXISTS),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED
        | OPT_TABLE_FEATURES_IF_NOT_EXISTS
        | OPT_TABLE_FEATURES_EXTERNAL
        | _TABLE_KEYS,
        column_auto_increment="GENERATED ALWAYS AS IDENTITY",
        column_comment="COMMENT {}",
        temporary="TEMPORARY",
        comment="COMMENT {}",
        location="LOCATION {}",
    ),
)

DuckDb = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_REGEXP_MATCHES,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_FUNCTION),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_ROW, arity=OPT_TUPLE_ARITY_EMPTY | OPT_TUPLE_ARITY_ONE),
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
        trim=OPT_OPERATOR_TRIM_FUNCTION,
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_INSIDE_PARENS,
        function_filter=OPT_FUNCTION_FILTER_FILTER,
    ),
    schema=SchemaConfig(features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_IF_NOT_EXISTS
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_EXPRESSIONS,
        index_type=OPT_INDEX_TYPE_AFTER_COLUMNS,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED
        | OPT_INDEX_DROP_FEATURES_CASCADE
        | OPT_INDEX_DROP_FEATURES_RESTRICT,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED
        | OPT_TABLE_FEATURES_IF_NOT_EXISTS
        | _TABLE_CONSTRAINTS_NO_ACTION,
        column_auto_increment=None,
        column_comment=None,
        temporary="TEMPORARY",
    ),
)

Hive = Dialect(
    placeholder="?",
    like=LikeConfig(
        ilike=OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER,
        regex=OPT_LIKE_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode}'"),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_STRUCT, arity=OPT_TUPLE_ARITY_ONE),
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
        trim=OPT_OPERATOR_TRIM_SPACES_ONLY,
    ),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS | OPT_SCHEMA_FEATURES_IS_DATABASE,
        location=" LOCATION {}",
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_IF_NOT_EXISTS | OPT_TABLE_FEATURES_EXTERNAL,
        column_auto_increment=None,
        column_comment="COMMENT {}",
        temporary="TEMPORARY",
        comment="COMMENT {}",
        location="LOCATION {}",
    ),
)

MsSql = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_LIKE_ESCAPE_SCALAR),
    bitwise=BitwiseConfig(
        operations=OPT_BITWISE_OPERATIONS_AND | OPT_BITWISE_OPERATIONS_OR | OPT_BITWISE_OPERATIONS_NOT,
        xor=OPT_BITWISE_XOR_CARET,
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
        boolean=OPT_BOOL_FORMAT_INTEGER,
        bytes="0x{hex}",
        uuid="N'{}'",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_EMBEDDED_DDL,
        authorization=OPT_SCHEMA_AUTHORIZATION_OPTIONAL,
    ),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_COLUMN_ORDER
        | OPT_INDEX_FEATURES_PARTIAL
        | OPT_INDEX_FEATURES_INCLUDE,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
        drop_table=OPT_INDEX_DROP_TABLE_ON,
    ),
    table=TableConfig(
        # TODO(awolverp): MsSQL supports `OPT_TABLE_FEATURES_GENERATED_SUPPORTED`
        #                 but with a different syntax: `AS (expr) [PERSISTED]`
        features=_TABLE_CONSTRAINTS,
        column_auto_increment="IDENTITY(1,1)",
        column_comment=None,
    ),
)

MySql = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_LIKE_ESCAPE_SCALAR, regex=OPT_LIKE_REGEX_STANDARD),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{mysql}'", concat="CONCAT({left}, {right})"),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_ROW, arity=OPT_TUPLE_ARITY_ONE),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(
        bit="b'{}'",
        boolean=OPT_BOOL_FORMAT_INTEGER,
        uuid="'{}'",
    ),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS | OPT_SCHEMA_FEATURES_IS_DATABASE,
    ),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_COLUMN_ORDER
        | OPT_INDEX_FEATURES_EXPRESSIONS,
        index_type=OPT_INDEX_TYPE_AFTER_COLUMNS,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
        drop_table=OPT_INDEX_DROP_TABLE_ON,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED | OPT_TABLE_FEATURES_IF_NOT_EXISTS | _TABLE_CONSTRAINTS,
        column_auto_increment="AUTO_INCREMENT",
        column_comment="COMMENT {}",
        temporary="TEMPORARY",
        comment="COMMENT {}",
    ),
)

Oracle = Dialect(
    placeholder=":{}",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        ilike=OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER,
        regex=OPT_LIKE_REGEX_REGEXP_LIKE,
    ),
    bitwise=BitwiseConfig(operations=OPT_BITWISE_OPERATIONS_AND, syntax=OPT_BITWISE_SYNTAX_BIT_FUNCTION),
    cast=CastConfig(try_cast=None),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(
        boolean=OPT_BOOL_FORMAT_INTEGER,
        bytes="HEXTORAW('{hex}')",
        uuid="'{}'",
    ),
    operator=OperatorConfig(modulo=OPT_OPERATOR_MODULO_FUNCTION, collate=OPT_OPERATOR_COLLATE_INFIX_IDENT),
    aggregate=AggregateConfig(
        null_treatment=OPT_NULL_TREATMENT_POSTFIX,
        function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE,
    ),
    schema=SchemaConfig(features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_EMBEDDED_DDL),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_COLUMN_ORDER
        | OPT_INDEX_FEATURES_EXPRESSIONS,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED | OPT_TABLE_FEATURES_IF_NOT_EXISTS | _TABLE_CONSTRAINTS,
        column_auto_increment="GENERATED ALWAYS AS IDENTITY",
        column_comment=None,
        temporary="PRIVATE TEMPORARY",
    ),
)

PostgreSql = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        any=OPT_LIKE_ANY_ARRAY_CONSTRUCTOR,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_TILDE,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_HASH),
    string=StringConfig(template="E'{hex}'"),
    cast=CastConfig(try_cast=None, supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_ROW, arity=OPT_TUPLE_ARITY_EMPTY | OPT_TUPLE_ARITY_ONE),
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
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS | OPT_SCHEMA_FEATURES_EMBEDDED_DDL,
        authorization=OPT_SCHEMA_AUTHORIZATION_OPTIONAL,
    ),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_IF_NOT_EXISTS
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_COLUMN_ORDER
        | OPT_INDEX_FEATURES_EXPRESSIONS
        | OPT_INDEX_FEATURES_PARTIAL
        | OPT_INDEX_FEATURES_INCLUDE,
        index_type=OPT_INDEX_TYPE_BEFORE_COLUMNS,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED
        | OPT_INDEX_DROP_FEATURES_CASCADE
        | OPT_INDEX_DROP_FEATURES_RESTRICT,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED
        | OPT_TABLE_FEATURES_IF_NOT_EXISTS
        | OPT_TABLE_FEATURES_ON_COMMIT
        | _TABLE_CONSTRAINTS,
        column_auto_increment="GENERATED ALWAYS AS IDENTITY",
        column_comment=None,
        temporary="TEMPORARY",
    ),
)

Redshift = Dialect(
    placeholder="${}",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_TILDE,
        supports_similar_to=True,
    ),
    bitwise=BitwiseConfig(
        operations=OPT_BITWISE_OPERATIONS_AND | OPT_BITWISE_OPERATIONS_SHL | OPT_BITWISE_OPERATIONS_SHR
    ),
    cast=CastConfig(supports_double_colon=True),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(bytes="TO_VARBYTE('{hex}', 'hex')", uuid="'{}'"),
    operator=OperatorConfig(collate=OPT_OPERATOR_COLLATE_FUNCTION_STRING),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS | OPT_SCHEMA_FEATURES_EMBEDDED_DDL,
        authorization=OPT_SCHEMA_AUTHORIZATION_OPTIONAL,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_IF_NOT_EXISTS | _TABLE_KEYS,
        column_auto_increment="IDENTITY(1,1)",
        column_comment=None,
        temporary="TEMPORARY",
    ),
)

Snowflake = Dialect(
    placeholder=":{}",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR | OPT_LIKE_ESCAPE_ANY,
        any=OPT_LIKE_ANY_PATTERN_LIST,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(
        operations=_ALL_BITWISE,
        syntax=OPT_BITWISE_SYNTAX_BIT_FUNCTION,
        xor=OPT_BITWISE_XOR_BIT_FUNCTION,
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
        trim=OPT_OPERATOR_TRIM_FUNCTION,
        collate=OPT_OPERATOR_COLLATE_FUNCTION_STRING,
    ),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_POSTFIX),
    schema=SchemaConfig(
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS,
        location=" EXTERNAL_VOLUME = {}",
    ),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED | OPT_INDEX_FEATURES_IF_NOT_EXISTS | OPT_INDEX_FEATURES_INCLUDE,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
        drop_table=OPT_INDEX_DROP_TABLE_QUALIFIED,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED | OPT_TABLE_FEATURES_IF_NOT_EXISTS | _TABLE_CONSTRAINTS,
        column_auto_increment="AUTOINCREMENT",
        column_comment="COMMENT {}",
        temporary="TEMPORARY",
        comment="COMMENT = {}",
    ),
)

Spark = Dialect(
    placeholder="?",
    like=LikeConfig(
        escape=OPT_LIKE_ESCAPE_SCALAR,
        any=OPT_LIKE_ANY_PATTERN_LIST,
        ilike=OPT_I_LIKE_STRATEGY_STANDARD,
        regex=OPT_LIKE_REGEX_STANDARD,
    ),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE, xor=OPT_BITWISE_XOR_CARET),
    identifier="`{backtick}`",
    string=StringConfig(template="'{unicode_quote}'"),
    cast=CastConfig(supports_double_colon=True),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_STRUCT, arity=OPT_TUPLE_ARITY_ONE),
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
        features=OPT_SCHEMA_FEATURES_SUPPORTED | OPT_SCHEMA_FEATURES_IF_NOT_EXISTS | OPT_SCHEMA_FEATURES_IS_DATABASE,
        location=" LOCATION {}",
    ),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED | OPT_INDEX_FEATURES_IF_NOT_EXISTS,
        index_type=OPT_INDEX_TYPE_BEFORE_COLUMNS,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
        drop_table=OPT_INDEX_DROP_TABLE_ON,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_IF_NOT_EXISTS | OPT_TABLE_FEATURES_EXTERNAL | _TABLE_KEYS,
        column_auto_increment="GENERATED ALWAYS AS IDENTITY",
        column_comment="COMMENT {}",
        temporary="TEMPORARY",
        comment="COMMENT {}",
        location="LOCATION {}",
    ),
)

Sqlite = Dialect(
    like=LikeConfig(escape=OPT_LIKE_ESCAPE_SCALAR),
    bitwise=BitwiseConfig(operations=_ALL_BITWISE),
    cast=CastConfig(try_cast=None),
    tuple=TupleConfig(syntax=OPT_TUPLE_SYNTAX_PARENTHESES, arity=OPT_TUPLE_ARITY_EMPTY),
    temporal=TemporalConfig(
        date="'{}'",
        time="'{}'",
        datetime_naive="'{}'",
        datetime_aware="'{}'",
    ),
    literal=LiteralConfig(boolean=OPT_BOOL_FORMAT_INTEGER, uuid="'{}'"),
    operator=OperatorConfig(
        is_distinct_from=OPT_IS_DISTINCT_FROM_STANDARD,
        trim=OPT_OPERATOR_TRIM_FUNCTION,
        collate=OPT_OPERATOR_COLLATE_INFIX_IDENT,
    ),
    aggregate=AggregateConfig(function_filter=OPT_FUNCTION_FILTER_FILTER_WHERE),
    index=IndexConfig(
        features=OPT_INDEX_FEATURES_SUPPORTED
        | OPT_INDEX_FEATURES_IF_NOT_EXISTS
        | OPT_INDEX_FEATURES_UNIQUE
        | OPT_INDEX_FEATURES_COLUMN_ORDER
        | OPT_INDEX_FEATURES_EXPRESSIONS
        | OPT_INDEX_FEATURES_PARTIAL,
        drop_features=OPT_INDEX_DROP_FEATURES_SUPPORTED,
    ),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_IF_NOT_EXISTS | _TABLE_CONSTRAINTS,
        column_auto_increment="AUTOINCREMENT",
        column_comment=None,
        temporary="TEMPORARY",
    ),
)

Teradata = Dialect(
    placeholder="?",
    like=LikeConfig(escape=OPT_LIKE_ESCAPE_SCALAR, ilike=OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER),
    bitwise=BitwiseConfig(
        operations=_ALL_BITWISE,
        syntax=OPT_BITWISE_SYNTAX_SHIFT_FUNCTION,
        xor=OPT_BITWISE_XOR_BIT_FUNCTION,
    ),
    cast=CastConfig(try_cast="TRYCAST({expr} AS {type})"),
    temporal=TemporalConfig(
        date="DATE '{}'",
        time="'{}'",
        datetime_naive="TIMESTAMP '{}'",
        datetime_aware="TIMESTAMP WITH TIMEZONE '{}'",
    ),
    literal=LiteralConfig(bit="'{}'B", bytes="'{hex}'XBV", uuid="'{}'"),
    operator=OperatorConfig(modulo=OPT_OPERATOR_MODULO_INFIX),
    aggregate=AggregateConfig(null_treatment=OPT_NULL_TREATMENT_INSIDE_PARENS),
    table=TableConfig(
        features=OPT_TABLE_FEATURES_GENERATED_SUPPORTED | _TABLE_CONSTRAINTS_NO_ACTION,
        column_auto_increment="GENERATED ALWAYS AS IDENTITY(START WITH 1 INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 NO CYCLE)",
        column_comment=None,
        temporary="VOLATILE",
    ),
)
