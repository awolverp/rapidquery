#![allow(clippy::redundant_field_names)]

use crate::internal::template;

#[rapidquery_macros::pyclass("rapidquery.dialect.LikeConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LikeConfig {
    /// Where an inlined `LIKE` `ESCAPE` clause is legal.
    ///
    /// Empty writes no `ESCAPE` and rejects a set escape character.
    /// `SCALAR | ANY` allows both a scalar pattern and native `LIKE ANY`.
    ///
    /// - SCALAR: Scalar `LIKE` and an `OR` expansion.
    /// - ANY: Native `LIKE ANY (p, …) ESCAPE`.
    #[define(bits { SCALAR, ANY })]
    pub escape: LikeEscape,

    /// How `LIKE ANY` / `ILIKE ANY` is rendered.
    ///
    /// - OrExpansion: `(x LIKE p1 OR x LIKE p2)`. Useful when it isn't supported natively by the dialect.
    /// - ArrayConstructor: `LIKE ANY (ARRAY[p, …])`.
    /// - PatternList: `LIKE ANY (p, …)`.
    #[define(enum { #[default] OrExpansion, ArrayConstructor, PatternList })]
    pub any: LikeAny,

    /// Does the dialect support `ILIKE`, or should we fall back?
    ///
    /// - Unsupported
    /// - Standard: Native `ILIKE`.
    /// - UseLikeLower: Use `LIKE` with `LOWER()`.
    #[define(enum { #[default] Unsupported, Standard, UseLikeLower })]
    pub ilike: ILikeStrategy,

    /// How regex (`RLIKE` / `~` / `REGEXP_LIKE`, …) is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - Standard: `expr [NOT] RLIKE pattern`.
    /// - Tilde: `expr ~ pattern` / `expr !~ pattern`.
    /// - RegexpLike: `REGEXP_LIKE(expr, pattern)`.
    /// - RegexpContains: `REGEXP_CONTAINS(expr, pattern)`.
    /// - RegexpMatches: `regexp_matches(expr, pattern)`.
    #[define(enum { #[default] Unsupported, Standard, Tilde, RegexpLike, RegexpContains, RegexpMatches })]
    pub regex: LikeRegex,

    /// Whether the dialect supports the `SIMILAR TO` predicate.
    pub supports_similar_to: bool,
}

rapidquery_macros::pyo3_constructor! {
    LikeConfig => export_like_config_options {
        escape: bits LikeEscape { SCALAR, ANY } = empty,
        any: enum LikeAny { OrExpansion, ArrayConstructor, PatternList } = OrExpansion,
        ilike: enum ILikeStrategy { Unsupported, Standard, UseLikeLower } = Unsupported,
        regex: enum LikeRegex {
            Unsupported, Standard, Tilde, RegexpLike, RegexpContains, RegexpMatches
        } = Unsupported,
        supports_similar_to: bool = false,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.BitwiseConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitwiseConfig {
    /// Which bitwise operators a dialect can render.
    ///
    /// A missing flag rejects that operator. AND, OR, NOT, and the shifts share
    /// one spelling ([`BitwiseSyntax`]). XOR uses [`BitwiseXorSpelling`].
    #[define(bits { AND, OR, NOT, SHL, SHR })]
    pub operations: BitwiseOperations,

    /// Spelling shared by `&`, `|`, `~`, `<<`, and `>>`.
    ///
    /// - Infix: `& | ~ << >>`.
    /// - BitFunction: `BITAND` / `BITOR` / `BITNOT` / `BITSHIFTLEFT` / `BITSHIFTRIGHT`.
    /// - ShiftFunction: `BITAND` / `BITOR` / `BITNOT` / `SHIFTLEFT` / `SHIFTRIGHT`.
    #[define(enum { #[default] Infix, BitFunction, ShiftFunction })]
    pub syntax: BitwiseSyntax,

    /// How `^` is spelled when XOR is supported.
    ///
    /// - Unsupported
    /// - Caret: `left ^ right`.
    /// - Hash: `left # right`. PostgreSQL `^` is exponent.
    /// - Function: `xor(left, right)`.
    /// - BitFunction: `BITXOR(left, right)`.
    #[define(enum { #[default] Unsupported, Caret, Hash, Function, BitFunction })]
    pub xor: BitwiseXor,
}

rapidquery_macros::pyo3_constructor! {
    BitwiseConfig => export_bitwise_config_options {
        operations: bits BitwiseOperations { AND, OR, NOT, SHL, SHR } = empty,
        syntax: enum BitwiseSyntax { Infix, BitFunction, ShiftFunction } = Infix,
        xor: enum BitwiseXor { Unsupported, Caret, Hash, Function, BitFunction } = Unsupported,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.StringConfig", from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringConfig {
    /// String template e.g. `E'{hex}'`
    ///
    /// The template key picks how the text is escaped. A string containing `NUL` is rejected.
    /// Pick a key that escapes the delimiter used in your template: `'{single}'` or `"{double}"`.
    ///
    /// Escape by doubling (other characters are unchanged):
    /// - `single`: `'` → `''` (standard SQL, SQL Server, Oracle, SQLite, ...)
    /// - `double`: `"` → `""`
    ///
    /// Escape with a backslash (`\` is always escaped):
    /// - `hex`: Escape `\\`, `\'`, `\n`, `\r`, `\t`; other C0/DEL as `\xNN`. `"` is unchanged.
    /// - `hex_quote`: `hex` plus `\"`. Use it for `"` delimited strings.
    /// - `mysql`: (MySQL-specific) Escape `\\`, `\'`, `\"`, `\n`, `\r` and `\Z` for 0x1A; tab and other controls pass through.
    /// - `unicode`: Like `hex`, but other C0/DEL as `\uNNNN`. `"` is unchanged.
    /// - `unicode_quote`: `unicode` plus `\"`. Use it for `"` delimited strings.
    ///
    /// Only use a backslash key when the target database really reads backslash escapes
    /// inside string literals (e.g. MySQL, BigQuery, Hive, Spark, or PostgreSQL `E'...'`).
    ///
    /// Prefer using `StringConfig::write_string_literal` instead of directly using this template.
    pub template: template::Template,

    /// How binary operation string concatenation is spelled e.g. `{left} || {right}`
    ///
    /// Template keys are `left` and `right`.
    pub concat: template::Template,
}

rapidquery_macros::pyo3_constructor! {
    StringConfig => export_string_config_options {
        template: template(single?, double?, hex?, hex_quote?, mysql?, unicode?, unicode_quote?; 1) = "'{single}'",
        concat: template(left, right; 2) = "{left} || {right}",
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.CastConfig", from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CastConfig {
    /// How `TRY_CAST` is spelled e.g. `TRY_CAST({expr} AS {type})`.
    ///
    /// Template keys are `expr` and `type`.
    pub try_cast: Option<template::Template>,

    /// Whether the dialect supports the `expr::type` cast syntax.
    pub supports_double_colon: bool,
}

rapidquery_macros::pyo3_constructor! {
    CastConfig => export_cast_config_options {
        // NOTE: the old default was `"'{quote}'"` (copy-paste from StringConfig).
        try_cast: Option<template(expr, type; 2)> = "TRY_CAST({expr} AS {type})",
        supports_double_colon: bool = false,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.TupleConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TupleConfig {
    /// How a tuple / row value is spelled.
    ///
    /// Which arities are legal is [`TupleArity`], not another syntax variant.
    ///
    /// - Unsupported: Is not supported.
    /// - Row: `ROW(...)`.
    /// - Struct: `struct(...)`.
    /// - Parentheses: `(a, b, ...)`.
    #[define(enum { #[default] Unsupported, Row, Struct, Parentheses })]
    pub syntax: TupleSyntax,

    /// Arities below 2 that [`TupleSyntax`] accepts.
    ///
    /// Arity 2 and above is legal whenever the syntax is not [`TupleSyntax::Unsupported`].
    /// Empty means a tuple needs at least two elements.
    ///
    /// - EMPTY: Arity 0.
    /// - ONE: Arity 1.
    #[define(bits { EMPTY, ONE })]
    pub arity: TupleArity,
}

rapidquery_macros::pyo3_constructor! {
    TupleConfig => export_tuple_config_options {
        syntax: enum TupleSyntax { Unsupported, Row, Struct, Parentheses } = Unsupported,
        arity: bits TupleArity { EMPTY, ONE } = empty,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.TemporalConfig", from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalConfig {
    /// How a date literal is spelled e.g. `DATE '{}'`
    pub date: template::Template,

    /// How a time literal is spelled e.g. `TIME '{}'`
    pub time: template::Template,

    /// How a *naive* datetime/timestamp literal is spelled e.g. `TIMESTAMP '{}'`
    pub datetime_naive: template::Template,

    /// How a *aware* datetime/timestamp literal is spelled e.g. `TIMESTAMPTZ '{}'`
    pub datetime_aware: template::Template,
}

rapidquery_macros::pyo3_constructor! {
    TemporalConfig => export_temporal_config_options {
        date: template(_; 1) = "{}",
        time: template(_; 1) = "{}",
        datetime_naive: template(_; 1) = "{}",
        datetime_aware: template(_; 1) = "{}",
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.LiteralConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralConfig {
    /// How a boolean literal is spelled.
    ///
    /// - Keyword: `TRUE` / `FALSE`
    /// - Integer: `1` / `0`
    #[define(enum { #[default] Keyword, Integer })]
    pub boolean: BoolFormat,

    /// The way dialect accepts literal bit values e.g. `B'{}'`
    ///
    /// Prefer using `StringConfig::write_bits` instead of directly using this template.
    pub bit: Option<template::Template>,

    /// The way dialect accepts literal bytes values e.g. `X'{hex}'`
    ///
    /// Template keys are:
    /// - `hex`: bytes as hex (e.g. `DEADBEEF`)
    /// - `slashx`: each byte as `\xNN`
    ///
    /// Prefer using `StringConfig::write_bytes` instead of directly using this template.
    pub bytes: template::Template,

    /// How an inet literal is spelled e.g. `'{}'::inet`
    pub inet: Option<template::Template>,

    /// How a MAC address literal is spelled e.g. `'{}'::macaddr`
    pub macaddr: Option<template::Template>,

    /// How a UUID literal is spelled e.g. `'{}'::uuid`
    pub uuid: Option<template::Template>,

    /// How a JSON literal is spelled e.g. `JSON {}` (value is escaped and includes quotes itself).
    pub json: template::Template,

    /// The way dialect accepts array values e.g. `ARRAY[{}]`
    pub array: Option<template::Template>,
}

rapidquery_macros::pyo3_constructor! {
    LiteralConfig => export_literal_config_options {
        boolean: enum BoolFormat { Keyword, Integer } = Keyword,
        bit: Option<template(_; 1)> = None,
        bytes: template(hex?, slashx?; 1) = "X'{hex}'",
        inet: Option<template(_; 1)> = None,
        macaddr: Option<template(_; 1)> = None,
        uuid: Option<template(_; 1)> = None,
        json: template(_; 1) = "{}",
        array: Option<template(_; 1)> = None,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.OperatorConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatorConfig {
    /// How binary operation mod (`%`) is spelled.
    ///
    /// - Percent: `left % right`.
    /// - Function: `MOD(left, right)`.
    /// - Infix: `left MOD right`.
    #[define(enum { #[default] Percent, Function, Infix })]
    pub modulo: OperatorModulo,

    /// How `IS [NOT] DISTINCT FROM` is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - Standard: `left IS [NOT] DISTINCT FROM right`.
    /// - NullSafeEqual: `<=>` is null-safe equal (`IS NOT DISTINCT FROM`). `IS DISTINCT FROM` is `NOT (left <=> right)`.
    #[define(enum { #[default] Unsupported, Standard, NullSafeEqual })]
    pub is_distinct_from: IsDistinctFrom,

    /// How `TRIM` is spelled.
    ///
    /// - SqlFrom: `TRIM([LEADING|TRAILING] [chars FROM] expr)`.
    /// - Function: `TRIM` / `LTRIM` / `RTRIM` `(expr[, chars])`.
    /// - SpacesOnly: `TRIM` / `LTRIM` / `RTRIM` `(expr)`. A character set raises.
    #[define(enum { #[default] SqlFrom, Function, SpacesOnly })]
    pub trim: OperatorTrim,

    /// How `COLLATE` is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - InfixIdent: `expr COLLATE ident`.
    /// - FunctionString: `COLLATE(expr, 'body')`. The identifier body is a string literal.
    #[define(enum { #[default] Unsupported, InfixIdent, FunctionString })]
    pub collate: OperatorCollate,
}

rapidquery_macros::pyo3_constructor! {
    OperatorConfig => export_operator_config_options {
        modulo: enum OperatorModulo { Percent, Function, Infix } = Percent,
        is_distinct_from: enum IsDistinctFrom { Unsupported, Standard, NullSafeEqual } = Unsupported,
        trim: enum OperatorTrim { SqlFrom, Function, SpacesOnly } = SqlFrom,
        collate: enum OperatorCollate { Unsupported, InfixIdent, FunctionString } = Unsupported,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.SchemaConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaConfig {
    /// Supported independent schema features, or is it even supported?
    ///
    /// - SUPPORTED: Means dialect support schema.
    /// - IF_NOT_EXISTS: Is dialect support `IF NOT EXISTS` clause?
    /// - EMBEDDED_DDL: Is dialect support embedded DDL statements in `CREATE SCHEMA`?
    /// - IS_DATABASE: Schema has same meaning as database in this dialect.
    /// - DROP_CASCADE: Supports `CASCADE` keyword in `DROP SCHEMA`.
    /// - DROP_RESTRICT: Supports `RESTRICT` keyword in `DROP SCHEMA`.
    #[define(bits { SUPPORTED, IF_NOT_EXISTS, EMBEDDED_DDL, IS_DATABASE, DROP_CASCADE, DROP_RESTRICT })]
    pub features: SchemaFeatures,

    /// `AUTHORIZATION` clause style. How the schema owner is specified.
    ///
    /// - Unsupported: Means dialect does not support `AUTHORIZATION` for schema.
    /// - Optional: Means `AUTHORIZATION` clause is supported and it's optional.
    #[define(enum { #[default] Unsupported, Optional })]
    pub authorization: SchemaAuthorization,

    /// How schema location is expressed e.g. `LOCATION {}` (value is escaped and includes quotes itself).
    pub location: Option<template::Template>,
}

rapidquery_macros::pyo3_constructor! {
    SchemaConfig => export_schema_config_options {
        features: bits SchemaFeatures {
            SUPPORTED, IF_NOT_EXISTS, EMBEDDED_DDL, IS_DATABASE, DROP_CASCADE, DROP_RESTRICT
        } = empty,
        authorization: enum SchemaAuthorization { Unsupported, Optional } = Unsupported,
        location: Option<template(_; 1)> = None,
    }
}

#[rapidquery_macros::pyclass("rapidquery.dialect.AggregateConfig", from_py_object, eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AggregateConfig {
    /// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in functions.
    ///
    /// - Unsupported: Is not supported.
    /// - InsideParens: e.g. `FIRST_VALUE(x IGNORE NULLS)`.
    /// - Postfix: e.g. `FIRST_VALUE(x) IGNORE NULLS`.
    #[define(enum { #[default] Unsupported, InsideParens, Postfix })]
    pub null_treatment: NullTreatment,

    /// Function `FILTER` syntax.
    ///
    /// - Unsupported: Is not supported.
    /// - Filter: e.g. `FIRST_VALUE(x) FILTER (id > 10)`.
    /// - FilterWhere: e.g. `FIRST_VALUE(x) FILTER (WHERE id > 10)`.
    /// - InsideWhere: e.g. `FIRST_VALUE(x WHERE id > 10)`.
    #[define(enum { #[default] Unsupported, Filter, FilterWhere, InsideWhere })]
    pub function_filter: FunctionFilter,
}

rapidquery_macros::pyo3_constructor! {
    AggregateConfig => export_aggregate_config_options {
        null_treatment: enum NullTreatment { Unsupported, InsideParens, Postfix } = Unsupported,
        function_filter: enum FunctionFilter { Unsupported, Filter, FilterWhere, InsideWhere } = Unsupported,
    }
}

/// SQL dialect: quoting, literals, and which syntax is legal.
#[rapidquery_macros::pyclass("rapidquery.dialect.Dialect", eq, hash)]
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDialect {
    /// Positional parameter template e.g. `'?{}'` or `'?'`.
    ///
    /// `{}` is the 1-based index and may be omitted. `'?'` writes the marker
    /// alone. `'${}'` writes `$1`, `$2`, and so on.
    pub placeholder: template::Template,

    /// `LIKE` / `ILIKE` / regex configuration.
    pub like: LikeConfig,

    /// Bitwise operator support and spelling.
    pub bitwise: BitwiseConfig,

    /// Identifier template e.g. `"{double}"`
    ///
    /// The template key picks which character is escaped and how. Pick the key
    /// that matches the delimiter in your template, e.g. `` `{backtick}` `` or `[{bracket}]`.
    ///
    /// Escape by doubling (`x` becomes `xx`; other characters are unchanged):
    /// - `double`: `"` (PostgreSQL, SQLite, Oracle, Snowflake, ...)
    /// - `single`: `'`
    /// - `bracket`: `]` (SQL Server)
    /// - `backtick`: `` ` `` (MySQL, Hive, Spark, Databricks, ...)
    ///
    /// Escape with a backslash (`\` and the delimiter get a backslash; `\n`, `\r`, `\t` are
    /// written as escapes; other control characters as `\xNN`):
    /// - `slashdouble`: `"`
    /// - `slashsingle`: `'`
    /// - `slashbracket`: `]`
    /// - `slashbacktick`: `` ` `` (BigQuery)
    ///
    /// Only use a `slash*` key when the target database really reads backslash
    /// escapes inside quoted identifiers.
    pub identifier: template::Template,

    /// String literal format, escaping, and concatenation.
    pub string: StringConfig,

    /// Cast syntax (`TRY_CAST`, `::`, etc.).
    pub cast: CastConfig,

    /// Tuple / row-value syntax and arity support.
    pub tuple: TupleConfig,

    /// Date, time, and timestamp literal spelling.
    pub temporal: TemporalConfig,

    /// Other literal formats (bool, bit, bytes, UUID, JSON, …).
    pub literal: LiteralConfig,

    /// Miscellaneous operator spellings.
    pub operator: OperatorConfig,

    /// Aggregate syntax
    pub aggregate: AggregateConfig,

    /// Schema / DDL feature flags and clause styles.
    pub schema: SchemaConfig,
}

rapidquery_macros::pyo3_constructor! {
    PyDialect => export_pydialect_options {
        // `?` on the key lets a dialect omit `{}` (`'?'` writes the marker alone).
        placeholder: template(_?; 0..=1) = "?{}",
        like: class LikeConfig = LikeConfig::default(),
        bitwise: class BitwiseConfig = BitwiseConfig::default(),
        identifier: template(
            double?, single?, bracket?, backtick?,
            slashdouble?, slashsingle?, slashbracket?, slashbacktick?;
            1
        ) = "\"{double}\"",
        string: class StringConfig = StringConfig::default(),
        cast: class CastConfig = CastConfig::default(),
        tuple: class TupleConfig = TupleConfig::default(),
        temporal: class TemporalConfig = TemporalConfig::default(),
        literal: class LiteralConfig = LiteralConfig::default(),
        operator: class OperatorConfig = OperatorConfig::default(),
        aggregate: class AggregateConfig = AggregateConfig::default(),
        schema: class SchemaConfig = SchemaConfig::default(),
    }
}

#[cold]
pub(super) fn export_options(module: &::pyo3::Bound<'_, ::pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    export_aggregate_config_options(module)?;
    export_bitwise_config_options(module)?;
    export_cast_config_options(module)?;
    export_like_config_options(module)?;
    export_literal_config_options(module)?;
    export_operator_config_options(module)?;
    export_pydialect_options(module)?;
    export_schema_config_options(module)?;
    export_string_config_options(module)?;
    export_temporal_config_options(module)?;
    export_tuple_config_options(module)?;

    Ok(())
}
