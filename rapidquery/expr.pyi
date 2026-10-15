import typing
from collections.abc import Iterable
from typing import Self, final, overload

from rapidquery.dialect import Dialect
from rapidquery.serializers import Serializer

class _IdentClassVarProtocol(typing.Protocol):
    __ident__: Ident

class _IdentPropertyProtocol(typing.Protocol):
    @property
    def __ident__(self) -> Ident: ...

_IdentExtractor = str | Unquote | Ident | Asterisk | type[Asterisk] | _IdentClassVarProtocol | _IdentPropertyProtocol

class Expr:
    """
    Base of every SQL expression node.

    Do not construct `Expr` itself. Build a subclass (`Ident`, `Value`,
    `BinaryExpr`, and the rest), then render it. `to_sql` inlines literals.
    `prepare` binds them and returns the SQL plus the bound objects.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident
        >>> (Ident("age") > 18).to_sql(Sqlite)
        '"age" > 18'
    """

    def to_sql(self, dialect: Dialect) -> str:
        """
        Render this node with values written as SQL literals.

        Use this to inspect SQL. Prefer `prepare` when a driver will bind
        the parameters.

        Args:
            dialect: Dialect that chooses quoting and literal spelling.

        Returns:
            The SQL text.

        Raises:
            RuntimeError: The dialect rejects this construct.
            NotImplementedError: The dialect has no spelling for this construct.
            ValueError: A `CustomExpr` template has a different number of `{}`
                slots than arguments.
        """

    def prepare(self, dialect: Dialect) -> tuple[str, tuple[typing.Any, ...]]:
        """
        Render this node and bind each `Value` as a placeholder.

        Identifiers and keywords are still written as SQL. Only values become
        parameters, in left-to-right order. The placeholder character comes
        from the dialect (`?` or `$1`).

        Args:
            dialect: Dialect that chooses quoting and placeholders.

        Returns:
            The SQL text and the bound Python objects, in placeholder order.

        Raises:
            RuntimeError: The dialect rejects this construct.
            NotImplementedError: The dialect has no spelling for this construct.
            ValueError: A `CustomExpr` template has a different number of `{}`
                slots than arguments.

        Examples:
            >>> from rapidquery.dialect import PostgreSql
            >>> from rapidquery.expr import Value
            >>> Value(1).prepare(PostgreSql)
            ('$1', (1,))
        """

    def __add__(self, other: object, /) -> BinaryExpr:
        """SQL ``self + other``."""

    def __radd__(self, other: object, /) -> BinaryExpr:
        """SQL ``other + self``, when the left operand is not an expression."""

    def __sub__(self, other: object, /) -> BinaryExpr:
        """SQL ``self - other``."""

    def __rsub__(self, other: object, /) -> BinaryExpr:
        """SQL ``other - self``, when the left operand is not an expression."""

    def __mul__(self, other: object, /) -> BinaryExpr:
        """SQL ``self * other``."""

    def __rmul__(self, other: object, /) -> BinaryExpr:
        """SQL ``other * self``, when the left operand is not an expression."""

    def __truediv__(self, other: object, /) -> BinaryExpr:
        """SQL ``self / other``."""

    def __rtruediv__(self, other: object, /) -> BinaryExpr:
        """SQL ``other / self``, when the left operand is not an expression."""

    def __mod__(self, other: object, /) -> BinaryExpr:
        """SQL ``self % other``."""

    def __rmod__(self, other: object, /) -> BinaryExpr:
        """SQL ``other % self``, when the left operand is not an expression."""

    def __eq__(self, other: object, /) -> BinaryExpr:  # pyright: ignore[reportIncompatibleMethodOverride]
        """SQL ``self = other``."""

    def __ne__(self, other: object, /) -> BinaryExpr:  # pyright: ignore[reportIncompatibleMethodOverride]
        """SQL ``self <> other``."""

    def __lt__(self, other: object, /) -> BinaryExpr:
        """SQL ``self < other``."""

    def __le__(self, other: object, /) -> BinaryExpr:
        """SQL ``self <= other``."""

    def __gt__(self, other: object, /) -> BinaryExpr:
        """SQL ``self > other``."""

    def __ge__(self, other: object, /) -> BinaryExpr:
        """SQL ``self >= other``."""

    def __and__(self, other: object, /) -> BinaryExpr:
        """SQL ``self AND other``. Python ``&`` is logical AND, not bitwise."""

    def __rand__(self, other: object, /) -> BinaryExpr:
        """SQL ``other AND self``. Python ``&`` is logical AND, not bitwise."""

    def __or__(self, other: object, /) -> BinaryExpr:
        """SQL ``self OR other``. Python ``|`` is logical OR, not bitwise."""

    def __ror__(self, other: object, /) -> BinaryExpr:
        """SQL ``other OR self``. Python ``|`` is logical OR, not bitwise."""

    def bit_and(self, other: object, /) -> BinaryExpr:
        """SQL bitwise ``self & other``."""

    def bit_or(self, other: object, /) -> BinaryExpr:
        """
        SQL bitwise ``self | other``.

        Is not supported by Redshift, Oracle.
        """

    def bit_xor(self, other: object, /) -> BinaryExpr:
        """
        SQL bitwise ``self ^ other``.

        Is not supported by Oracle, Redshift, SQLite.
        """

    def bit_not(self) -> UnaryExpr:
        """
        SQL ``~self``. This is not bound to Python ``~``.

        Is not supported by Redshift, Oracle.
        """

    def __lshift__(self, other: object, /) -> BinaryExpr:
        """
        SQL ``self << other``.

        Is not supported by MsSQL, Oracle.
        """

    def __rlshift__(self, other: object, /) -> BinaryExpr:
        """
        SQL ``other << self``, when the left operand is not an expression.

        Is not supported by MsSQL, Oracle.
        """

    def __rshift__(self, other: object, /) -> BinaryExpr:
        """
        SQL ``self >> other``.

        Is not supported by MsSQL, Oracle.
        """

    def __rrshift__(self, other: object, /) -> BinaryExpr:
        """
        SQL ``other >> self``, when the left operand is not an expression.

        Is not supported by MsSQL, Oracle.
        """

    def concat(self, other: object, /) -> BinaryExpr:
        """SQL ``self || other``. Dialects may rewrite ``||`` as a function or as ``+``."""

    def __neg__(self) -> UnaryExpr:
        """SQL ``-self``."""

    def __pos__(self) -> UnaryExpr:
        """SQL ``+self``."""

    def not_(self) -> UnaryExpr:
        """SQL ``NOT self``."""

    def between(self, low: object, high: object, /) -> BetweenExpr:
        """SQL ``self BETWEEN low AND high``."""

    def not_between(self, low: object, high: object, /) -> BetweenExpr:
        """SQL ``self NOT BETWEEN low AND high``."""

    def is_(self, kind: None | bool | str, /) -> IsExpr:
        """
        SQL ``self IS kind``.

        ``None`` is ``IS NULL``. ``True`` and ``False`` are ``IS TRUE`` and ``IS FALSE``.
        A string is ``NULL``, ``TRUE``, ``FALSE``, or ``UNKNOWN``.

        Raises:
            TypeError: ``kind`` is not ``None``, ``bool``, or ``str``.
            ValueError: ``kind`` is a string and not one of those four words.
        """

    def is_not(self, kind: None | bool | str, /) -> IsExpr:
        """
        SQL ``self IS NOT kind``.

        ``None`` is ``IS NOT NULL``. ``True`` and ``False`` are ``IS NOT TRUE`` and
        ``IS NOT FALSE``. A string is ``NULL``, ``TRUE``, ``FALSE``, or ``UNKNOWN``.

        Raises:
            TypeError: ``kind`` is not ``None``, ``bool``, or ``str``.
            ValueError: ``kind`` is a string and not one of those four words.
        """

    def is_distinct_from(self, other: object, /) -> IsDistinctFromExpr:
        """SQL ``self IS DISTINCT FROM other``."""

    def is_not_distinct_from(self, other: object, /) -> IsDistinctFromExpr:
        """SQL ``self IS NOT DISTINCT FROM other``."""

    def in_(self, list: Iterable[object], /) -> InListExpr:
        """SQL ``self IN (list)``."""

    def not_in(self, list: Iterable[object], /) -> InListExpr:
        """SQL ``self NOT IN (list)``."""

    def like(self, pattern: object, /, *, escape: str | None = None) -> LikeExpr:
        """
        SQL ``self LIKE pattern``.

        A list or tuple of patterns is ``LIKE ANY``.

        Args:
            pattern: One pattern, or a list or tuple for ``ANY``.
            escape: One escape character, or ``None``.

        Raises:
            TypeError: ``escape`` is not one character.
        """

    def not_like(self, pattern: object, /, *, escape: str | None = None) -> LikeExpr:
        """
        SQL ``self NOT LIKE pattern``.

        A list or tuple of patterns is ``NOT LIKE ANY``.

        Args:
            pattern: One pattern, or a list or tuple for ``ANY``.
            escape: One escape character, or ``None``.

        Raises:
            TypeError: ``escape`` is not one character.
        """

    def ilike(self, pattern: object, /, *, escape: str | None = None) -> LikeExpr:
        """
        SQL ``self ILIKE pattern``.

        A list or tuple of patterns is ``ILIKE ANY``.

        Args:
            pattern: One pattern, or a list or tuple for ``ANY``.
            escape: One escape character, or ``None``.

        Raises:
            TypeError: ``escape`` is not one character.
        """

    def not_ilike(self, pattern: object, /, *, escape: str | None = None) -> LikeExpr:
        """
        SQL ``self NOT ILIKE pattern``.

        A list or tuple of patterns is ``NOT ILIKE ANY``.

        Args:
            pattern: One pattern, or a list or tuple for ``ANY``.
            escape: One escape character, or ``None``.

        Raises:
            TypeError: ``escape`` is not one character.
        """

    def similar_to(self, pattern: object, /) -> SimilarToExpr:
        """SQL ``self SIMILAR TO pattern``."""

    def not_similar_to(self, pattern: object, /) -> SimilarToExpr:
        """SQL ``self NOT SIMILAR TO pattern``."""

    def rlike(self, pattern: object, /) -> RLikeExpr:
        """SQL regex match of ``self`` against ``pattern``."""

    def not_rlike(self, pattern: object, /) -> RLikeExpr:
        """Negated SQL regex match of ``self`` against ``pattern``."""

    def cast_as(self, data_type_name: str, /) -> CastExpr:
        """SQL ``CAST(self AS data_type_name)``."""

    def try_cast_as(self, data_type_name: str, /) -> CastExpr:
        """SQL ``TRY_CAST(self AS data_type_name)``."""

    def collate(self, collation: _IdentExtractor, /) -> CollateExpr:
        """SQL ``self COLLATE collation``. The collation is an identifier."""

    def trim(
        self,
        *,
        where: typing.Literal["BOTH", "LEADING", "TRAILING"] | str = "BOTH",
        characters: object | None = None,
    ) -> TrimExpr:
        """
        SQL ``TRIM``, ``LTRIM``, or ``RTRIM`` of this expression.

        Args:
            where: ``BOTH``, ``LEADING``, or ``TRAILING``.
            characters: Characters to strip, or ``None`` for spaces.

        Raises:
            ValueError: ``where`` is not one of those three words.
        """

    def __repr__(self) -> str:
        """Constructor-style text, such as ``Ident("id")``."""

@final
class Unquote:
    """
    Identifier body that is written without quote characters.

    Use this for a keyword-like name that must stay bare, such as
    `CURRENT_TIMESTAMP`. The name must match `[A-Za-z_][A-Za-z0-9_]*`.
    This is not a raw SQL fragment.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Ident, Unquote
        >>> Ident(Unquote("COUNT")).to_sql(PostgreSql)
        'COUNT'
    """

    def __new__(cls, name: str, /) -> Self:
        """
        Store `name` as an unquoted identifier part.

        Args:
            name: ASCII identifier body. A digit cannot be first.

        Raises:
            ValueError: `name` is empty or is not a simple ASCII identifier.
        """

    @property
    def name(self) -> str:
        """The identifier body, without quotes."""

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Asterisk:
    """
    The SQL wildcard `*`.

    Pass the class or an instance as the last part of an `Ident` (`t.*`).
    `Ident("*")` is a quoted column whose name is a star, not this wildcard.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Asterisk, Ident
        >>> Ident("t", Asterisk).to_sql(PostgreSql)
        '"t".*'
    """

    def __new__(cls) -> Self:
        """Return the wildcard marker."""

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Ident(Expr):
    """
    Dotted SQL name: schema, table, column, or a final wildcard.

    Use `Ident` wherever a name must be quoted for the dialect. Pass one
    argument per part. A `str` is always quoted, including `"*"`. Flatten a
    nested `Ident` by passing it as one argument. At most 10 parts.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Ident
        >>> Ident("users", "id").to_sql(PostgreSql)
        '"users"."id"'
    """

    def __new__(cls, *parts: _IdentExtractor) -> Self:
        """
        Build an identifier from parts.

        Args:
            parts: One part each. A `str` is quoted. `Unquote` stays bare.
                `Asterisk` is legal only as the last part.

        Raises:
            TypeError: A part is not a string, `Unquote`, `Asterisk`, or `Ident`.
            ValueError: There are no parts, a part is empty, `Asterisk` is not
                last, or there are more than 10 parts.
        """

    def __len__(self) -> int:
        """Number of parts. At least one."""

    def __getitem__(self, index: int) -> str | Unquote | Asterisk:
        """
        Return the part at `index`.

        A quoted part comes back as `str`. Negative indexes count from the end.

        Raises:
            IndexError: `index` is outside the parts.
        """

    def to_list(self) -> list[str | Unquote | Asterisk]:
        """
        Copy the parts into a list.

        Quoted parts are `str`. Unquoted parts are `Unquote`. A wildcard is
        `Asterisk`.
        """

_T = typing.TypeVar("_T")

@final
class Value(Expr, typing.Generic[_T]):
    """
    A Python object kept until SQL is rendered.

    Use `Value` for literals and for driver parameters. `to_sql` runs the
    serializer (`None` becomes `NULL`). `prepare` binds the object and does
    not call the serializer. Omit the serializer to infer one from the
    object's type. Passing an existing `Value` reuses its object.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Value
        >>> Value("hi").to_sql(PostgreSql)
        "E'hi'"
    """

    @overload
    def __new__(cls, object: _T | None, /) -> Self:
        """Wrap `object` and infer a serializer from its type."""

    @overload
    def __new__(cls, object: _T | None, serializer: Serializer[_T] | None, /) -> Self:
        """
        Wrap `object` with an explicit serializer.

        Args:
            object: Value to bind or serialize. An existing `Value` contributes
                its object.
            serializer: How `to_sql` spells `object`. `None` infers one, or
                keeps the serializer when `object` is already a `Value`.

        Raises:
            TypeError: `serializer` is not a `Serializer`.
        """

    def get_object(self) -> _T | None:
        """The wrapped Python object. `None` renders as `NULL`."""

    def get_serializer(self) -> Serializer[_T]:
        """Serializer used by `to_sql`. `prepare` does not call it."""

@final
class Function(Expr):
    """
    SQL function call, including a bare name with no arguments.

    Use this for `count`, `sum`, and dialect functions that are not their own
    node. The name is an identifier: a string is quoted, so a niladic keyword
    such as `CURRENT_TIMESTAMP` must be an `Unquote`. `filter` adds
    `FILTER (WHERE …)` where the dialect allows it. Null treatment is a
    separate clause, placed where that dialect requires it.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Function, Ident
        >>> Function("count", Ident("id")).to_sql(PostgreSql)
        '"count"("id")'
    """

    def __new__(cls, name: _IdentExtractor, /, *args: object, distinct: bool = False) -> Self:
        """
        Build a call.

        Args:
            name: Function name. A `str` is a quoted identifier, and a dotted
                string is split into parts.
            args: Arguments. A `str` becomes a `Value`, not an identifier.
            distinct: When true, write `DISTINCT` before the arguments.

        Raises:
            TypeError: `name` is not an identifier.
            ValueError: `name` is not a legal identifier.
        """

    def filter(self, predicate: object, /) -> Self:
        """
        Return a copy with `FILTER (WHERE predicate)`.

        The predicate is any expression. A string becomes a literal. Rendering
        raises when the dialect has no `FILTER` syntax.

        Args:
            predicate: Boolean expression placed in the filter.

        This method is not supported by Hive, MsSQL, MySQL, Redshift, Snowflake, Teradata.
        """

    def ignore_nulls(self) -> Self:
        """
        Return a copy that writes `IGNORE NULLS`.

        Is not supported by Hive, MySQL, PostgreSql, Sqlite.
        """

    def respect_nulls(self) -> Self:
        """
        Return a copy that writes `RESPECT NULLS`.

        Is not supported by Hive, MySQL, PostgreSql, Sqlite.
        """

    def get_name(self) -> Ident:
        """The function name."""

    def get_distinct(self) -> bool:
        """Whether the call writes `DISTINCT`."""

    def get_args(self) -> tuple[Expr, ...]:
        """The argument expressions, in order."""

    def get_filter(self) -> Expr | None:
        """The `FILTER` predicate, or `None`."""

    def get_null_treatment(self) -> typing.Literal["IGNORE NULLS", "RESPECT NULLS"] | None:
        """`IGNORE NULLS`, `RESPECT NULLS`, or `None`."""

@final
class TupleExpr(Expr):
    """
    Parenthesized row value.

    Use this for `ROW(a, b)` and the other tuple spellings a dialect allows.
    A string argument is a literal, not a column name. Rendering raises when
    the dialect rejects tuples or rejects this arity.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import TupleExpr
        >>> TupleExpr(1, 2).to_sql(PostgreSql)
        'ROW(1, 2)'
    """

    def __new__(cls, *items: object) -> Self:
        """
        Build a row from expressions.

        Args:
            items: One expression per element. Strings become literals.
        """

    def get_items(self) -> tuple[Expr, ...]:
        """The element expressions, in order."""

@final
class UnaryExpr(Expr):
    """Prefix operator: `NOT`, `+`, `-`, or `~`.

    Use this for negation and bitwise complement. `NOT` is written with a
    space. The others are written against the operand. `~` becomes a function
    on dialects that have no prefix complement.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident, UnaryExpr
        >>> UnaryExpr("NOT", Ident("active")).to_sql(Sqlite)
        'NOT "active"'
    """

    def __new__(cls, operator: typing.Literal["NOT", "+", "-", "~"] | str, expr: object, /) -> Self:
        """
        Build a prefix operation.

        Args:
            operator: `NOT`, `+`, `-`, or `~`. Case is ignored for `NOT`.
            expr: The operand. A string becomes a literal.

        Raises:
            ValueError: `operator` is not one of those four spellings.
        """

    def get_operator(self) -> typing.Literal["NOT", "+", "-", "~"]:
        """The operator spelling."""

    def get_expr(self) -> Expr:
        """The operand."""

@final
class BinaryExpr(Expr):
    """
    Infix operator between two expressions.

    Use this for arithmetic, comparison, `AND` / `OR`, concatenation, and
    bitwise operators. `!=` is stored and written as `<>`. A string on either
    side is a literal. Dialects may rewrite `%`, `^`, `||`, and shifts into
    function calls.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import BinaryExpr, Ident
        >>> BinaryExpr(Ident("age"), ">", 18).to_sql(Sqlite)
        '"age" > 18'
    """

    def __new__(
        cls,
        left: object,
        operator: typing.Literal[
            "+",
            "-",
            "*",
            "/",
            "%",
            "=",
            "<>",
            "!=",
            "<",
            "<=",
            ">=",
            ">",
            "AND",
            "OR",
            "&",
            "|",
            "^",
            "<<",
            ">>",
            "||",
        ]
        | str,
        right: object,
        /,
    ) -> Self:
        """
        Build an infix operation.

        Args:
            left: Left operand.
            operator: One of the spellings listed on this method. `!=` is stored
                as `<>`. Any other spelling is kept as a custom operator.
            right: Right operand.
        """

    def get_left(self) -> Expr:
        """The left operand."""

    def get_operator(self) -> str:
        """The operator spelling. `!=` is returned as `<>`."""

    def get_right(self) -> Expr:
        """The right operand."""

@final
class IsExpr(Expr):
    """`expr IS [NOT] NULL`, `TRUE`, `FALSE`, or `UNKNOWN`.

    Use this for null tests and boolean state tests. Pass the kind as a
    string. Set `negated` for `IS NOT`.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident, IsExpr
        >>> IsExpr(Ident("name"), "NULL").to_sql(Sqlite)
        '"name" IS NULL'
    """

    def __new__(
        cls,
        expr: object,
        kind: typing.Literal["NULL", "TRUE", "FALSE", "UNKNOWN"] | str,
        /,
        *,
        negated: bool = False,
    ) -> Self:
        """
        Build an `IS` test.

        Args:
            expr: Tested expression.
            kind: `NULL`, `TRUE`, `FALSE`, or `UNKNOWN`.
            negated: Write `IS NOT` instead of `IS`.

        Raises:
            ValueError: `kind` is not one of those four words.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_kind(self) -> typing.Literal["NULL", "TRUE", "FALSE", "UNKNOWN"]:
        """`NULL`, `TRUE`, `FALSE`, or `UNKNOWN`."""

    def get_negated(self) -> bool:
        """Whether the test is `IS NOT`."""

@final
class IsDistinctFromExpr(Expr):
    """`left IS [NOT] DISTINCT FROM right`.

    Use this when `NULL` on either side must not make the comparison unknown.
    Rendering follows the dialect: standard syntax, a null-safe operator, or
    an error when the dialect has neither.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Ident, IsDistinctFromExpr
        >>> IsDistinctFromExpr(Ident("a"), Ident("b")).to_sql(PostgreSql)
        '"a" IS DISTINCT FROM "b"'
    """

    def __new__(cls, left: object, right: object, /, *, negated: bool = False) -> Self:
        """
        Build an `IS DISTINCT FROM` comparison.

        Args:
            left: Left expression.
            right: Right expression.
            negated: Write `IS NOT DISTINCT FROM`.

        ``IS DISTINCT FROM`` is not supported by Oracle, Redshift, Teradata.
        """

    def get_left(self) -> Expr:
        """The left expression."""

    def get_right(self) -> Expr:
        """The right expression."""

    def get_negated(self) -> bool:
        """Whether the predicate is negated."""

@final
class InListExpr(Expr):
    """`expr [NOT] IN (item, …)`.

    Use this for a fixed list of expressions. A string item is a literal.
    An empty list renders as a boolean literal and does not write `expr`.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident, InListExpr
        >>> InListExpr(Ident("id"), [1, 2]).to_sql(Sqlite)
        '"id" IN (1, 2)'
    """

    def __new__(cls, expr: object, list: Iterable[object], /, *, negated: bool = False) -> Self:
        """
        Build an `IN` list.

        Args:
            expr: Tested expression.
            list: Membership expressions.
            negated: Write `NOT IN`.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_list(self) -> tuple[Expr, ...]:
        """The membership expressions."""

    def get_negated(self) -> bool:
        """Whether the predicate is `NOT IN`."""

@final
class BetweenExpr(Expr):
    """`expr [NOT] BETWEEN low AND high`.

    Use this for an inclusive range. Both bounds are expressions. There is
    no `SYMMETRIC` clause. Some dialects parenthesize the whole predicate.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import BetweenExpr, Ident
        >>> BetweenExpr(Ident("n"), 1, 3).to_sql(Sqlite)
        '"n" BETWEEN 1 AND 3'
    """

    def __new__(cls, expr: object, low: object, high: object, /, *, negated: bool = False) -> Self:
        """
        Build a `BETWEEN` predicate.

        Args:
            expr: Tested expression.
            low: Inclusive lower bound.
            high: Inclusive upper bound.
            negated: Write `NOT BETWEEN`.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_low(self) -> Expr:
        """The lower bound."""

    def get_high(self) -> Expr:
        """The upper bound."""

    def get_negated(self) -> bool:
        """Whether the predicate is `NOT BETWEEN`."""

@final
class LikeExpr(Expr):
    """`LIKE`, `ILIKE`, or `LIKE ANY`, with an optional `ESCAPE` character.

    Use this for pattern predicates. A string pattern is one pattern. A list
    or tuple of patterns is `LIKE ANY`, spelled the way the dialect allows.
    `case_sensitive=False` selects `ILIKE` or the dialect's fallback. `escape`
    must be one character and is inlined, never bound.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident, LikeExpr
        >>> LikeExpr(Ident("name"), "A%").to_sql(Sqlite)
        '"name" LIKE \\'A%\\''
    """

    def __new__(
        cls,
        expr: object,
        pattern: object,
        /,
        *,
        negated: bool = False,
        case_sensitive: bool = True,
        escape: str | None = None,
    ) -> Self:
        """
        Build a pattern predicate.

        Args:
            expr: Tested expression.
            pattern: One pattern, or a list or tuple of patterns for `ANY`.
            negated: Negate the match.
            case_sensitive: When false, render the dialect's case-insensitive form.
            escape: One escape character, or `None` to omit ``ESCAPE``. ``ESCAPE`` is
                    not supported by BigQuery, Hive.

        Raises:
            TypeError: `escape` is not one character, or `pattern` is not an
                expression, list, or tuple.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_pattern(self) -> Expr | tuple[Expr, ...]:
        """One pattern, or a tuple of patterns when this is `LIKE ANY`."""

    def get_negated(self) -> bool:
        """Whether the match is negated."""

    def get_case_sensitive(self) -> bool:
        """Whether the match keeps case."""

    def get_escape(self) -> str | None:
        """The escape character, or `None`."""

@final
class SimilarToExpr(Expr):
    """`expr [NOT] SIMILAR TO pattern`.

    Use this only for dialects that implement SQL `SIMILAR TO` (PostgreSQL,
    Redshift, DuckDB). Other dialects raise at render time. There is no
    `ESCAPE` clause.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Ident, SimilarToExpr
        >>> SimilarToExpr(Ident("name"), "a%").to_sql(PostgreSql)
        '"name" SIMILAR TO E\\'a%\\''
    """

    def __new__(cls, expr: object, pattern: object, /, *, negated: bool = False) -> Self:
        """
        Build a `SIMILAR TO` predicate.

        Args:
            expr: Tested expression.
            pattern: SQL regular pattern.
            negated: Write `NOT SIMILAR TO`.

        ``SIMILAR TO`` is supported by DuckDB, PostgreSql, Redshift.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_pattern(self) -> Expr:
        """The pattern."""

    def get_negated(self) -> bool:
        """Whether the predicate is negated."""

@final
class RLikeExpr(Expr):
    """Regex match, spelled the way the dialect spells it.

    Use this instead of writing `RLIKE`, `~`, or `REGEXP_LIKE` yourself.
    PostgreSQL and Redshift use `~`. Other dialects use `RLIKE` or a function,
    or raise when they have no regex predicate. There is no `ESCAPE` clause.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import Ident, RLikeExpr
        >>> RLikeExpr(Ident("name"), "a.*").to_sql(PostgreSql)
        '"name" ~ E\\'a.*\\''
    """

    def __new__(cls, expr: object, pattern: object, /, *, negated: bool = False) -> Self:
        """
        Build a regex predicate.

        Args:
            expr: Tested expression.
            pattern: Regular expression.
            negated: Negate the match.

        ``RLIKE`` is not supported by MsSQL, Sqlite, Teradata.
        """

    def get_expr(self) -> Expr:
        """The tested expression."""

    def get_pattern(self) -> Expr:
        """The regular expression."""

    def get_negated(self) -> bool:
        """Whether the match is negated."""

@final
class CastExpr(Expr):
    """`CAST`, `TRY_CAST`, or PostgreSQL-style `::`.

    Use this to name the result type. The type string is a SQL type name, not
    a quoted identifier. `TRY_CAST` is rewritten to the dialect's spelling,
    such as `SAFE_CAST`, or rejected when the dialect has no try-cast.
    `kind="::"` renders only when the dialect allows that operator.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import CastExpr, Ident
        >>> CastExpr(Ident("id"), "int").to_sql(PostgreSql)
        'CAST("id" AS INT)'
    """

    def __new__(
        cls,
        expr: object,
        data_type_name: str,
        /,
        *,
        kind: typing.Literal["CAST", "TRY_CAST", "::"] | str = "CAST",
    ) -> Self:
        """
        Build a cast.

        Args:
            expr: Input expression.
            data_type_name: SQL type name, such as `"int"` or `"varchar(16)"`.
            kind: `CAST`, `TRY_CAST`, or `::`. ``TRY_CAST`` is not supported by
                  Hive, MySQL, Oracle, PostgreSql, Sqlite. ``::`` is supported
                  by Databricks, DuckDB, PostgreSql, Redshift, Snowflake, Spark.

        Raises:
            ValueError: `kind` is not one of those three spellings.
        """

    def get_expr(self) -> Expr:
        """The input expression."""

    def get_data_type_name(self) -> str:
        """The SQL type name, in the spelling that will be written."""

    def get_kind(self) -> typing.Literal["CAST", "TRY_CAST", "::"]:
        """`CAST`, `TRY_CAST`, or `::`."""

@final
class TrimExpr(Expr):
    """`TRIM` of an expression, from one side, of an optional character set.

    Use this for `TRIM`, `LTRIM`, and `RTRIM`. `where` selects both ends,
    the leading end, or the trailing end. `characters` is the set to strip.
    Omit it to strip spaces. The written function depends on the dialect.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import Ident, TrimExpr
        >>> TrimExpr(Ident("s")).to_sql(Sqlite)
        'TRIM("s")'
    """

    def __new__(
        cls,
        expr: object,
        /,
        *,
        where: typing.Literal["BOTH", "LEADING", "TRAILING"] | str = "BOTH",
        characters: object | None = None,
    ) -> Self:
        """
        Build a trim call.

        Args:
            expr: Input expression.
            where: `BOTH`, `LEADING`, or `TRAILING`.
            characters: Characters to strip, or `None` for spaces.

        Raises:
            ValueError: `where` is not one of those three words.
        """

    def get_expr(self) -> Expr:
        """The input expression."""

    def get_where(self) -> typing.Literal["BOTH", "LEADING", "TRAILING"]:
        """Which side is trimmed."""

    def get_characters(self) -> Expr | None:
        """The character set, or `None` when trimming spaces."""

@final
class CollateExpr(Expr):
    """
    `expr COLLATE collation`.

    Use this to attach a collation name. The collation is an identifier, not
    a string literal, except on dialects that spell collation as a function
    and then quote the name as text. Rendering raises when the dialect has
    no collation syntax.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.expr import CollateExpr, Ident
        >>> CollateExpr(Ident("s"), "en_US").to_sql(PostgreSql)
        '"s" COLLATE "en_US"'
    """

    def __new__(cls, expr: object, collation: str | Unquote | Ident | Asterisk | type[Asterisk], /) -> Self:
        """
        Attach a collation.

        Args:
            expr: Input expression.
            collation: Collation name, as an identifier.

        ``COLLATE`` is not supported by Hive, Spark, Teradata.
        """

    def get_expr(self) -> Expr:
        """The input expression."""

    def get_collation(self) -> Ident:
        """The collation name."""

@final
class CaseExpr(Expr):
    """`CASE` expression, searched or with an operand.

    Use this for conditional results. Omit the operand for `CASE WHEN`.
    Pass an operand for `CASE operand WHEN`. `when` and `else_` return a new
    node. A `CASE` with no branches is still constructible; render it only
    after adding a branch.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import CaseExpr, Ident
        >>> CaseExpr().when(Ident("a"), 1).else_(0).to_sql(Sqlite)
        'CASE WHEN "a" THEN 1 ELSE 0 END'
    """

    def __new__(cls, operand: object | None = None, /) -> Self:
        """
        Start a `CASE`.

        Args:
            operand: The `CASE operand` expression, or `None` for a searched
                `CASE`.
        """

    def when(self, when: object, then: object) -> Self:
        """Return a copy with one more `WHEN` / `THEN` pair.

        Args:
            when: Comparison value, or a condition when there is no operand.
            then: Result when this branch matches.
        """

    def else_(self, result: object) -> Self:
        """
        Return a copy whose `ELSE` result is `result`.

        Args:
            result: Value written after `ELSE`.
        """

    def get_operand(self) -> Expr | None:
        """The `CASE` operand, or `None` for a searched `CASE`."""

    def get_when_list(self) -> tuple[tuple[Expr, Expr], ...]:
        """`(WHEN, THEN)` pairs, in order."""

    def get_else(self) -> Expr | None:
        """The `ELSE` expression, or `None`."""

@final
class CustomExpr(Expr):
    """SQL template with `{}` slots filled by expressions.

    Use this when no node spells the fragment yet. Each `{}` consumes one
    argument, rendered with the same dialect. `{{` and `}}` write a literal
    brace. The template is not escaped. Keep it to keywords and punctuation
    you trust, and put data in the arguments.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.expr import CustomExpr, Ident
        >>> CustomExpr("COALESCE({}, 0)", Ident("n")).to_sql(Sqlite)
        'COALESCE("n", 0)'
    """

    def __new__(cls, sql: str, *args: object) -> Self:
        """
        Build a template.

        Args:
            sql: SQL text. `{}` marks an argument. `{{` and `}}` are braces.
            args: One expression per `{}` slot.

        The slot count is checked when the expression is rendered, not here.
        """

    def get_sql(self) -> str:
        """The template, including `{}` slots."""

    def get_args(self) -> tuple[Expr, ...]:
        """The expressions that fill `{}` slots, in order."""
