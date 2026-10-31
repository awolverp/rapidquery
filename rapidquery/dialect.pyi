"""SQL dialects and the flags that configure one."""

from typing import Final, Self, final

@final
class AggregateConfig:
    """Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` spelling.

    Controls how null-treatment modifiers and aggregate filters are written
    inside function calls such as ``FIRST_VALUE``. Both options default to
    ``UNSUPPORTED``, which rejects the syntax.
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
                ``IGNORE NULLS`` / ``RESPECT NULLS`` is written. Defaults to
                ``UNSUPPORTED`` (``0``), which rejects it.
            function_filter: ``OPT_FUNCTION_FILTER_*`` value selecting the
                aggregate ``FILTER`` syntax. Defaults to ``UNSUPPORTED`` (``0``),
                which rejects it.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is an ``AggregateConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def null_treatment(self) -> int:
        """``OPT_NULL_TREATMENT_*`` value for ``{IGNORE|RESPECT} NULLS`` in functions.

        ``UNSUPPORTED`` rejects the syntax.
        ``INSIDE_PARENS`` writes ``FIRST_VALUE(x IGNORE NULLS)``.
        ``POSTFIX`` writes ``FIRST_VALUE(x) IGNORE NULLS``.
        """

    @property
    def function_filter(self) -> int:
        """``OPT_FUNCTION_FILTER_*`` value for the function ``FILTER`` syntax.

        ``UNSUPPORTED`` rejects the syntax.
        ``FILTER`` writes ``FIRST_VALUE(x) FILTER (id > 10)``.
        ``FILTER_WHERE`` writes ``FIRST_VALUE(x) FILTER (WHERE id > 10)``.
        ``INSIDE_WHERE`` writes ``FIRST_VALUE(x WHERE id > 10)``.
        """

@final
class BitwiseConfig:
    """Which bitwise operators exist, and how ``& | ~ << >>`` and ``^`` are spelled.

    A bitwise operator is rejected unless its flag is set in ``operations``.
    AND, OR, NOT, and the shifts share one spelling (``syntax``). XOR has its
    own spelling (``xor``).
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
            operations: ``OPT_BITWISE_OPERATIONS_*`` flags (``AND``, ``OR``, ``NOT``,
                ``SHL``, ``SHR``) for the operators the dialect supports. A missing
                flag rejects that operator. Defaults to none (``0``).
            syntax: ``OPT_BITWISE_SYNTAX_*`` value shared by ``&``, ``|``, ``~``,
                ``<<``, and ``>>``. Defaults to ``INFIX``.
            xor: ``OPT_BITWISE_XOR_*`` value selecting how ``^`` is written.
                Defaults to ``UNSUPPORTED`` (``0``), which rejects XOR.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``BitwiseConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def operations(self) -> int:
        """``OPT_BITWISE_OPERATIONS_*`` flags for the operators the dialect can render.

        Flags: ``AND``, ``OR``, ``NOT``, ``SHL``, ``SHR``. A missing flag rejects
        that operator. XOR is controlled separately by ``xor``.
        """

    @property
    def syntax(self) -> int:
        """``OPT_BITWISE_SYNTAX_*`` value for the spelling shared by AND, OR, NOT, and shifts.

        ``INFIX`` writes ``& | ~ << >>``.
        ``BIT_FUNCTION`` writes ``BITAND`` / ``BITOR`` / ``BITNOT`` / ``BITSHIFTLEFT`` / ``BITSHIFTRIGHT``.
        ``SHIFT_FUNCTION`` writes ``BITAND`` / ``BITOR`` / ``BITNOT`` / ``SHIFTLEFT`` / ``SHIFTRIGHT``.
        """

    @property
    def xor(self) -> int:
        """``OPT_BITWISE_XOR_*`` value for how ``^`` (XOR) is spelled.

        ``UNSUPPORTED`` rejects XOR.
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
            try_cast: Template for ``TRY_CAST`` with exactly the keys ``{expr}`` and
                ``{type}`` (e.g. ``TRY_CAST({expr} AS {type})``, the default), or
                ``None`` if the dialect has no ``TRY_CAST``.
            supports_double_colon: Whether the ``expr::type`` cast syntax is valid.
                Defaults to ``False``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``CastConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def try_cast(self) -> str | None:
        """Template for ``TRY_CAST``, using the ``{expr}`` and ``{type}`` keys, or ``None`` if unsupported."""

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
            escape: ``OPT_LIKE_ESCAPE_*`` flags for where an inlined ``LIKE ... ESCAPE``
                clause is legal. Empty (``0``, the default) writes no ``ESCAPE`` and
                rejects a set escape character. ``SCALAR | ANY`` allows both a scalar
                pattern and native ``LIKE ANY``.
            any: ``OPT_LIKE_ANY_*`` value selecting how ``LIKE ANY`` / ``ILIKE ANY``
                is rendered. Defaults to ``OR_EXPANSION``.
            ilike: ``OPT_I_LIKE_STRATEGY_*`` value selecting whether ``ILIKE`` is
                native or falls back to another form. Defaults to ``UNSUPPORTED``.
            regex: ``OPT_LIKE_REGEX_*`` value selecting how regex matching is spelled.
                Defaults to ``UNSUPPORTED``.
            supports_similar_to: Whether the dialect supports the ``SIMILAR TO``
                predicate. Defaults to ``False``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``LikeConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def escape(self) -> int:
        """``OPT_LIKE_ESCAPE_*`` flags for where an inlined ``ESCAPE`` clause is legal.

        Empty writes no ``ESCAPE`` and rejects a set escape character.
        ``SCALAR`` allows scalar ``LIKE`` and an ``OR`` expansion.
        ``ANY`` allows native ``LIKE ANY (p, ...) ESCAPE``.
        ``SCALAR | ANY`` allows both.
        """

    @property
    def any(self) -> int:
        """``OPT_LIKE_ANY_*`` value selecting the ``LIKE ANY`` / ``ILIKE ANY`` rendering.

        ``OR_EXPANSION`` writes ``(x LIKE p1 OR x LIKE p2)``. Useful when the
        dialect has no native support.
        ``ARRAY_CONSTRUCTOR`` writes ``LIKE ANY (ARRAY[p, ...])``.
        ``PATTERN_LIST`` writes ``LIKE ANY (p, ...)``.
        """

    @property
    def ilike(self) -> int:
        """``OPT_I_LIKE_STRATEGY_*`` value: does the dialect support ``ILIKE``, or should it fall back?

        ``UNSUPPORTED`` rejects ``ILIKE``.
        ``STANDARD`` uses native ``ILIKE``.
        ``USE_LIKE_LOWER`` writes ``LIKE`` with ``LOWER()`` on both sides.
        """

    @property
    def regex(self) -> int:
        """``OPT_LIKE_REGEX_*`` value selecting how regex matching (``RLIKE`` / ``~`` / ``REGEXP_LIKE``, ...) is spelled.

        ``UNSUPPORTED`` rejects regex matching.
        ``STANDARD`` writes ``expr [NOT] RLIKE pattern``.
        ``TILDE`` writes ``expr ~ pattern`` / ``expr !~ pattern``.
        ``REGEXP_LIKE`` writes ``REGEXP_LIKE(expr, pattern)``.
        ``REGEXP_CONTAINS`` writes ``REGEXP_CONTAINS(expr, pattern)``.
        ``REGEXP_MATCHES`` writes ``regexp_matches(expr, pattern)``.
        """

    @property
    def supports_similar_to(self) -> bool:
        """``True`` if the dialect supports the ``SIMILAR TO`` predicate."""

@final
class LiteralConfig:
    """Boolean, bit, bytes, network, UUID, JSON, and array literals.

    ``bit``, ``inet``, ``macaddr``, ``uuid``, and ``array`` are ``None`` when the
    dialect has no spelling. Every template other than ``bytes`` has exactly one
    ``{}`` placeholder. ``json`` receives a value that is already escaped and
    includes its own quotes.
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
        r"""Create a ``LiteralConfig``.

        Args:
            boolean: ``OPT_BOOL_FORMAT_*`` value. ``KEYWORD`` (default) writes
                ``TRUE`` / ``FALSE``, and ``INTEGER`` writes ``1`` / ``0``.
            bit: Template for bit-string literals, such as ``B'{}'``, or ``None``
                if unsupported. Prefer ``serialize_bits`` over using it directly.
            bytes: Template for byte-string literals, with exactly one key:
                ``{hex}`` (bytes as hex, e.g. ``DEADBEEF``) or ``{slashx}`` (each
                byte as ``\xNN``). Defaults to ``X'{hex}'``. Prefer
                ``serialize_bytes`` over using it directly.
            inet: Template for inet literals, such as ``'{}'::inet``, or ``None``
                if the dialect has no spelling.
            macaddr: Template for MAC address literals, such as ``'{}'::macaddr``,
                or ``None`` if the dialect has no spelling.
            uuid: Template for UUID literals, such as ``'{}'::uuid``, or ``None``
                if the dialect has no spelling.
            json: Template for JSON literals, such as ``JSON {}``. The value is
                escaped and already includes its quotes. Defaults to ``{}``.
            array: Template for array literals, such as ``ARRAY[{}]``, or ``None``
                if unsupported.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``LiteralConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def boolean(self) -> int:
        """``OPT_BOOL_FORMAT_*`` value: ``KEYWORD`` for ``TRUE``/``FALSE``, or ``INTEGER`` for ``1``/``0``."""

    @property
    def bit(self) -> str | None:
        """Template for bit-string literals (e.g. ``B'{}'``), or ``None`` if the dialect has no bit literal.

        Prefer ``serialize_bits`` over using this template directly.
        """

    @property
    def bytes(self) -> str:
        r"""Template for byte-string literals (e.g. ``X'{hex}'``).

        Template keys: ``{hex}`` is the bytes as hex (e.g. ``DEADBEEF``), and
        ``{slashx}`` is each byte as ``\xNN``. Prefer ``serialize_bytes`` over
        using this template directly.
        """

    @property
    def inet(self) -> str | None:
        """Template for inet literals (e.g. ``'{}'::inet``), or ``None`` if unsupported."""

    @property
    def macaddr(self) -> str | None:
        """Template for MAC address literals (e.g. ``'{}'::macaddr``), or ``None`` if unsupported."""

    @property
    def uuid(self) -> str | None:
        """Template for UUID literals (e.g. ``'{}'::uuid``), or ``None`` if unsupported."""

    @property
    def json(self) -> str:
        """Template for JSON literals (e.g. ``JSON {}``).

        The value passed in is already escaped and includes its own quotes.
        """

    @property
    def array(self) -> str | None:
        """Template for array literals (e.g. ``ARRAY[{}]``), or ``None`` if unsupported."""

    def serialize_boolean(self, value: bool, /) -> str:
        """Write a boolean literal as ``TRUE``/``FALSE`` or ``1``/``0``, according to ``boolean``."""

    def serialize_bits(self, value: str, /) -> str:
        """Write a bit-string (a string of ``'0'``/``'1'`` characters) as a native literal using ``bit``."""

    def serialize_bytes(self, value: bytes, /) -> str:
        """Write ``bytes`` as a native blob/bytea/varbyte literal using the ``bytes`` template."""

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
            modulo: ``OPT_OPERATOR_MODULO_*`` value selecting how modulo is
                written. Defaults to ``PERCENT``.
            is_distinct_from: ``OPT_IS_DISTINCT_FROM_*`` value selecting how
                ``IS [NOT] DISTINCT FROM`` is written. Defaults to ``UNSUPPORTED``.
            trim: ``OPT_OPERATOR_TRIM_*`` value selecting how ``TRIM`` is written.
                Defaults to ``SQL_FROM``.
            collate: ``OPT_OPERATOR_COLLATE_*`` value selecting how ``COLLATE`` is
                written. Defaults to ``UNSUPPORTED``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is an ``OperatorConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def modulo(self) -> int:
        """``OPT_OPERATOR_MODULO_*`` value for how binary modulo (``%``) is spelled.

        ``PERCENT`` writes ``left % right``.
        ``FUNCTION`` writes ``MOD(left, right)``.
        ``INFIX`` writes ``left MOD right``.
        """

    @property
    def is_distinct_from(self) -> int:
        """``OPT_IS_DISTINCT_FROM_*`` value for ``IS [NOT] DISTINCT FROM``.

        ``UNSUPPORTED`` rejects the operator.
        ``STANDARD`` writes ``left IS [NOT] DISTINCT FROM right``.
        ``NULL_SAFE_EQUAL`` uses ``<=>``, which is null-safe equality
        (``IS NOT DISTINCT FROM``). ``IS DISTINCT FROM`` is written as ``NOT (left <=> right)``.
        """

    @property
    def trim(self) -> int:
        """``OPT_OPERATOR_TRIM_*`` value for how ``TRIM`` is spelled.

        ``SQL_FROM`` writes ``TRIM([LEADING|TRAILING] [chars FROM] expr)``.
        ``FUNCTION`` writes ``TRIM`` / ``LTRIM`` / ``RTRIM`` ``(expr[, chars])``.
        ``SPACES_ONLY`` writes ``TRIM`` / ``LTRIM`` / ``RTRIM`` ``(expr)``. Supplying a character set raises an error.
        """

    @property
    def collate(self) -> int:
        """``OPT_OPERATOR_COLLATE_*`` value for how ``COLLATE`` is spelled.

        ``UNSUPPORTED`` rejects ``COLLATE``.
        ``INFIX_IDENT`` writes ``expr COLLATE ident``.
        ``FUNCTION_STRING`` writes ``COLLATE(expr, 'body')``, where the collation name is a string literal.
        """

@final
class SchemaConfig:
    """``CREATE`` / ``DROP SCHEMA`` features, owner clause, and location clause.

    ``features`` must include ``OPT_SCHEMA_FEATURES_SUPPORTED`` or schema
    statements are rejected. ``location`` receives a value that is already
    escaped and includes its own quotes; ``None`` rejects a location.
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
            features: ``OPT_SCHEMA_FEATURES_*`` flags describing schema support.
                Must include ``OPT_SCHEMA_FEATURES_SUPPORTED`` for any schema
                statement to be accepted. Defaults to none (``0``).
            authorization: ``OPT_SCHEMA_AUTHORIZATION_*`` value for the
                ``AUTHORIZATION`` owner clause. Defaults to ``UNSUPPORTED``.
            location: Template for the schema location clause with exactly one
                ``{}`` placeholder (e.g. ``LOCATION {}``), or ``None`` (default)
                to reject a location. The value is escaped and includes its own quotes.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``SchemaConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def features(self) -> int:
        """``OPT_SCHEMA_FEATURES_*`` flags for the independent schema features supported.

        ``SUPPORTED``: the dialect supports schemas.
        ``IF_NOT_EXISTS``: ``CREATE SCHEMA IF NOT EXISTS`` is supported.
        ``EMBEDDED_DDL``: DDL statements can be embedded in ``CREATE SCHEMA``.
        ``IS_DATABASE``: a schema has the same meaning as a database in this dialect.
        ``DROP_CASCADE``: ``DROP SCHEMA ... CASCADE`` is supported.
        ``DROP_RESTRICT``: ``DROP SCHEMA ... RESTRICT`` is supported.
        """

    @property
    def authorization(self) -> int:
        """``OPT_SCHEMA_AUTHORIZATION_*`` value: how the schema owner (``AUTHORIZATION``) is specified.

        ``UNSUPPORTED`` means the clause is not allowed.
        ``OPTIONAL`` means the clause is allowed but not required.
        """

    @property
    def location(self) -> str | None:
        """Template for how a schema location is expressed (e.g. ``LOCATION {}``), or ``None`` if unsupported.

        The value is escaped and includes its own quotes.
        """

@final
class StringConfig:
    r"""String literal template and ``||`` / ``+`` / ``CONCAT`` spelling.

    The template key selects how text is escaped: ``{single}``, ``{double}``,
    ``{hex}``, ``{hex_quote}``, ``{mysql}``, ``{unicode}``, or ``{unicode_quote}``.
    ``concat`` keys are ``{left}`` and ``{right}``.
    """

    def __new__(
        cls,
        *,
        template: str = "'{single}'",
        concat: str = "{left} || {right}",
    ) -> Self:
        r"""Create a ``StringConfig``.

        Args:
            template: String literal template with exactly one escape key, such as
                ``'{single}'`` (default) or ``E'{hex}'``. A string containing NUL
                is rejected. Pick a key that escapes the delimiter used in the
                template (``'{single}'`` or ``"{double}"``).

                Escape by doubling (other characters are unchanged):

                - ``single``: ``'`` becomes ``''`` (standard SQL, SQL Server, Oracle, SQLite, ...).
                - ``double``: ``"`` becomes ``""``.

                Escape with a backslash (``\`` is always escaped):

                - ``hex``: escapes ``\\``, ``\'``, ``\n``, ``\r``, ``\t``; other C0/DEL as ``\xNN``. ``"`` is unchanged.
                - ``hex_quote``: ``hex`` plus ``\"``. Use for ``"`` delimited strings.
                - ``mysql``: (MySQL-specific) escapes ``\\``, ``\'``, ``\"``, ``\n``, ``\r`` and ``\Z`` for 0x1A; tab and other controls pass through.
                - ``unicode``: like ``hex``, but other C0/DEL as ``\uNNNN``. ``"`` is unchanged.
                - ``unicode_quote``: ``unicode`` plus ``\"``. Use for ``"`` delimited strings.

                Only use a backslash key when the target database really reads
                backslash escapes inside string literals (e.g. MySQL, BigQuery,
                Hive, Spark, or PostgreSQL ``E'...'``).
            concat: Template for binary string concatenation with exactly the keys
                ``{left}`` and ``{right}``. Defaults to ``{left} || {right}``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``StringConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def template(self) -> str:
        """String literal template (e.g. ``E'{hex}'``), containing one escape key.

        The key (``{single}``, ``{double}``, ``{hex}``, ``{hex_quote}``, ``{mysql}``,
        ``{unicode}``, or ``{unicode_quote}``) picks how the text is escaped.
        A string containing NUL is rejected. Prefer ``serialize_string`` over using
        the template directly.
        """

    @property
    def concat(self) -> str:
        """Template for binary string concatenation (e.g. ``{left} || {right}``), using the ``{left}`` and ``{right}`` keys."""

    def serialize_string(self, value: str, /) -> str:
        """Escape ``value`` according to ``template`` and wrap it as a complete string literal."""

@final
class TemporalConfig:
    """Date, time, and timestamp literal templates.

    Each template has exactly one ``{}`` placeholder, which is the raw text
    (``2024-01-15``), so the quotes belong in the template (``DATE '{}'``,
    ``DATE E'{}'``). All default to ``{}``.
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
            datetime_naive: Template for naive timestamps (without a timezone),
                such as ``TIMESTAMP '{}'``.
            datetime_aware: Template for timezone-aware timestamps, such as
                ``TIMESTAMPTZ '{}'``.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``TemporalConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def date(self) -> str:
        """Template for date literals (e.g. ``DATE '{}'``). ``{}`` is replaced by the raw date text."""

    @property
    def time(self) -> str:
        """Template for time literals (e.g. ``TIME '{}'``). ``{}`` is replaced by the raw time text."""

    @property
    def datetime_naive(self) -> str:
        """Template for naive timestamps (e.g. ``TIMESTAMP '{}'``). ``{}`` is replaced by the raw text."""

    @property
    def datetime_aware(self) -> str:
        """Template for timezone-aware timestamps (e.g. ``TIMESTAMPTZ '{}'``). ``{}`` is replaced by the raw text."""

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
            syntax: ``OPT_TUPLE_SYNTAX_*`` value selecting how a tuple is written.
                Defaults to ``UNSUPPORTED``.
            arity: ``OPT_TUPLE_ARITY_*`` flags for tuple sizes below 2 that are
                allowed. Defaults to none (``0``), meaning a tuple needs at least
                two elements.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``TupleConfig`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def syntax(self) -> int:
        """``OPT_TUPLE_SYNTAX_*`` value for how a tuple / row value is written.

        ``UNSUPPORTED`` rejects tuples.
        ``ROW`` writes ``ROW(...)``.
        ``STRUCT`` writes ``struct(...)``.
        ``PARENTHESES`` writes ``(a, b, ...)``.

        Which arities are legal is controlled by ``arity``, not by another syntax value.
        """

    @property
    def arity(self) -> int:
        """``OPT_TUPLE_ARITY_*`` flags for tuple sizes below 2 that the syntax accepts.

        Sizes 2 and above are always allowed when ``syntax`` is not ``UNSUPPORTED``.
        Empty means a tuple needs at least two elements.
        ``OPT_TUPLE_ARITY_EMPTY`` allows size 0. ``OPT_TUPLE_ARITY_ONE`` allows size 1.
        """

@final
class Dialect:
    r"""Quoting, literal spelling, and which syntax is legal.

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
        r"""Create a ``Dialect``.

        Args:
            placeholder: Positional parameter template with at most one ``{}``,
                which is the 1-based index and may be omitted. ``'?'`` writes the
                marker alone; ``'${}'`` writes ``$1``, ``$2``, and so on.
                Defaults to ``'?{}'``.
            like: ``LIKE`` / ``ILIKE`` / regex configuration.
            bitwise: Bitwise operator support and spelling.
            identifier: Identifier template with exactly one key, such as
                ``"{double}"`` (default) or ``` `{backtick}` ```. The key must match
                the delimiter in the template and picks how the name is escaped.

                Escape by doubling (``x`` becomes ``xx``; other characters are unchanged):

                - ``double``: ``"`` (PostgreSQL, SQLite, Oracle, Snowflake, ...).
                - ``single``: ``'``.
                - ``bracket``: ``]`` (SQL Server).
                - ``backtick``: ``` ` ``` (MySQL, Hive, Spark, Databricks, ...).

                Escape with a backslash (``\`` and the delimiter get a backslash;
                ``\n``, ``\r``, ``\t`` are written as escapes; other control
                characters as ``\xNN``):

                - ``slashdouble``: ``"``.
                - ``slashsingle``: ``'``.
                - ``slashbracket``: ``]``.
                - ``slashbacktick``: ``` ` ``` (BigQuery).

                Only use a ``slash*`` key when the target database really reads
                backslash escapes inside quoted identifiers.
            string: String literal format, escaping, and concatenation.
            cast: Cast syntax configuration (``TRY_CAST``, ``::``).
            tuple: Tuple / row-value syntax and arity support.
            temporal: Date, time, and timestamp literal spelling.
            literal: Other literal formats (bool, bit, bytes, inet, MAC address,
                UUID, JSON, array).
            operator: Spelling of modulo, ``IS DISTINCT FROM``, ``TRIM``, and ``COLLATE``.
            aggregate: Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` syntax.
            schema: Schema / DDL feature flags and clause styles.

        Every config argument defaults to its class's default-constructed value.
        """

    def __eq__(self, value: object, /) -> bool:
        """Return ``True`` if ``value`` is a ``Dialect`` with equal settings."""

    def __hash__(self) -> int:
        """Return a hash consistent with ``__eq__``."""

    @property
    def placeholder(self) -> str:
        """Positional parameter template, such as ``'?{}'``, ``'?'``, or ``'${}'``.

        ``{}`` is the 1-based index and may be omitted; ``'?'`` writes the marker alone.
        """

    @property
    def like(self) -> LikeConfig:
        """``LIKE`` / ``ILIKE`` / regex configuration."""

    @property
    def bitwise(self) -> BitwiseConfig:
        """Bitwise operator support and spelling."""

    @property
    def identifier(self) -> str:
        """Identifier quoting template, such as ``"{double}"`` or ``` `{backtick}` ```.

        The key (``double``, ``single``, ``bracket``, ``backtick``, or a
        ``slash*`` variant) picks which character is escaped and how.
        """

    @property
    def string(self) -> StringConfig:
        """String literal format, escaping, and concatenation."""

    @property
    def cast(self) -> CastConfig:
        """Cast syntax configuration (``TRY_CAST`` and ``::``)."""

    @property
    def tuple(self) -> TupleConfig:
        """Tuple / row-value syntax and arity support."""

    @property
    def temporal(self) -> TemporalConfig:
        """Date, time, and timestamp literal spelling."""

    @property
    def literal(self) -> LiteralConfig:
        """Other literal formats: boolean, bit, bytes, inet, MAC address, UUID, JSON, and array."""

    @property
    def operator(self) -> OperatorConfig:
        """Miscellaneous operator spellings: modulo, ``IS DISTINCT FROM``, ``TRIM``, and ``COLLATE``."""

    @property
    def aggregate(self) -> AggregateConfig:
        """Aggregate ``{IGNORE|RESPECT} NULLS`` and ``FILTER`` syntax."""

    @property
    def schema(self) -> SchemaConfig:
        """Schema / DDL feature flags and clause styles."""

# AggregateConfig.function_filter: where the aggregate FILTER clause is written.
OPT_FUNCTION_FILTER_UNSUPPORTED: Final[int]  # Aggregate FILTER is not supported.
OPT_FUNCTION_FILTER_FILTER: Final[int]  # f(x) FILTER (cond)
OPT_FUNCTION_FILTER_FILTER_WHERE: Final[int]  # f(x) FILTER (WHERE cond)
OPT_FUNCTION_FILTER_INSIDE_WHERE: Final[int]  # f(x WHERE cond)

# AggregateConfig.null_treatment: where IGNORE NULLS / RESPECT NULLS is written.
OPT_NULL_TREATMENT_UNSUPPORTED: Final[int]  # IGNORE/RESPECT NULLS is not supported.
OPT_NULL_TREATMENT_INSIDE_PARENS: Final[int]  # FIRST_VALUE(x IGNORE NULLS)
OPT_NULL_TREATMENT_POSTFIX: Final[int]  # FIRST_VALUE(x) IGNORE NULLS

# BitwiseConfig.operations: flags for which bitwise operators are available. A missing flag rejects the operator.
OPT_BITWISE_OPERATIONS_AND: Final[int]  # &
OPT_BITWISE_OPERATIONS_OR: Final[int]  # |
OPT_BITWISE_OPERATIONS_NOT: Final[int]  # ~
OPT_BITWISE_OPERATIONS_SHL: Final[int]  # <<
OPT_BITWISE_OPERATIONS_SHR: Final[int]  # >>

# BitwiseConfig.syntax: spelling shared by AND, OR, NOT, and the shifts.
OPT_BITWISE_SYNTAX_INFIX: Final[int]  # & | ~ << >>
OPT_BITWISE_SYNTAX_BIT_FUNCTION: Final[int]  # BITAND / BITOR / BITNOT / BITSHIFTLEFT / BITSHIFTRIGHT
OPT_BITWISE_SYNTAX_SHIFT_FUNCTION: Final[int]  # BITAND / BITOR / BITNOT / SHIFTLEFT / SHIFTRIGHT

# BitwiseConfig.xor: spelling of ^.
OPT_BITWISE_XOR_UNSUPPORTED: Final[int]  # XOR is not supported.
OPT_BITWISE_XOR_CARET: Final[int]  # left ^ right
OPT_BITWISE_XOR_HASH: Final[int]  # left # right (PostgreSQL's ^ is exponent)
OPT_BITWISE_XOR_FUNCTION: Final[int]  # xor(left, right)
OPT_BITWISE_XOR_BIT_FUNCTION: Final[int]  # BITXOR(left, right)

# LikeConfig.escape: flags for where an ESCAPE clause is allowed. Empty rejects any escape character.
OPT_LIKE_ESCAPE_SCALAR: Final[int]  # Scalar LIKE ... ESCAPE, including OR expansion.
OPT_LIKE_ESCAPE_ANY: Final[int]  # Native LIKE ANY (p, ...) ESCAPE.

# LikeConfig.any: rendering of LIKE ANY / ILIKE ANY.
OPT_LIKE_ANY_OR_EXPANSION: Final[int]  # (x LIKE p1 OR x LIKE p2); for dialects without native support.
OPT_LIKE_ANY_ARRAY_CONSTRUCTOR: Final[int]  # LIKE ANY (ARRAY[p, ...])
OPT_LIKE_ANY_PATTERN_LIST: Final[int]  # LIKE ANY (p, ...)

# LikeConfig.ilike: rendering of ILIKE.
OPT_I_LIKE_STRATEGY_UNSUPPORTED: Final[int]  # ILIKE is not supported.
OPT_I_LIKE_STRATEGY_STANDARD: Final[int]  # Native ILIKE.
OPT_I_LIKE_STRATEGY_USE_LIKE_LOWER: Final[int]  # LIKE with LOWER() on both sides.

# LikeConfig.regex: spelling of regex matching.
OPT_LIKE_REGEX_UNSUPPORTED: Final[int]  # Regex matching is not supported.
OPT_LIKE_REGEX_STANDARD: Final[int]  # expr [NOT] RLIKE pattern
OPT_LIKE_REGEX_TILDE: Final[int]  # expr ~ pattern / expr !~ pattern
OPT_LIKE_REGEX_REGEXP_LIKE: Final[int]  # REGEXP_LIKE(expr, pattern)
OPT_LIKE_REGEX_REGEXP_CONTAINS: Final[int]  # REGEXP_CONTAINS(expr, pattern)
OPT_LIKE_REGEX_REGEXP_MATCHES: Final[int]  # regexp_matches(expr, pattern)

# OperatorConfig.is_distinct_from: spelling of IS [NOT] DISTINCT FROM.
OPT_IS_DISTINCT_FROM_UNSUPPORTED: Final[int]  # Not supported.
OPT_IS_DISTINCT_FROM_STANDARD: Final[int]  # left IS [NOT] DISTINCT FROM right
OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL: Final[
    int
]  # <=> is IS NOT DISTINCT FROM; IS DISTINCT FROM is NOT (left <=> right).

# OperatorConfig.modulo: spelling of %.
OPT_OPERATOR_MODULO_PERCENT: Final[int]  # left % right
OPT_OPERATOR_MODULO_FUNCTION: Final[int]  # MOD(left, right)
OPT_OPERATOR_MODULO_INFIX: Final[int]  # left MOD right

# OperatorConfig.trim: spelling of TRIM.
OPT_OPERATOR_TRIM_SQL_FROM: Final[int]  # TRIM([LEADING|TRAILING] [chars FROM] expr)
OPT_OPERATOR_TRIM_FUNCTION: Final[int]  # TRIM / LTRIM / RTRIM (expr[, chars])
OPT_OPERATOR_TRIM_SPACES_ONLY: Final[int]  # TRIM / LTRIM / RTRIM (expr); a character set raises.

# OperatorConfig.collate: spelling of COLLATE.
OPT_OPERATOR_COLLATE_UNSUPPORTED: Final[int]  # Not supported.
OPT_OPERATOR_COLLATE_INFIX_IDENT: Final[int]  # expr COLLATE ident
OPT_OPERATOR_COLLATE_FUNCTION_STRING: Final[int]  # COLLATE(expr, 'body'); the body is a string literal.

# TupleConfig.arity: flags for tuple sizes below 2 that are allowed (size 2+ is always allowed if syntax is supported).
OPT_TUPLE_ARITY_EMPTY: Final[int]  # Size 0.
OPT_TUPLE_ARITY_ONE: Final[int]  # Size 1.

# TupleConfig.syntax: how a tuple is written.
OPT_TUPLE_SYNTAX_ROW: Final[int]  # ROW(...)
OPT_TUPLE_SYNTAX_STRUCT: Final[int]  # struct(...)
OPT_TUPLE_SYNTAX_PARENTHESES: Final[int]  # (a, b, ...)
OPT_TUPLE_SYNTAX_UNSUPPORTED: Final[int]  # Tuples are not supported.

# LiteralConfig.boolean: spelling of boolean literals.
OPT_BOOL_FORMAT_KEYWORD: Final[int]  # TRUE / FALSE
OPT_BOOL_FORMAT_INTEGER: Final[int]  # 1 / 0

# SchemaConfig.features: independent schema feature flags.
OPT_SCHEMA_FEATURES_SUPPORTED: Final[int]  # Schemas are supported. Required for any schema statement.
OPT_SCHEMA_FEATURES_IF_NOT_EXISTS: Final[int]  # CREATE SCHEMA IF NOT EXISTS
OPT_SCHEMA_FEATURES_EMBEDDED_DDL: Final[int]  # DDL statements embedded in CREATE SCHEMA
OPT_SCHEMA_FEATURES_IS_DATABASE: Final[int]  # A schema means the same thing as a database.
OPT_SCHEMA_FEATURES_DROP_CASCADE: Final[int]  # DROP SCHEMA ... CASCADE
OPT_SCHEMA_FEATURES_DROP_RESTRICT: Final[int]  # DROP SCHEMA ... RESTRICT

# SchemaConfig.authorization: AUTHORIZATION owner clause.
OPT_SCHEMA_AUTHORIZATION_UNSUPPORTED: Final[int]  # Not allowed.
OPT_SCHEMA_AUTHORIZATION_OPTIONAL: Final[int]  # Allowed, but not required.

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
