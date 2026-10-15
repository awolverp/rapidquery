use bitfield_struct::bitenum;
use bitfield_struct::bitfield;
use bitflags::bitflags;

use crate::internal::utils;

use super::errors::FormatError;

/// Positional parameter syntax
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placeholder {
    /// e.g. `'?'`
    #[fallback]
    Simple,
    /// e.g. `'?1'`
    Numbered,
}

/// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NullTreatment {
    /// Is not supported
    #[fallback]
    Unsupported,
    /// e.g. `FIRST_VALUE(x IGNORE NULLS)`
    InsideParens,
    /// e.g. `FIRST_VALUE(x) IGNORE NULLS`
    Postfix,
}

bitflags! {
    /// Where an inlined `LIKE` `ESCAPE` clause is legal.
    ///
    /// Empty writes no `ESCAPE` and rejects a set escape character.
    /// `SCALAR | ANY` allows both a scalar pattern and native `LIKE ANY`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LikeEscape: u8 {
        /// Scalar `LIKE` and an `OR` expansion.
        const SCALAR = 1 << 0;
        /// Native `LIKE ANY (p, …) ESCAPE`.
        const ANY = 1 << 1;
    }
}

const fn like_escape_into(value: LikeEscape) -> u8 {
    value.bits()
}

const fn like_escape_from(bits: u8) -> LikeEscape {
    LikeEscape::from_bits_retain(bits)
}

/// How `LIKE ANY` / `ILIKE ANY` is rendered.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LikeAny {
    /// `(x LIKE p1 OR x LIKE p2)`.
    /// Useful when it isn't supported natively by dialect
    #[fallback]
    OrExpansion,
    /// `LIKE ANY (ARRAY[p, …])`.
    ArrayConstructor,
    /// `LIKE ANY (p, …)`.
    PatternList,
}

/// Does dialect support `ILIKE` expression or we should fallback to another options?
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ILikeExpression {
    #[fallback]
    Standard,
    /// Use `LIKE` expression and `LOWER()` function
    UseLikeLower,
    /// Fallback to `LIKE` expression
    UseLike,
}

/// How regex `RLike` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RLikeExpression {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `expr [NOT] RLIKE pattern`
    Standard,
    /// `expr ~ pattern` / `expr !~ pattern`
    Tilde,
    /// `REGEXP_LIKE(expr, pattern)`
    RegexpLike,
    /// `REGEXP_CONTAINS(expr, pattern)`
    RegexpContains,
    /// `regexp_matches(expr, pattern)`
    RegexpMatches,
}

/// How binary operation StringConcat (`||`) is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringConcat {
    /// `left || right`.
    #[fallback]
    Pipes,
    /// `left + right`. SQL Server `||` is 2025+; classic concat is `+`.
    Plus,
    /// `CONCAT(left, right)`. MySQL `||` is logical OR unless `PIPES_AS_CONCAT`.
    ConcatFunction,
}

bitflags! {
    /// Which bitwise operators a dialect can render.
    ///
    /// A missing flag rejects that operator. AND, OR, NOT, and the shifts share
    /// one spelling ([`BitCall`]). XOR uses [`XorSpelling`] and only when
    /// [`Self::XOR`] is set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BitOp: u8 {
        const AND = 1 << 0;
        const OR = 1 << 1;
        const NOT = 1 << 2;
        const SHL = 1 << 3;
        const SHR = 1 << 4;
        const XOR = 1 << 5;

        /// `& | ~ << >>`, without XOR.
        const INFIX = Self::AND.bits() | Self::OR.bits() | Self::NOT.bits() | Self::SHL.bits() | Self::SHR.bits();
        /// `& << >>`.
        const AND_SHIFT = Self::AND.bits() | Self::SHL.bits() | Self::SHR.bits();
        /// `& | ~`, no shifts.
        const NO_SHIFT = Self::AND.bits() | Self::OR.bits() | Self::NOT.bits();
    }
}

const fn bit_op_into(value: BitOp) -> u8 {
    value.bits()
}

const fn bit_op_from(bits: u8) -> BitOp {
    BitOp::from_bits_retain(bits)
}

/// Spelling shared by `&`, `|`, `~`, `<<`, and `>>`.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitCall {
    /// `& | ~ << >>`.
    #[fallback]
    Infix,
    /// `BITAND` / `BITOR` / `BITNOT` / `BITSHIFTLEFT` / `BITSHIFTRIGHT`.
    BitFunction,
    /// `BITAND` / `BITOR` / `BITNOT` / `SHIFTLEFT` / `SHIFTRIGHT`.
    ShiftFunction,
}

/// How `^` is spelled when [`BitOp::XOR`] is set.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XorSpelling {
    /// `left ^ right`.
    #[fallback]
    Caret,
    /// `left # right`. PostgreSQL `^` is exponent.
    Hash,
    /// `xor(left, right)`.
    Function,
    /// `BITXOR(left, right)`.
    BitFunction,
}

/// How binary operation mod (`%`) is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modulo {
    /// `left % right`.
    #[fallback]
    Percent,
    /// `MOD(left, right)`.
    Function,
    /// `left MOD right`.
    Infix,
}

/// How `IS [NOT] DISTINCT FROM` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsDistinctFrom {
    /// Is not supported
    Unsupported,
    /// `left IS [NOT] DISTINCT FROM right`.
    #[fallback]
    Standard,
    /// `<=>` is null-safe equal (`IS NOT DISTINCT FROM`).
    /// `IS DISTINCT FROM` is `NOT (left <=> right)`.
    NullSafeEqual,
}

/// How `TRY_CAST` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TryCast {
    /// Is not supported
    Unsupported,
    /// `TRY_CAST(expr AS type)`.
    #[fallback]
    TryCast,
    /// `TRYCAST(expr AS type)`.
    TryCastWord,
    /// `SAFE_CAST(expr AS type)`. BigQuery's spelling of a null-on-failure cast.
    SafeCast,
}

/// How `TRIM` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trim {
    /// `TRIM([LEADING|TRAILING] [chars FROM] expr)`.
    #[fallback]
    SqlFrom,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr[, chars])`.
    Function,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr)`. A character set raises.
    SpacesOnly,
}

/// How `COLLATE` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Collate {
    /// Is not supported
    Unsupported,
    /// `expr COLLATE ident`.
    #[fallback]
    InfixIdent,
    /// `COLLATE(expr, 'body')`. The identifier body is a string literal.
    FunctionString,
}

/// How a tuple / row value is spelled.
///
/// Which arities are legal is [`TupleArity`], not another syntax variant.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TupleSyntax {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `ROW(...)`.
    Row,
    /// `struct(...)`.
    Struct,
    /// `(a, b, ...)`.
    Parentheses,
}

bitflags! {
    /// Arities below 2 that [`TupleSyntax`] accepts.
    ///
    /// Arity 2 and above is legal whenever the syntax is not [`TupleSyntax::Unsupported`].
    /// Empty means a tuple needs at least two elements.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TupleArity: u8 {
        /// Arity 0 (`ROW()`).
        const EMPTY = 1 << 0;
        /// Arity 1.
        const ONE = 1 << 1;

        /// Arity 0 and arity 1.
        const EMPTY_AND_ONE = Self::EMPTY.bits() | Self::ONE.bits();
    }
}

const fn tuple_arity_into(value: TupleArity) -> u8 {
    value.bits()
}

const fn tuple_arity_from(bits: u8) -> TupleArity {
    TupleArity::from_bits_retain(bits)
}

/// The way dialect accepts literal bit values
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitFormat {
    /// Is not supported.
    Unsupported,
    /// e.g. `B'101'`
    #[fallback]
    PrefixBigB,
    /// e.g. `b'101'`
    PrefixSmallB,
    /// e.g. `'101'B`
    PostfixBigB,
}

/// How a boolean literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`
    #[fallback]
    Keyword,
    /// `1` / `0`
    Integer,
}

/// Delimiters wrapped around a quoted identifier.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierFormat {
    /// `"name"`
    #[fallback]
    DoubleQuote,
    /// `` `name` ``
    Backtick,
    /// `[name]`
    SquareBracket,
}

/// How to escape identifiers.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierEscape {
    /// Escape identifier delimiters with doubling.
    #[fallback]
    Double,
    /// Escape identifier delimiters with backslash.
    Backslash,
}

/// Delimiters wrapped around a literal string.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringFormat {
    /// `'string'`
    #[fallback]
    SingleQuote,
    /// `E'string'`
    PrefixBigE,
    /// `N'string'`
    PrefixBigN,
}

/// How to escape literal strings.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringEscape {
    /// `'` → `''`. Other characters are unchanged.
    #[fallback]
    Quote,
    /// `\\`, `\'`, `\n`, `\r`, `\t`; other C0/DEL as `\xNN`.
    BackslashHex,
    /// Also `\"` and `\Z` for 0x1A; tab and other controls pass through.
    BackslashMysql,
    /// Other C0/DEL as `\uNNNN`. No `\"`.
    BackslashUnicode,
    /// Other C0/DEL as `\xNN`.
    BackslashHexZero,
    /// [`Self::BackslashUnicode`] plus `\"`.
    BackslashUnicodeQuote,
}

/// The way dialect accepts literal bytes values.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BytesFormat {
    /// `X'DEADBEEF'`
    #[fallback]
    PrefixBigX,
    /// `0xDEADBEEF`
    PrefixZeroX,
    /// `'\xDEADBEEF'::bytea`
    SlashXBytea,
    /// `B'\xNN…'` (each byte as `\xNN`)
    PrefixBigBSlashX,
    /// `HEXTORAW('DEADBEEF')`
    HexToRaw,
    /// `UNHEX('DEADBEEF')`
    Unhex,
    /// `TO_VARBYTE('DEADBEEF', 'hex')`
    ToVarbyte,
    /// `'DEADBEEF'XBV`
    PostfixXbv,
    /// `TO_BINARY('DEADBEEF', 'HEX')`
    ToBinary,
    /// `'\xHH…'::BLOB`; empty is `''::BLOB`
    SlashXBlob,
}

/// Function `FILTER` syntax.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionFilter {
    /// Is not supported.
    Unsupported,
    /// e.g. FIRST_VALUE(x) FILTER (id > 10)
    Filter,
    /// e.g. FIRST_VALUE(x) FILTER (WHERE id > 10)
    #[fallback]
    FilterWhere,
    /// e.g. FIRST_VALUE(x WHERE id > 10)
    InsideWhere,
}

/// The way dialect accepts array values.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `ARRAY[…]`
    Constructor,
    /// `[…]`
    List,
    /// `array(…)`
    Function,
}

/// How a date literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// Quoted string.
    #[fallback]
    Quoted,
    /// `DATE`, a space, then a quoted literal.
    Keyword,
}

/// How a time literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFormat {
    /// Quoted string.
    #[fallback]
    Quoted,
    /// `TIME`, a space, then a quoted literal.
    Keyword,
}

/// How a timestamp literal is spelled.
///
/// Naive values use `TIMESTAMP` when a keyword is configured. Aware values
/// (`datetime` with a tzinfo, or a UTC epoch number) use the keyword this
/// variant names.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimestampFormat {
    /// Both naive and aware values are a quoted string. No keyword.
    Quoted,
    /// Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMPTZ '…'`.
    Timestamptz,
    /// Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMP WITH TIMEZONE '…'`.
    #[fallback]
    WithTimezone,
}

/// How an inet literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InetFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `'192.168.1.1/32'`
    Quoted,
    /// `'192.168.1.1/32'::inet`
    Cast,
}

/// How a MAC address literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacAddrFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `'01:23:45:67:89:AB'`
    Quoted,
    /// `'01:23:45:67:89:AB'::macaddr`
    Cast,
}

/// How a UUID literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UuidFormat {
    /// Quoted hyphenated text.
    #[fallback]
    Quoted,
    /// `UUID '…'`
    UuidKeyword,
    /// `'…'::uuid`
    CastUuid,
}

/// How a JSON literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonFormat {
    /// Quoted JSON text.
    #[fallback]
    Quoted,
    /// `JSON '…'`
    JsonKeyword,
    /// `PARSE_JSON('…')`
    ParseJson,
}

/// Placeholder character plus `?` versus `?1`.
#[bitfield(u64)]
#[derive(PartialEq, Eq, Hash)]
pub struct Placeholders {
    /// Placeholder character as `u32`. The caller stores a valid `char`.
    #[bits(32)]
    pub character: u32,
    /// `?` or `?1`.
    #[bits(1)]
    pub style: Placeholder,
    #[bits(31)]
    __: u32,
}

/// `LIKE` / `ILIKE` spelling and where `ESCAPE` is legal.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Like {
    #[bits(2, from = like_escape_from, into = like_escape_into)]
    pub escape: LikeEscape,
    #[bits(2)]
    pub any: LikeAny,
    #[bits(2)]
    pub ilike: ILikeExpression,
    /// Whether `SIMILAR TO` is supported.
    pub similar_to: bool,
    #[bits(1)]
    __: u8,
}

/// Postfix clauses on a function: null treatment and `FILTER`.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Aggregate {
    #[bits(2)]
    pub nulls: NullTreatment,
    #[bits(2)]
    pub filter: FunctionFilter,
    #[bits(4)]
    __: u8,
}

/// Bitwise operators: which exist, how AND/OR/NOT/shifts are spelled, and XOR.
///
/// `call` is shared. The builtins never mix an infix AND with a function OR.
/// XOR does not use `call`.
#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
pub struct Bitwise {
    #[bits(6, from = bit_op_from, into = bit_op_into)]
    pub ops: BitOp,
    #[bits(2)]
    pub call: BitCall,
    #[bits(2)]
    pub xor: XorSpelling,
    #[bits(6)]
    __: u8,
}

/// Identifier delimiter and how that delimiter is escaped.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Identifier {
    #[bits(2)]
    pub format: IdentifierFormat,
    #[bits(1)]
    pub escape: IdentifierEscape,
    #[bits(5)]
    __: u8,
}

/// String-literal prefix and escape style.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct StringLiteral {
    #[bits(2)]
    pub format: StringFormat,
    #[bits(3)]
    pub escape: StringEscape,
    #[bits(3)]
    __: u8,
}

/// `TRY_CAST` spelling and whether `expr::type` is legal.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Cast {
    #[bits(2)]
    pub try_cast: TryCast,
    pub double_colon: bool,
    #[bits(5)]
    __: u8,
}

/// Tuple syntax and which arities below 2 are legal.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct TupleFlags {
    #[bits(2)]
    pub syntax: TupleSyntax,
    #[bits(2, from = tuple_arity_from, into = tuple_arity_into)]
    pub arity: TupleArity,
    #[bits(4)]
    __: u8,
}

/// Date, time, and timestamp literal spellings.
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Temporal {
    #[bits(1)]
    pub date: DateFormat,
    #[bits(1)]
    pub time: TimeFormat,
    #[bits(2)]
    pub timestamp: TimestampFormat,
    #[bits(4)]
    __: u8,
}

/// Scalar and typed literal spellings.
#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
pub struct Literals {
    #[bits(1)]
    pub bool_format: BoolFormat,
    #[bits(2)]
    pub bit_format: BitFormat,
    #[bits(4)]
    pub bytes_format: BytesFormat,
    #[bits(2)]
    pub inet: InetFormat,
    #[bits(2)]
    pub macaddr: MacAddrFormat,
    #[bits(2)]
    pub uuid: UuidFormat,
    #[bits(2)]
    pub json: JsonFormat,
    #[bits(1)]
    __: u8,
}

/// Packed options that decide how one dialect spells SQL.
///
/// Related leaves share one nested field. A bitenum is one spelling. A
/// bitflags field is a set that can hold several values at once. Builtin
/// dialects set every leaf. A flag the writer does not implement makes
/// rendering fail.
///
/// ```
/// use rapidquery::dialect::flags::BoolFormat;
/// use rapidquery::dialect::flags::DialectFlags;
///
/// let flags = DialectFlags::new().with_bool_format(BoolFormat::Integer);
/// let mut out = String::new();
/// flags.write_boolean(&mut out, true).unwrap();
/// assert_eq!(out, "1");
/// ```
#[bitfield(u128)]
#[derive(PartialEq, Eq, Hash)]
pub struct DialectFlags {
    /// Placeholder character and `?` versus numbered `?1`.
    #[bits(33)]
    pub placeholders: Placeholders,
    /// `LIKE`, `ILIKE`, `ESCAPE`, and `SIMILAR TO`.
    #[bits(7)]
    pub like: Like,
    /// `{IGNORE|RESPECT} NULLS` and `FILTER`.
    #[bits(4)]
    pub aggregate: Aggregate,
    /// Bitwise operators and XOR. `BitOp::INFIX` is not the zero value.
    #[bits(10)]
    pub bitwise: Bitwise,
    /// Identifier delimiter and escape.
    #[bits(3)]
    pub identifier: Identifier,
    /// String literal prefix and escape.
    #[bits(5)]
    pub string: StringLiteral,
    /// `TRY_CAST` and `::`.
    #[bits(3)]
    pub cast: Cast,
    /// Tuple syntax and arity. The field is `tuples` so `tuple()` stays the syntax leaf.
    #[bits(4)]
    pub tuples: TupleFlags,
    /// Date, time, and timestamp literals.
    #[bits(4)]
    pub temporal: Temporal,
    /// Boolean, bit, bytes, inet, MAC, UUID, and JSON literals.
    #[bits(15)]
    pub literals: Literals,
    /// How binary operation StringConcat (`||`) is spelled.
    #[bits(2)]
    pub string_concat: StringConcat,
    /// How binary operation mod (`%`) is spelled.
    #[bits(2)]
    pub modulo: Modulo,
    /// How `IS [NOT] DISTINCT FROM` is spelled.
    #[bits(2)]
    pub is_distinct_from: IsDistinctFrom,
    /// How TRIM is spelled.
    #[bits(2)]
    pub trim: Trim,
    /// How `COLLATE` is spelled.
    #[bits(2)]
    pub collate: Collate,
    /// The way dialect accepts array values.
    #[bits(2)]
    pub array_format: ArrayFormat,
    /// How regex `RLike` is spelled.
    #[bits(3)]
    pub rlike_expression: RLikeExpression,
    /// Wrap `IN` / `BETWEEN` in parentheses.
    ///
    /// SQL Server ranks `AND` above `IN`/`BETWEEN`/`OR`, and those four share
    /// a level evaluated left to right. The parentheses keep the predicate
    /// from being parsed as part of the left operand.
    #[bits(1)]
    pub parenthesize_in_between: bool,

    // Padding. 33+7+4+10+3+5+3+4+4+15+2+2+2+2+2+2+3+1 = 104.
    #[bits(24)]
    __: u32,
}

impl DialectFlags {
    /// Stored placeholder character. The caller keeps this a valid `char`.
    #[inline]
    pub const fn placeholder_character(&self) -> u32 {
        self.placeholders().character()
    }

    /// Set the placeholder character.
    #[inline]
    pub const fn with_placeholder_character(self, character: u32) -> Self {
        self.with_placeholders(self.placeholders().with_character(character))
    }

    /// `?` or numbered `?1`.
    #[inline]
    pub const fn placeholder(&self) -> Placeholder {
        self.placeholders().style()
    }

    /// Set `?` or numbered `?1`.
    #[inline]
    pub const fn with_placeholder(self, style: Placeholder) -> Self {
        self.with_placeholders(self.placeholders().with_style(style))
    }

    /// Where `ESCAPE` is legal. Empty rejects it.
    #[inline]
    pub const fn like_escape(&self) -> LikeEscape {
        self.like().escape()
    }

    /// Set where `ESCAPE` is legal.
    #[inline]
    pub const fn with_like_escape(self, escape: LikeEscape) -> Self {
        self.with_like(self.like().with_escape(escape))
    }

    /// How `LIKE ANY` is spelled.
    #[inline]
    pub const fn like_any(&self) -> LikeAny {
        self.like().any()
    }

    /// Set how `LIKE ANY` is spelled.
    #[inline]
    pub const fn with_like_any(self, any: LikeAny) -> Self {
        self.with_like(self.like().with_any(any))
    }

    /// How `ILIKE` is spelled.
    #[inline]
    pub const fn ilike_expression(&self) -> ILikeExpression {
        self.like().ilike()
    }

    /// Set how `ILIKE` is spelled.
    #[inline]
    pub const fn with_ilike_expression(self, ilike: ILikeExpression) -> Self {
        self.with_like(self.like().with_ilike(ilike))
    }

    /// Whether `SIMILAR TO` can be rendered.
    #[inline]
    pub const fn supports_similar_to(&self) -> bool {
        self.like().similar_to()
    }

    /// Set whether `SIMILAR TO` can be rendered.
    #[inline]
    pub const fn with_supports_similar_to(self, value: bool) -> Self {
        self.with_like(self.like().with_similar_to(value))
    }

    /// Where `{IGNORE|RESPECT} NULLS` is written.
    #[inline]
    pub const fn null_treatment(&self) -> NullTreatment {
        self.aggregate().nulls()
    }

    /// Set where `{IGNORE|RESPECT} NULLS` is written.
    #[inline]
    pub const fn with_null_treatment(self, value: NullTreatment) -> Self {
        self.with_aggregate(self.aggregate().with_nulls(value))
    }

    /// How aggregate `FILTER` is spelled.
    #[inline]
    pub const fn function_filter(&self) -> FunctionFilter {
        self.aggregate().filter()
    }

    /// Set how aggregate `FILTER` is spelled.
    #[inline]
    pub const fn with_function_filter(self, value: FunctionFilter) -> Self {
        self.with_aggregate(self.aggregate().with_filter(value))
    }

    /// Which bitwise operators exist.
    #[inline]
    pub const fn bit_ops(&self) -> BitOp {
        self.bitwise().ops()
    }

    /// Set which bitwise operators exist.
    #[inline]
    pub const fn with_bit_ops(self, ops: BitOp) -> Self {
        self.with_bitwise(self.bitwise().with_ops(ops))
    }

    /// Spelling of AND, OR, NOT, and shifts.
    #[inline]
    pub const fn bit_call(&self) -> BitCall {
        self.bitwise().call()
    }

    /// Set the spelling of AND, OR, NOT, and shifts.
    #[inline]
    pub const fn with_bit_call(self, call: BitCall) -> Self {
        self.with_bitwise(self.bitwise().with_call(call))
    }

    /// XOR spelling. Ignored unless [`BitOp::XOR`] is set.
    #[inline]
    pub const fn xor(&self) -> XorSpelling {
        self.bitwise().xor()
    }

    /// Set the XOR spelling.
    #[inline]
    pub const fn with_xor(self, xor: XorSpelling) -> Self {
        self.with_bitwise(self.bitwise().with_xor(xor))
    }

    /// Identifier delimiter.
    #[inline]
    pub const fn identifier_format(&self) -> IdentifierFormat {
        self.identifier().format()
    }

    /// Set the identifier delimiter.
    #[inline]
    pub const fn with_identifier_format(self, format: IdentifierFormat) -> Self {
        self.with_identifier(self.identifier().with_format(format))
    }

    /// How an identifier escapes its delimiter.
    #[inline]
    pub const fn identifier_escape(&self) -> IdentifierEscape {
        self.identifier().escape()
    }

    /// Set how an identifier escapes its delimiter.
    #[inline]
    pub const fn with_identifier_escape(self, escape: IdentifierEscape) -> Self {
        self.with_identifier(self.identifier().with_escape(escape))
    }

    /// String-literal prefix.
    #[inline]
    pub const fn string_format(&self) -> StringFormat {
        self.string().format()
    }

    /// Set the string-literal prefix.
    #[inline]
    pub const fn with_string_format(self, format: StringFormat) -> Self {
        self.with_string(self.string().with_format(format))
    }

    /// String escape style.
    #[inline]
    pub const fn string_escape(&self) -> StringEscape {
        self.string().escape()
    }

    /// Set the string escape style.
    #[inline]
    pub const fn with_string_escape(self, escape: StringEscape) -> Self {
        self.with_string(self.string().with_escape(escape))
    }

    /// How `TRY_CAST` is spelled.
    #[inline]
    pub const fn try_cast(&self) -> TryCast {
        self.cast().try_cast()
    }

    /// Set how `TRY_CAST` is spelled.
    #[inline]
    pub const fn with_try_cast(self, value: TryCast) -> Self {
        self.with_cast(self.cast().with_try_cast(value))
    }

    /// Whether `expr::type` is legal.
    #[inline]
    pub const fn supports_double_colon(&self) -> bool {
        self.cast().double_colon()
    }

    /// Set whether `expr::type` is legal.
    #[inline]
    pub const fn with_supports_double_colon(self, value: bool) -> Self {
        self.with_cast(self.cast().with_double_colon(value))
    }

    /// Tuple syntax. Arity limits are [`Self::tuple_arity`].
    #[inline]
    pub const fn tuple(&self) -> TupleSyntax {
        self.tuples().syntax()
    }

    /// Set the tuple syntax.
    #[inline]
    pub const fn with_tuple(self, syntax: TupleSyntax) -> Self {
        self.with_tuples(self.tuples().with_syntax(syntax))
    }

    /// Arities below 2 that the tuple syntax accepts.
    #[inline]
    pub const fn tuple_arity(&self) -> TupleArity {
        self.tuples().arity()
    }

    /// Set arities below 2 that the tuple syntax accepts.
    #[inline]
    pub const fn with_tuple_arity(self, arity: TupleArity) -> Self {
        self.with_tuples(self.tuples().with_arity(arity))
    }

    /// How a date literal is spelled.
    #[inline]
    pub const fn date_format(&self) -> DateFormat {
        self.temporal().date()
    }

    /// Set how a date literal is spelled.
    #[inline]
    pub const fn with_date_format(self, value: DateFormat) -> Self {
        self.with_temporal(self.temporal().with_date(value))
    }

    /// How a time literal is spelled.
    #[inline]
    pub const fn time_format(&self) -> TimeFormat {
        self.temporal().time()
    }

    /// Set how a time literal is spelled.
    #[inline]
    pub const fn with_time_format(self, value: TimeFormat) -> Self {
        self.with_temporal(self.temporal().with_time(value))
    }

    /// How a timestamp literal is spelled.
    #[inline]
    pub const fn timestamp_format(&self) -> TimestampFormat {
        self.temporal().timestamp()
    }

    /// Set how a timestamp literal is spelled.
    #[inline]
    pub const fn with_timestamp_format(self, value: TimestampFormat) -> Self {
        self.with_temporal(self.temporal().with_timestamp(value))
    }

    /// How a boolean literal is spelled.
    #[inline]
    pub const fn bool_format(&self) -> BoolFormat {
        self.literals().bool_format()
    }

    /// Set how a boolean literal is spelled.
    #[inline]
    pub const fn with_bool_format(self, value: BoolFormat) -> Self {
        self.with_literals(self.literals().with_bool_format(value))
    }

    /// How a bit-string literal is spelled.
    #[inline]
    pub const fn bit_format(&self) -> BitFormat {
        self.literals().bit_format()
    }

    /// Set how a bit-string literal is spelled.
    #[inline]
    pub const fn with_bit_format(self, value: BitFormat) -> Self {
        self.with_literals(self.literals().with_bit_format(value))
    }

    /// How a byte literal is spelled.
    #[inline]
    pub const fn bytes_format(&self) -> BytesFormat {
        self.literals().bytes_format()
    }

    /// Set how a byte literal is spelled.
    #[inline]
    pub const fn with_bytes_format(self, value: BytesFormat) -> Self {
        self.with_literals(self.literals().with_bytes_format(value))
    }

    /// How an inet literal is spelled.
    #[inline]
    pub const fn inet_format(&self) -> InetFormat {
        self.literals().inet()
    }

    /// Set how an inet literal is spelled.
    #[inline]
    pub const fn with_inet_format(self, value: InetFormat) -> Self {
        self.with_literals(self.literals().with_inet(value))
    }

    /// How a MAC address literal is spelled.
    #[inline]
    pub const fn macaddr_format(&self) -> MacAddrFormat {
        self.literals().macaddr()
    }

    /// Set how a MAC address literal is spelled.
    #[inline]
    pub const fn with_macaddr_format(self, value: MacAddrFormat) -> Self {
        self.with_literals(self.literals().with_macaddr(value))
    }

    /// How a UUID literal is spelled.
    #[inline]
    pub const fn uuid_format(&self) -> UuidFormat {
        self.literals().uuid()
    }

    /// Set how a UUID literal is spelled.
    #[inline]
    pub const fn with_uuid_format(self, value: UuidFormat) -> Self {
        self.with_literals(self.literals().with_uuid(value))
    }

    /// How a JSON literal is spelled.
    #[inline]
    pub const fn json_format(&self) -> JsonFormat {
        self.literals().json()
    }

    /// Set how a JSON literal is spelled.
    #[inline]
    pub const fn with_json_format(self, value: JsonFormat) -> Self {
        self.with_literals(self.literals().with_json(value))
    }

    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn write_boolean<W: std::fmt::Write>(&self, writer: &mut W, value: bool) -> Result<(), FormatError> {
        match (self.bool_format(), value) {
            (BoolFormat::Integer, false) => writer.write_str("0")?,
            (BoolFormat::Integer, true) => writer.write_str("1")?,
            (BoolFormat::Keyword, false) => writer.write_str("FALSE")?,
            (BoolFormat::Keyword, true) => writer.write_str("TRUE")?,
        }
        Ok(())
    }

    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    pub(super) fn escape_string<W: std::fmt::Write>(&self, writer: &mut W, string: &str) -> Result<(), FormatError> {
        let result = match self.string_escape() {
            StringEscape::Quote => utils::replace_to(string, '\'', "''", writer),
            StringEscape::BackslashHex => write_backslash_hex(writer, string),
            StringEscape::BackslashMysql => write_backslash_mysql(writer, string),
            StringEscape::BackslashUnicode => write_backslash_unicode(writer, string, false),
            StringEscape::BackslashHexZero => write_backslash_hex_zero(writer, string),
            StringEscape::BackslashUnicodeQuote => write_backslash_unicode(writer, string, true),
        };
        result.map_err(Into::into)
    }

    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn write_identifier<W: std::fmt::Write>(&self, writer: &mut W, ident: &str) -> std::fmt::Result {
        let (start, end) = match self.identifier_format() {
            IdentifierFormat::DoubleQuote => ('"', '"'),
            IdentifierFormat::Backtick => ('`', '`'),
            IdentifierFormat::SquareBracket => ('[', ']'),
        };
        writer.write_char(start)?;

        match self.identifier_escape() {
            IdentifierEscape::Double => match self.identifier_format() {
                IdentifierFormat::DoubleQuote => {
                    utils::replace_to(ident, '"', "\"\"", writer)?;
                }
                IdentifierFormat::Backtick => {
                    utils::replace_to(ident, '`', "``", writer)?;
                }
                IdentifierFormat::SquareBracket => {
                    utils::replace_to(ident, ']', "]]", writer)?;
                }
            },
            IdentifierEscape::Backslash => match self.identifier_format() {
                IdentifierFormat::DoubleQuote => {
                    utils::replace_to(ident, '"', "\\\"", writer)?;
                }
                IdentifierFormat::Backtick => {
                    write_identifier_backtick_escape(writer, ident)?;
                }
                IdentifierFormat::SquareBracket => {
                    utils::replace_to(ident, ']', "\\]", writer)?;
                }
            },
        }

        writer.write_char(end)
    }

    /// Escape `raw` and wrap it as a complete string literal.
    #[inline]
    pub fn write_string_literal<W: std::fmt::Write>(&self, writer: &mut W, raw: &str) -> Result<(), FormatError> {
        if raw.contains('\0') {
            return Err(FormatError::NulInString);
        }

        match self.string_format() {
            StringFormat::PrefixBigE => writer.write_str("E'")?,
            StringFormat::PrefixBigN => writer.write_str("N'")?,
            StringFormat::SingleQuote => writer.write_char('\'')?,
        }
        self.escape_string(writer, raw)?;
        writer.write_char('\'')?;
        Ok(())
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Native formats embed `bits` as-is. Callers must pass only `'0'` and `'1'`;
    /// that is checked with `debug_assert!`.
    #[inline]
    pub fn write_bits<W: std::fmt::Write>(&self, writer: &mut W, bits: &str) -> Result<(), FormatError> {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "write_bits: invalid input: {bits}"
        );
        match self.bit_format() {
            BitFormat::Unsupported => Err(FormatError::Unsupported("BitSerializer")),
            BitFormat::PrefixBigB => write!(writer, "B'{bits}'").map_err(Into::into),
            BitFormat::PrefixSmallB => write!(writer, "b'{bits}'").map_err(Into::into),
            BitFormat::PostfixBigB => write!(writer, "'{bits}'B").map_err(Into::into),
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write_bytes<W: std::fmt::Write>(&self, writer: &mut W, bytes: &[u8]) -> Result<(), FormatError> {
        let result = match self.bytes_format() {
            BytesFormat::PrefixBigX => {
                writer.write_str("X'")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_char('\'')
            }
            BytesFormat::PrefixZeroX => {
                writer.write_str("0x")?;
                bytes_to_hex(writer, bytes)
            }
            BytesFormat::SlashXBytea => {
                writer.write_str("'\\x")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'::bytea")
            }
            BytesFormat::PrefixBigBSlashX => {
                writer.write_str("B'")?;
                write_bytes_slash_x(writer, bytes)?;
                writer.write_char('\'')
            }
            BytesFormat::HexToRaw => {
                writer.write_str("HEXTORAW('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            BytesFormat::Unhex => {
                writer.write_str("UNHEX('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            BytesFormat::ToVarbyte => {
                writer.write_str("TO_VARBYTE('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'hex')")
            }
            BytesFormat::PostfixXbv => {
                writer.write_char('\'')?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'XBV")
            }
            BytesFormat::ToBinary => {
                writer.write_str("TO_BINARY('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'HEX')")
            }
            BytesFormat::SlashXBlob => {
                if bytes.is_empty() {
                    writer.write_str("''::BLOB")
                } else {
                    writer.write_char('\'')?;
                    write_bytes_slash_x(writer, bytes)?;
                    writer.write_str("'::BLOB")
                }
            }
        };
        result.map_err(Into::into)
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
fn write_hex2<W: std::fmt::Write>(writer: &mut W, byte: u8) -> std::fmt::Result {
    let buf = [HEX_DIGITS[(byte >> 4) as usize], HEX_DIGITS[(byte & 0x0F) as usize]];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
///
/// 64 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn bytes_to_hex<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(64) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 2;
            buf[o] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 1] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 2;
        // SAFETY: `buf[..n]` is filled only from `HEX_DIGITS` (ASCII).
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) })?;
    }
    Ok(())
}

/// Write each byte as `\xNN` (uppercase hex).
///
/// 32 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn write_bytes_slash_x<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(32) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 4;
            buf[o] = b'\\';
            buf[o + 1] = b'x';
            buf[o + 2] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 3] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 4;
        // SAFETY: `buf[..n]` is ASCII (`\`, `x`, and hex digits).
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) })?;
    }
    Ok(())
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
fn write_byte_hex_escape<W: std::fmt::Write>(writer: &mut W, byte: u8) -> std::fmt::Result {
    writer.write_str("\\x")?;
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
fn write_unicode4_escape<W: std::fmt::Write>(writer: &mut W, code: u32) -> std::fmt::Result {
    debug_assert!(code <= 0xFFFF);
    writer.write_str("\\u")?;
    write_hex2(writer, (code >> 8) as u8)?;
    write_hex2(writer, code as u8)
}

/// Use the string escape table, plus `\``.
///
/// `'` is left as itself. Only known escapes are emitted.
fn write_identifier_backtick_escape<W: std::fmt::Write>(writer: &mut W, ident: &str) -> std::fmt::Result {
    write_span_escapes(writer, ident, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'`' => SpanEscape::Literal("\\`"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}

/// Write `bytes[from..to]` as a string.
///
/// # Safety invariant
/// `bytes` is the raw view of a `&str`. Callers split only on ASCII bytes,
/// which are char boundaries, or at `bytes.len()`.
#[inline]
fn write_byte_span<W: std::fmt::Write>(writer: &mut W, bytes: &[u8], from: usize, to: usize) -> std::fmt::Result {
    if from == to {
        return Ok(());
    }
    // SAFETY: see invariant above.
    let chunk = unsafe { bytes.get_unchecked(from..to) };
    writer.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })
}

enum SpanEscape {
    Keep,
    Literal(&'static str),
    HexByte,
    Unicode4,
}

/// Copy each plain span with one `write_str`. Special bytes are ASCII.
#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_span_escapes<W: std::fmt::Write>(
    writer: &mut W,
    string: &str,
    mut classify: impl FnMut(u8) -> SpanEscape,
) -> std::fmt::Result {
    let bytes = string.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match classify(bytes[i]) {
            SpanEscape::Keep => i += 1,
            SpanEscape::Literal(repl) => {
                write_byte_span(writer, bytes, start, i)?;
                writer.write_str(repl)?;
                i += 1;
                start = i;
            }
            SpanEscape::HexByte => {
                write_byte_span(writer, bytes, start, i)?;
                write_byte_hex_escape(writer, bytes[i])?;
                i += 1;
                start = i;
            }
            SpanEscape::Unicode4 => {
                write_byte_span(writer, bytes, start, i)?;
                write_unicode4_escape(writer, bytes[i] as u32)?;
                i += 1;
                start = i;
            }
        }
    }
    write_byte_span(writer, bytes, start, bytes.len())
}

fn write_backslash_hex<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_mysql<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'"' => SpanEscape::Literal("\\\""),
        0 => SpanEscape::Literal("\\0"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        0x1a => SpanEscape::Literal("\\Z"),
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_unicode<W: std::fmt::Write>(
    writer: &mut W,
    string: &str,
    escape_double_quote: bool,
) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'"' if escape_double_quote => SpanEscape::Literal("\\\""),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        0 => SpanEscape::Literal("\\0"),
        b if b < 32 || b == 127 => SpanEscape::Unicode4,
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_hex_zero<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        0 => SpanEscape::Literal("\\0"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}
