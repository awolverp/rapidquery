"""SQL dialects and the flags that configure one."""

from typing import Final, Self, final

@final
class AggregateConfig:
    """Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` spelling.

    Controls how null-treatment modifiers and aggregate filters are written
    inside function calls such as ``FIRST_VALUE``.
    """

    def __new__(
        cls,
        *,
        null_treatment: int = 0,
        function_filter: int = 0,
    ) -> Self:
        """Create an ``AggregateConfig``.

        Args:
            null_treatment: ``OPT_NULL_TREATMENT_*`` value selecting where
                ``IGNORE NULLS`` / ``RESPECT NULLS`` is written. ``0`` rejects it.
            function_filter: ``OPT_FUNCTION_FILTER_*`` value selecting the
                ``FILTER`` syntax. ``0`` rejects it.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is an ``AggregateConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def null_treatment(self) -> int:
        """``OPT_NULL_TREATMENT_*`` value: ``UNSUPPORTED``, ``INSIDE_PARENS``, or ``POSTFIX``.

        ``INSIDE_PARENS`` writes ``FIRST_VALUE(x IGNORE NULLS)``.
        ``POSTFIX`` writes ``FIRST_VALUE(x) IGNORE NULLS``.
        """

    @property
    def function_filter(self) -> int:
        """``OPT_FUNCTION_FILTER_*`` value describing the aggregate ``FILTER`` syntax.

        ``FILTER`` writes ``f(x) FILTER (cond)``.
        ``FILTER_WHERE`` writes ``f(x) FILTER (WHERE cond)``.
        ``INSIDE_WHERE`` writes ``f(x WHERE cond)``.
        """

@final
class BitwiseConfig:
    """Which bitwise operators exist, and how ``& | ~ << >>`` and ``^`` are spelled.

    A bitwise operator is rejected unless its flag is set in ``operations``.
    """

    def __new__(
        cls,
        *,
        operations: int = 0,
        syntax: int = 0,
        xor: int = 0,
    ) -> Self:
        """Create a ``BitwiseConfig``.

        Args:
            operations: ``OPT_OPERATIONS_*`` flags for the operators the dialect supports.
            syntax: ``OPT_SYNTAX_*`` value shared by ``&``, ``|``, ``~``, ``<<``, and ``>>``.
            xor: ``OPT_XOR_*`` value selecting how ``^`` is written.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``BitwiseConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def operations(self) -> int:
        """``OPT_OPERATIONS_*`` flags for the supported operators: AND, OR, NOT, SHL, SHR."""

    @property
    def syntax(self) -> int:
        """``OPT_SYNTAX_*`` value for the spelling shared by AND, OR, NOT, and shifts.

        ``INFIX`` writes ``& | ~ << >>``.
        ``BIT_FUNCTION`` writes ``BITAND`` / ``BITOR`` / ``BITNOT`` / ``BITSHIFTLEFT`` / ``BITSHIFTRIGHT``.
        ``SHIFT_FUNCTION`` writes ``BITAND`` / ``BITOR`` / ``BITNOT`` / ``SHIFTLEFT`` / ``SHIFTRIGHT``.
        """

    @property
    def xor(self) -> int:
        """``OPT_XOR_*`` value for how ``^`` (XOR) is spelled.

        ``CARET`` writes ``left ^ right``.
        ``HASH`` writes ``left # right``. In PostgreSQL, ``^`` is exponentiation.
        ``FUNCTION`` writes ``xor(left, right)``.
        ``BIT_FUNCTION`` writes ``BITXOR(left, right)``.
        """

@final
class CastConfig:
    """``TRY_CAST`` spelling, and whether ``expr::type`` is legal."""

    def __new__(
        cls,
        *,
        try_cast: str | None = "TRY_CAST({expr} AS {type})",
        supports_double_colon: bool = False,
    ) -> Self:
        """Create a ``CastConfig``.

        Args:
            try_cast: Template for ``TRY_CAST`` with ``{expr}`` and ``{type}`` keys,
                or ``None`` if the dialect has no ``TRY_CAST``.
            supports_double_colon: Whether ``expr::type`` is valid syntax.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``CastConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def try_cast(self) -> str | None:
        """Template for ``TRY_CAST``, using ``{expr}`` and ``{type}`` keys, or ``None`` if unsupported."""

    @property
    def supports_double_colon(self) -> bool:
        """``True`` if the dialect accepts the ``expr::type`` cast syntax."""

@final
class LikeConfig:
    """``LIKE`` / ``ILIKE`` / ``SIMILAR TO`` / regex spelling and escape support."""

    def __new__(
        cls,
        *,
        escape: int = 0,
        any: int = 0,
        ilike: int = 0,
        regex: int = 0,
        supports_similar_to: bool = False,
    ) -> Self:
        """Create a ``LikeConfig``.

        Args:
            escape: ``OPT_ESCAPE_*`` flags for where an ``ESCAPE`` clause is legal.
                ``0`` writes no ``ESCAPE`` and rejects an escape character.
            any: ``OPT_ANY_*`` value selecting how ``LIKE ANY`` / ``ILIKE ANY`` is rendered.
            ilike: ``OPT_ILIKE_*`` value selecting how ``ILIKE`` is rendered.
            regex: ``OPT_REGEX_*`` value selecting how regex matching is spelled.
            supports_similar_to: Whether the dialect supports ``SIMILAR TO``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``LikeConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def escape(self) -> int:
        """``OPT_ESCAPE_*`` flags for where an ``ESCAPE`` clause is allowed.

        ``SCALAR`` allows a scalar ``LIKE`` pattern with ``ESCAPE``.
        ``ANY`` allows native ``LIKE ANY (...) ESCAPE``.
        """

    @property
    def any(self) -> int:
        """``OPT_ANY_*`` value selecting the ``LIKE ANY`` rendering.

        ``OR_EXPANSION`` writes ``(x LIKE p1 OR x LIKE p2)``.
        ``ARRAY_CONSTRUCTOR`` writes ``LIKE ANY (ARRAY[p, ...])``.
        ``PATTERN_LIST`` writes ``LIKE ANY (p, ...)``.
        """

    @property
    def ilike(self) -> int:
        """``OPT_ILIKE_*`` value selecting how ``ILIKE`` is rendered.

        ``STANDARD`` uses native ``ILIKE``.
        ``USE_LIKE_LOWER`` writes ``LIKE`` with ``LOWER()`` on both sides.
        ``UNSUPPORTED`` rejects ``ILIKE``.
        """

    @property
    def regex(self) -> int:
        """``OPT_REGEX_*`` value selecting how regex matching is spelled.

        ``STANDARD`` writes ``expr [NOT] RLIKE pattern``.
        ``TILDE`` writes ``expr ~ pattern`` / ``expr !~ pattern``.
        ``REGEXP_LIKE``, ``REGEXP_CONTAINS``, and ``REGEXP_MATCHES`` write the matching function call.
        ``UNSUPPORTED`` rejects regex matching.
        """

    @property
    def supports_similar_to(self) -> bool:
        """``True`` if the dialect supports the ``SIMILAR TO`` predicate."""

@final
class LiteralConfig:
    """Boolean, bit, bytes, network, UUID, JSON, and array literals.

    ``bit``, ``inet``, ``macaddr``, and ``array`` are ``None`` when the dialect
    has no spelling. ``json`` receives an already quoted string.
    """

    def __new__(
        cls,
        *,
        boolean: int = 0,
        bit: str | None = None,
        bytes: str = "X'{hex}'",
        inet: str | None = None,
        macaddr: str | None = None,
        uuid: str | None = None,
        json: str = "{}",
        array: str | None = None,
    ) -> Self:
        """Create a ``LiteralConfig``.

        Args:
            boolean: ``OPT_BOOLEAN_*`` value. ``KEYWORD`` writes ``TRUE`` / ``FALSE``,
                and ``INTEGER`` writes ``1`` / ``0``.
            bit: Template for bit-string literals, such as ``B'{}'``, or ``None`` if unsupported.
            bytes: Template for byte-string literals, using the ``{hex}`` key (e.g. ``X'{hex}'``).
            inet: Template for inet literals, or ``None`` if the dialect has no spelling.
            macaddr: Template for MAC address literals, or ``None`` if the dialect has no spelling.
            uuid: Template for UUID literals, or ``None`` if the dialect has no spelling.
            json: Template for JSON literals. The value already includes its quotes.
            array: Template for array literals, such as ``ARRAY[{}]``, or ``None`` if unsupported.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``LiteralConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def boolean(self) -> int:
        """``OPT_BOOLEAN_*`` value: ``KEYWORD`` for ``TRUE``/``FALSE``, or ``INTEGER`` for ``1``/``0``."""

    @property
    def bit(self) -> str | None:
        """Template for bit-string literals, or ``None`` if the dialect has no bit literal."""

    @property
    def bytes(self) -> str:
        """Template for byte-string literals, using the ``{hex}`` key (e.g. ``X'{hex}'``)."""

    @property
    def inet(self) -> str | None:
        """Template for inet literals, or ``None`` if unsupported."""

    @property
    def macaddr(self) -> str | None:
        """Template for MAC address literals, or ``None`` if unsupported."""

    @property
    def uuid(self) -> str | None:
        """Template for UUID literals, or ``None`` if unsupported."""

    @property
    def json(self) -> str:
        """Template for JSON literals. The value passed in already includes its quotes."""

    @property
    def array(self) -> str | None:
        """Template for array literals (e.g. ``ARRAY[{}]``), or ``None`` if unsupported."""

@final
class OperatorConfig:
    """``%``, ``IS DISTINCT FROM``, ``TRIM``, and ``COLLATE`` spelling."""

    def __new__(
        cls,
        *,
        modulo: int = 0,
        is_distinct_from: int = 0,
        trim: int = 0,
        collate: int = 0,
    ) -> Self:
        """Create an ``OperatorConfig``.

        Args:
            modulo: ``OPT_MODULO_*`` value selecting how modulo is written.
            is_distinct_from: ``OPT_IS_DISTINCT_FROM_*`` value selecting how
                ``IS [NOT] DISTINCT FROM`` is written.
            trim: ``OPT_TRIM_*`` value selecting how ``TRIM`` is written.
            collate: ``OPT_COLLATE_*`` value selecting how ``COLLATE`` is written.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is an ``OperatorConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def modulo(self) -> int:
        """``OPT_MODULO_*`` value for modulo.

        ``PERCENT`` writes ``left % right``.
        ``FUNCTION`` writes ``MOD(left, right)``.
        ``INFIX`` writes ``left MOD right``.
        """

    @property
    def is_distinct_from(self) -> int:
        """``OPT_IS_DISTINCT_FROM_*`` value for ``IS [NOT] DISTINCT FROM``.

        ``STANDARD`` writes ``left IS [NOT] DISTINCT FROM right``.
        ``NULL_SAFE_EQUAL`` uses ``<=>`` for null-safe equality. ``IS DISTINCT FROM`` becomes ``NOT (left <=> right)``.
        ``UNSUPPORTED`` rejects the operator.
        """

    @property
    def trim(self) -> int:
        """``OPT_TRIM_*`` value for how ``TRIM`` is spelled.

        ``SQL_FROM`` writes ``TRIM([LEADING|TRAILING] [chars FROM] expr)``.
        ``FUNCTION`` writes ``TRIM`` / ``LTRIM`` / ``RTRIM`` ``(expr[, chars])``.
        ``SPACES_ONLY`` writes ``TRIM`` / ``LTRIM`` / ``RTRIM`` ``(expr)``. Supplying a character set raises an error.
        """

    @property
    def collate(self) -> int:
        """``OPT_COLLATE_*`` value for how ``COLLATE`` is spelled.

        ``INFIX_IDENT`` writes ``expr COLLATE ident``.
        ``FUNCTION_STRING`` writes ``COLLATE(expr, 'body')``, where the collation name is a string literal.
        ``UNSUPPORTED`` rejects ``COLLATE``.
        """

@final
class SchemaConfig:
    """``CREATE`` / ``DROP SCHEMA`` features, owner clause, and location clause.

    ``features`` must include ``OPT_FEATURES_SUPPORTED`` or schema statements
    are rejected. ``location`` receives an already quoted string and supplies
    its own leading space (``' LOCATION {}'``). ``None`` rejects a location.
    """

    def __new__(
        cls,
        *,
        features: int = 0,
        authorization: int = 0,
        location: str | None = None,
    ) -> Self:
        """Create a ``SchemaConfig``.

        Args:
            features: ``OPT_FEATURES_*`` flags. Must include ``OPT_FEATURES_SUPPORTED``
                for any schema statement to be accepted.
            authorization: ``OPT_AUTHORIZATION_*`` value for the ``AUTHORIZATION`` owner clause.
            location: Template for the ``LOCATION`` clause, including its leading space
                (e.g. ``' LOCATION {}'``), or ``None`` to reject a location.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``SchemaConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def features(self) -> int:
        """``OPT_FEATURES_*`` flags describing schema support.

        ``SUPPORTED`` means the dialect supports schemas. Other flags cover ``IF NOT EXISTS``,
        embedded DDL in ``CREATE SCHEMA``, whether a schema is a database, and ``CASCADE`` / ``RESTRICT`` on ``DROP SCHEMA``.
        """

    @property
    def authorization(self) -> int:
        """``OPT_AUTHORIZATION_*`` value for the ``AUTHORIZATION`` owner clause.

        ``UNSUPPORTED`` means the clause is not allowed.
        ``OPTIONAL`` means the clause is allowed but not required.
        """

    @property
    def location(self) -> str | None:
        """Template for the ``LOCATION`` clause, including its leading space, or ``None`` if unsupported."""

@final
class StringConfig:
    """String literal template and ``||`` / ``+`` / ``CONCAT`` spelling.

    The template key selects the escape: ``{single}``, ``{double}``, ``{hex}``,
    ``{hex_quote}``, ``{mysql}``, ``{unicode}``, or ``{unicode_quote}``.
    ``concat`` keys are ``{left}`` and ``{right}``.
    """

    def __new__(
        cls,
        *,
        template: str = "'{single}'",
        concat: str = "{left} || {right}",
    ) -> Self:
        """Create a ``StringConfig``.

        Args:
            template: String literal template with one escape key, such as ``'{single}'``
                or ``E'{hex}'``. A string containing NUL is rejected.
            concat: Template for concatenation with ``{left}`` and ``{right}`` keys.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``StringConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def template(self) -> str:
        """String literal template, containing one escape key such as ``{single}`` or ``{hex}``."""

    @property
    def concat(self) -> str:
        """Concatenation template using ``{left}`` and ``{right}`` keys (e.g. ``{left} || {right}``)."""

@final
class TemporalConfig:
    """Date, time, and timestamp literal templates.

    ``{}`` is the raw text (``2024-01-15``), so the quotes belong in the
    template (``DATE '{}'``, ``DATE E'{}'``).
    """

    def __new__(
        cls,
        *,
        date: str = "{}",
        time: str = "{}",
        datetime_naive: str = "{}",
        datetime_aware: str = "{}",
    ) -> Self:
        """Create a ``TemporalConfig``.

        Args:
            date: Template for date literals, such as ``DATE '{}'``.
            time: Template for time literals, such as ``TIME '{}'``.
            datetime_naive: Template for timestamps without a timezone, such as ``TIMESTAMP '{}'``.
            datetime_aware: Template for timestamps with a timezone, such as ``TIMESTAMPTZ '{}'``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``TemporalConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def date(self) -> str:
        """Template for date literals. ``{}`` is replaced by the raw date text."""

    @property
    def time(self) -> str:
        """Template for time literals. ``{}`` is replaced by the raw time text."""

    @property
    def datetime_naive(self) -> str:
        """Template for timestamps without a timezone. ``{}`` is replaced by the raw text."""

    @property
    def datetime_aware(self) -> str:
        """Template for timestamps with a timezone. ``{}`` is replaced by the raw text."""

@final
class TupleConfig:
    """Tuple / row-value spelling and which arities below 2 are legal."""

    def __new__(
        cls,
        *,
        syntax: int = 0,
        arity: int = 0,
    ) -> Self:
        """Create a ``TupleConfig``.

        Args:
            syntax: ``OPT_SYNTAX_*`` value selecting how a tuple is written.
            arity: ``OPT_ARITY_*`` flags for tuple sizes below 2 that are allowed.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``TupleConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def syntax(self) -> int:
        """``OPT_SYNTAX_*`` value for how a tuple is written.

        ``ROW`` writes ``ROW(...)``.
        ``STRUCT`` writes ``struct(...)``.
        ``PARENTHESES`` writes ``(a, b, ...)``.
        ``UNSUPPORTED`` rejects tuples.
        """

    @property
    def arity(self) -> int:
        """``OPT_ARITY_*`` flags for tuple sizes below 2 that are allowed.

        Sizes 2 and above are always allowed when ``syntax`` is supported.
        ``OPT_ARITY_EMPTY`` allows size 0. ``OPT_ARITY_ONE`` allows size 1.
        """

@final
class Dialect:
    """Quoting, literal spelling, and which syntax is legal.

    ``placeholder`` may be a bare marker (``'?'``) or include the 1-based
    index (``'?{}'``, ``'${}'``, ``':{}'``). ``identifier`` keys are
    ``{double}``, ``{single}``, ``{bracket}``, ``{backtick}``, and the
    ``slash*`` backslash forms.
    """

    def __new__(
        cls,
        *,
        placeholder: str = "?{}",
        like: LikeConfig = ...,
        bitwise: BitwiseConfig = ...,
        identifier: str = '"{double}"',
        string: StringConfig = ...,
        cast: CastConfig = ...,
        tuple: TupleConfig = ...,
        temporal: TemporalConfig = ...,
        literal: LiteralConfig = ...,
        operator: OperatorConfig = ...,
        aggregate: AggregateConfig = ...,
        schema: SchemaConfig = ...,
    ) -> Self:
        """Create a ``Dialect``.

        Args:
            placeholder: Positional parameter template. Use ``'?'`` for a bare marker,
                or include ``{}`` for the 1-based index (e.g. ``'${}'`` gives ``$1``, ``$2``).
            like: ``LIKE`` / ``ILIKE`` / regex configuration.
            bitwise: Bitwise operator support and spelling.
            identifier: Identifier template such as ``"{double}"`` or ``` `{backtick}` ```.
                The key must match the delimiter in the template.
            string: String literal and concatenation configuration.
            cast: Cast syntax configuration (``TRY_CAST``, ``::``).
            tuple: Tuple and row-value configuration.
            temporal: Date, time, and timestamp literal templates.
            literal: Boolean, bit, bytes, network, UUID, JSON, and array literal configuration.
            operator: Spelling of modulo, ``IS DISTINCT FROM``, ``TRIM``, and ``COLLATE``.
            aggregate: Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` configuration.
            schema: Schema and DDL feature configuration.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``Dialect`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def placeholder(self) -> str:
        """Positional parameter template, such as ``'?{}'``, ``'?'``, or ``'${}'``."""

    @property
    def like(self) -> LikeConfig:
        """``LIKE`` / ``ILIKE`` / regex configuration."""

    @property
    def bitwise(self) -> BitwiseConfig:
        """Bitwise operator support and spelling."""

    @property
    def identifier(self) -> str:
        """Identifier quoting template, such as ``"{double}"`` or ``` `{backtick}` ```."""

    @property
    def string(self) -> StringConfig:
        """String literal and concatenation configuration."""

    @property
    def cast(self) -> CastConfig:
        """Cast syntax configuration (``TRY_CAST`` and ``::``)."""

    @property
    def tuple(self) -> TupleConfig:
        """Tuple and row-value configuration."""

    @property
    def temporal(self) -> TemporalConfig:
        """Date, time, and timestamp literal templates."""

    @property
    def literal(self) -> LiteralConfig:
        """Boolean, bit, bytes, network, UUID, JSON, and array literal configuration."""

    @property
    def operator(self) -> OperatorConfig:
        """Spelling of modulo, ``IS DISTINCT FROM``, ``TRIM``, and ``COLLATE``."""

    @property
    def aggregate(self) -> AggregateConfig:
        """Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` configuration."""

    @property
    def schema(self) -> SchemaConfig:
        """Schema and DDL feature configuration."""

# ---------------------------------------------------------------------------
# Feature flags. Combine with ``|`` and pass to the matching ``*Config``.
# A flag that is not set means the dialect does not support that form.
# ---------------------------------------------------------------------------

# AggregateConfig.function_filter: where the aggregate FILTER clause is written.
OPT_FUNCTION_FILTER_UNSUPPORTED: Final[int]  # Aggregate FILTER is not supported.
OPT_FUNCTION_FILTER_FILTER: Final[int]  # f(x) FILTER (cond)
OPT_FUNCTION_FILTER_FILTER_WHERE: Final[int]  # f(x) FILTER (WHERE cond)
OPT_FUNCTION_FILTER_INSIDE_WHERE: Final[int]  # f(x WHERE cond)

# AggregateConfig.null_treatment: where IGNORE NULLS / RESPECT NULLS is written.
OPT_NULL_TREATMENT_UNSUPPORTED: Final[int]  # IGNORE/RESPECT NULLS is not supported.
OPT_NULL_TREATMENT_INSIDE_PARENS: Final[int]  # FIRST_VALUE(x IGNORE NULLS)
OPT_NULL_TREATMENT_POSTFIX: Final[int]  # FIRST_VALUE(x) IGNORE NULLS

# BitwiseConfig.operations: which bitwise operators are available.
OPT_OPERATIONS_AND: Final[int]  # &
OPT_OPERATIONS_OR: Final[int]  # |
OPT_OPERATIONS_NOT: Final[int]  # ~
OPT_OPERATIONS_SHL: Final[int]  # <<
OPT_OPERATIONS_SHR: Final[int]  # >>

# BitwiseConfig.syntax: spelling shared by AND, OR, NOT, and the shifts.
OPT_SYNTAX_INFIX: Final[int]  # & | ~ << >>
OPT_SYNTAX_BIT_FUNCTION: Final[int]  # BITAND / BITOR / BITNOT / BITSHIFTLEFT / BITSHIFTRIGHT
OPT_SYNTAX_SHIFT_FUNCTION: Final[int]  # BITAND / BITOR / BITNOT / SHIFTLEFT / SHIFTRIGHT

# BitwiseConfig.xor: spelling of ^.
OPT_XOR_UNSUPPORTED: Final[int]  # XOR is not supported.
OPT_XOR_CARET: Final[int]  # left ^ right
OPT_XOR_HASH: Final[int]  # left # right
OPT_XOR_FUNCTION: Final[int]  # xor(left, right)
OPT_XOR_BIT_FUNCTION: Final[int]  # BITXOR(left, right)

# LikeConfig.escape: where an ESCAPE clause is allowed.
OPT_ESCAPE_SCALAR: Final[int]  # Scalar LIKE ... ESCAPE, including OR expansion.
OPT_ESCAPE_ANY: Final[int]  # Native LIKE ANY (...) ESCAPE.

# LikeConfig.any: rendering of LIKE ANY / ILIKE ANY.
OPT_ANY_OR_EXPANSION: Final[int]  # (x LIKE p1 OR x LIKE p2)
OPT_ANY_ARRAY_CONSTRUCTOR: Final[int]  # LIKE ANY (ARRAY[p, ...])
OPT_ANY_PATTERN_LIST: Final[int]  # LIKE ANY (p, ...)

# LikeConfig.ilike: rendering of ILIKE.
OPT_ILIKE_UNSUPPORTED: Final[int]  # ILIKE is not supported.
OPT_ILIKE_STANDARD: Final[int]  # Native ILIKE.
OPT_ILIKE_USE_LIKE_LOWER: Final[int]  # LIKE with LOWER() on both sides.

# LikeConfig.regex: spelling of regex matching.
OPT_REGEX_UNSUPPORTED: Final[int]  # Regex matching is not supported.
OPT_REGEX_STANDARD: Final[int]  # expr [NOT] RLIKE pattern
OPT_REGEX_TILDE: Final[int]  # expr ~ pattern / expr !~ pattern
OPT_REGEX_REGEXP_LIKE: Final[int]  # REGEXP_LIKE(expr, pattern)
OPT_REGEX_REGEXP_CONTAINS: Final[int]  # REGEXP_CONTAINS(expr, pattern)
OPT_REGEX_REGEXP_MATCHES: Final[int]  # regexp_matches(expr, pattern)

# OperatorConfig.is_distinct_from: spelling of IS [NOT] DISTINCT FROM.
OPT_IS_DISTINCT_FROM_UNSUPPORTED: Final[int]  # Not supported.
OPT_IS_DISTINCT_FROM_STANDARD: Final[int]  # left IS [NOT] DISTINCT FROM right
OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL: Final[int]  # Uses <=>, where IS DISTINCT FROM is NOT (left <=> right).

# OperatorConfig.modulo: spelling of %.
OPT_MODULO_PERCENT: Final[int]  # left % right
OPT_MODULO_FUNCTION: Final[int]  # MOD(left, right)
OPT_MODULO_INFIX: Final[int]  # left MOD right

# OperatorConfig.trim: spelling of TRIM.
OPT_TRIM_SQL_FROM: Final[int]  # TRIM([LEADING|TRAILING] [chars FROM] expr)
OPT_TRIM_FUNCTION: Final[int]  # TRIM / LTRIM / RTRIM (expr[, chars])
OPT_TRIM_SPACES_ONLY: Final[int]  # TRIM / LTRIM / RTRIM (expr); a character set raises.

# OperatorConfig.collate: spelling of COLLATE.
OPT_COLLATE_UNSUPPORTED: Final[int]  # Not supported.
OPT_COLLATE_INFIX_IDENT: Final[int]  # expr COLLATE ident
OPT_COLLATE_FUNCTION_STRING: Final[int]  # COLLATE(expr, 'body')

# TupleConfig.arity: tuple sizes below 2 that are allowed.
OPT_ARITY_EMPTY: Final[int]  # Size 0.
OPT_ARITY_ONE: Final[int]  # Size 1.

# TupleConfig.syntax: how a tuple is written.
OPT_SYNTAX_ROW: Final[int]  # ROW(...)
OPT_SYNTAX_STRUCT: Final[int]  # struct(...)
OPT_SYNTAX_PARENTHESES: Final[int]  # (a, b, ...)
OPT_SYNTAX_UNSUPPORTED: Final[int]  # Tuples are not supported.

# LiteralConfig.boolean: spelling of boolean literals.
OPT_BOOLEAN_KEYWORD: Final[int]  # TRUE / FALSE
OPT_BOOLEAN_INTEGER: Final[int]  # 1 / 0

# SchemaConfig.features: schema statement support.
OPT_FEATURES_SUPPORTED: Final[int]  # Schemas are supported. Required for any schema statement.
OPT_FEATURES_IF_NOT_EXISTS: Final[int]  # CREATE SCHEMA IF NOT EXISTS
OPT_FEATURES_EMBEDDED_DDL: Final[int]  # DDL statements embedded in CREATE SCHEMA
OPT_FEATURES_IS_DATABASE: Final[int]  # A schema means the same thing as a database.
OPT_FEATURES_DROP_CASCADE: Final[int]  # DROP SCHEMA ... CASCADE
OPT_FEATURES_DROP_RESTRICT: Final[int]  # DROP SCHEMA ... RESTRICT

# SchemaConfig.authorization: AUTHORIZATION owner clause.
OPT_AUTHORIZATION_UNSUPPORTED: Final[int]  # Not allowed.
OPT_AUTHORIZATION_OPTIONAL: Final[int]  # Allowed, but not required.

BigQuery: Final[Dialect]
Databricks: Final[Dialect]
DuckDb: Final[Dialect]
Hive: Final[Dialect]
MsSql: Final[Dialect]
MySql: Final[Dialect]
Oracle: Final[Dialect]
PostgreSql: Final[Dialect]
Redshift: Final[Dialect]
Snowflake: Final[Dialect]
Spark: Final[Dialect]
Sqlite: Final[Dialect]
Teradata: Final[Dialect]
