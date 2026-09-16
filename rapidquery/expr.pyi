import typing
from collections.abc import Iterable, Sequence
from typing import Self, final, overload

from rapidquery.data_type import DataType, Serializer

class Expr:
    """
    Abstract SQL expression.

    Concrete nodes are frozen subclasses (``Ident``, ``Value``,
    ``Function``, ``BinaryExpr``, …). This base class cannot be
    instantiated.
    """

@final
class Unquote:
    """
    Marker for an unquoted identifier part.

    The body is emitted without dialect delimiters. It must be a simple
    ASCII name (``[A-Za-z_][A-Za-z0-9_]*``). This is not a raw-SQL
    fragment.
    """

    def __new__(cls, name: str, /) -> Self:
        """
        Create an unquoted identifier part.

        Args:
            name: The identifier body, unquoted.

        Returns:
            A new ``Unquote``.

        Raises:
            ValueError: If ``name`` is empty or not a simple ASCII
                identifier (letters, digits, underscore; first character
                not a digit).
        """

    @property
    def name(self) -> str:
        """The unquoted identifier body.

        Returns:
            The string passed to the constructor.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Asterisk:
    """
    SQL wildcard ``*``.

    Pass the class (``Asterisk``) or an instance (``Asterisk()``) as an
    identifier part. It must be the last part of an ``Ident`` (``t.*``).
    ``Ident("*")`` is a quoted column named star, not this wildcard.
    """

    def __new__(cls) -> Self:
        """
        Create a wildcard part.

        Returns:
            A new ``Asterisk``.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Ident(Expr):
    """
    SQL identifier (``schema.table.column``).

    An identifier is a sequence of parts. Each constructor argument is
    one part: a quoted name (``str``), an ``Unquote`` name, an
    ``Asterisk`` wildcard, or a nested ``Ident`` whose parts are
    flattened.

    Never empty. At most 10 parts. An asterisk may appear only as the
    last part.
    """

    def __new__(cls, *parts: str | Unquote | Ident | Asterisk | type[Asterisk]) -> Self:
        """
        Create a SQL identifier.

        Args:
            *parts: One argument per part. A ``str`` is always quoted,
                including ``"*"`` (a column named star). Pass
                ``Asterisk`` for the SQL wildcard. Dotted names are
                ``Ident("schema", "table")``, not
                ``Ident("schema.table")``. Sequences are not accepted.

        Returns:
            A new ``Ident``.

        Raises:
            TypeError: If an argument is not ``str``, ``Unquote``,
                ``Asterisk``, or ``Ident``.
            ValueError: If there are no parts, a part is empty,
                ``Asterisk`` is not last, an ``Unquote`` body is not a
                simple ASCII name, or there are more than 10 parts.
        """

    def __len__(self) -> int:
        """
        Return the number of identifier parts.

        Returns:
            Part count; always at least 1.
        """

    def to_list(self) -> Sequence[str | Unquote | Asterisk]:
        """
        Return the parts as a list.

        Returns:
            Each part as a ``str``, ``Unquote``, or ``Asterisk``.
        """

_T = typing.TypeVar("_T")

@final
class Value(Expr, typing.Generic[_T]):
    """
    A Python object plus the serializer used to emit it as SQL.

    ``Value`` does not convert the object until SQL is built.
    Parameterized statements bind the object as a placeholder. Raw SQL
    runs the serializer (``None`` becomes ``NULL``).

    If ``serializer`` is omitted, one is inferred from the object's
    type. Passing an existing ``Value`` keeps its object (and its
    serializer, unless a new one is given). A ``DataType`` as
    ``serializer`` contributes only its serializer; the data type is
    not stored.
    """

    @overload
    def __new__(cls, object: _T, /) -> typing.Self: ...
    @overload
    def __new__(
        cls,
        object: _T,
        serializer: Serializer[_T] | DataType[_T] | None = None,
        /,
    ) -> typing.Self:
        """
        Create a SQL value.

        Args:
            object: The Python object to bind or serialize. If this is
                already a ``Value``, its object is reused.
            serializer: How to render ``object`` as a SQL literal. A
                ``DataType`` is accepted and only its serializer is
                kept. If omitted, a serializer is inferred from
                ``object`` (or kept, when ``object`` is already a
                ``Value``).

        Returns:
            A new ``Value``.

        Raises:
            TypeError: If ``serializer`` is not a ``Serializer`` or
                ``DataType``.
        """

    def get_object(self) -> _T:
        """
        Return the wrapped Python object.

        Returns:
            The object passed to the constructor (or taken from an
            existing ``Value``).
        """

    def get_serializer(self) -> Serializer[_T]:
        """
        Return the serializer used for raw SQL literals.

        Returns:
            The ``Serializer`` stored on this value. Not used when the
            value is bound as a parameter.
        """

@final
class Function(Expr):
    """
    SQL function call.

    ``name`` is an identifier (``SUM``, ``schema.fn``). Passing no
    positional arguments emits a bare call such as
    ``CURRENT_TIMESTAMP`` (no parentheses). ``distinct=True`` inserts
    ``DISTINCT`` inside the parentheses.
    """

    def __new__(
        cls,
        name: str | Ident | Unquote,
        *args: object,
        distinct: bool | None = None,
        filter: Expr | None = None,
    ) -> Self:
        """
        Create a function call.

        Args:
            name: Function name (``str``, ``Ident``, or ``Unquote``).
            *args: Positional argument expressions. A bare ``str`` is a
                column. An empty argument list means no parentheses
                (``CURRENT_TIMESTAMP``).
            distinct: ``True`` inserts ``DISTINCT``, ``False`` inserts
                ``ALL``, omitted inserts neither.
            filter: Optional ``FILTER (WHERE …)`` predicate.

        Returns:
            A new ``Function``.

        Raises:
            TypeError: If ``name`` is not an identifier.
            ValueError: If ``name`` is an empty or invalid identifier.
        """

    def get_name(self) -> Ident:
        """
        Return the function name.

        Returns:
            The function name as an ``Ident``.
        """

    def get_args(self) -> tuple[Expr, ...] | None:
        """
        Return the positional arguments.

        Returns:
            A tuple of argument expressions, or ``None`` when the call
            has no parentheses.
        """

    def get_filter(self) -> Expr | None:
        """
        Return the ``FILTER (WHERE …)`` predicate, if any.

        Returns:
            The filter expression, or ``None``.
        """

@final
class TupleExpr(Expr):
    """Parenthesized row / tuple expression ``(a, b, …)``."""

    def __new__(cls, *items: object) -> Self:
        """
        Create a tuple expression.

        Args:
            *items: Elements of the tuple. A bare ``str`` is a column.

        Returns:
            A new ``TupleExpr``.
        """

    def get_items(self) -> tuple[Expr, ...]:
        """
        Return the tuple elements.

        Returns:
            The elements as a tuple of expressions.
        """

@final
class UnaryExpr(Expr):
    """Unary expression (``NOT x``, ``-x``, ``+x``, ``~x``)."""

    def __new__(cls, operator: str, expr: object, /) -> Self:
        """
        Create a unary expression.

        Args:
            operator: One of ``NOT``, ``+``, ``-``, or ``~``.
            expr: The operand. A bare ``str`` is a column.

        Returns:
            A new ``UnaryExpr``.

        Raises:
            ValueError: If ``operator`` is not a known unary operator.
        """

    def get_operator(self) -> str:
        """
        Return the unary operator spelling.

        Returns:
            ``NOT``, ``+``, ``-``, or ``~``.
        """

    def get_expr(self) -> Expr:
        """
        Return the operand.

        Returns:
            The operand expression.
        """

@final
class BinaryExpr(Expr):
    """
    Binary expression (``a + b``, ``a AND b``, custom operators).

    A bare ``str`` on either side is a column. Pass ``Value(...)`` for a
    string literal. Unknown operator spellings are stored as a custom
    operator (for example ``->``).
    """

    def __new__(cls, left: object, operator: str, right: object, /) -> Self:
        """
        Create a binary expression.

        Args:
            left: Left operand. A bare ``str`` is a column.
            operator: SQL operator spelling (``+``, ``AND``, ``=``,
                ``->``, …). ``!=`` is stored as ``<>``.
            right: Right operand. A bare ``str`` is a column.

        Returns:
            A new ``BinaryExpr``.
        """

    def get_left(self) -> Expr:
        """
        Return the left operand.

        Returns:
            The left-hand expression.
        """

    def get_operator(self) -> str:
        """
        Return the operator spelling.

        Returns:
            The SQL operator (``+``, ``AND``, ``<>``, or the custom
            string).
        """

    def get_right(self) -> Expr:
        """
        Return the right operand.

        Returns:
            The right-hand expression.
        """

@final
class IsExpr(Expr):
    """``expr IS [NOT] {NULL | TRUE | FALSE | UNKNOWN}``."""

    def __new__(cls, expr: object, kind: str, /, *, negated: bool = False) -> Self:
        """
        Create an ``IS`` predicate.

        Args:
            expr: The tested expression. A bare ``str`` is a column.
            kind: ``NULL``, ``TRUE``, ``FALSE``, or ``UNKNOWN``.
            negated: If ``True``, emit ``IS NOT``.

        Returns:
            A new ``IsExpr``.

        Raises:
            ValueError: If ``kind`` is not one of the accepted spellings.
        """

    def get_expr(self) -> Expr:
        """
        Return the tested expression.

        Returns:
            The left-hand expression.
        """

    def get_kind(self) -> str:
        """
        Return the ``IS`` kind.

        Returns:
            ``NULL``, ``TRUE``, ``FALSE``, or ``UNKNOWN``.
        """

    def get_negated(self) -> bool:
        """
        Return whether this is ``IS NOT``.

        Returns:
            ``True`` if the predicate is negated.
        """

@final
class IsDistinctFromExpr(Expr):
    """``a IS [NOT] DISTINCT FROM b``."""

    def __new__(cls, left: object, right: object, /, *, negated: bool = False) -> Self:
        """
        Create an ``IS DISTINCT FROM`` predicate.

        Args:
            left: Left operand. A bare ``str`` is a column.
            right: Right operand. A bare ``str`` is a column.
            negated: If ``True``, emit ``IS NOT DISTINCT FROM``.

        Returns:
            A new ``IsDistinctFromExpr``.
        """

    def get_left(self) -> Expr:
        """
        Return the left operand.

        Returns:
            The left-hand expression.
        """

    def get_right(self) -> Expr:
        """
        Return the right operand.

        Returns:
            The right-hand expression.
        """

    def get_negated(self) -> bool:
        """
        Return whether this is ``IS NOT DISTINCT FROM``.

        Returns:
            ``True`` if the predicate is negated.
        """

@final
class InListExpr(Expr):
    """``expr [NOT] IN (…)``."""

    def __new__(
        cls, expr: object, list: Iterable[object], /, *, negated: bool = False
    ) -> Self:
        """
        Create an ``IN`` list predicate.

        Args:
            expr: The tested expression. A bare ``str`` is a column.
            list: Iterable of list elements. Each bare ``str`` is a
                column.
            negated: If ``True``, emit ``NOT IN``.

        Returns:
            A new ``InListExpr``.
        """

    def get_expr(self) -> Expr:
        """
        Return the tested expression.

        Returns:
            The left-hand expression.
        """

    def get_list(self) -> tuple[Expr, ...]:
        """
        Return the ``IN`` list.

        Returns:
            The list elements as a tuple of expressions.
        """

    def get_negated(self) -> bool:
        """
        Return whether this is ``NOT IN``.

        Returns:
            ``True`` if the predicate is negated.
        """

@final
class BetweenExpr(Expr):
    """``expr [NOT] BETWEEN low AND high``."""

    def __new__(
        cls,
        expr: object,
        low: object,
        high: object,
        /,
        *,
        negated: bool = False,
    ) -> Self:
        """
        Create a ``BETWEEN`` predicate.

        Args:
            expr: The tested expression. A bare ``str`` is a column.
            low: Lower bound. A bare ``str`` is a column.
            high: Upper bound. A bare ``str`` is a column.
            negated: If ``True``, emit ``NOT BETWEEN``.

        Returns:
            A new ``BetweenExpr``.
        """

    def get_expr(self) -> Expr:
        """
        Return the tested expression.

        Returns:
            The left-hand expression.
        """

    def get_low(self) -> Expr:
        """
        Return the lower bound.

        Returns:
            The low expression.
        """

    def get_high(self) -> Expr:
        """
        Return the upper bound.

        Returns:
            The high expression.
        """

    def get_negated(self) -> bool:
        """
        Return whether this is ``NOT BETWEEN``.

        Returns:
            ``True`` if the predicate is negated.
        """

@final
class LikeExpr(Expr):
    """``expr [NOT] LIKE / ILIKE / SIMILAR TO / RLIKE pattern``.

    The pattern uses literal-priority extraction: a bare ``str`` is a
    string value, not a column.
    """

    def __new__(
        cls,
        expr: object,
        pattern: object,
        /,
        *,
        kind: str = "LIKE",
        negated: bool = False,
        escape: str | None = None,
    ) -> Self:
        """
        Create a ``LIKE``-family predicate.

        Args:
            expr: The tested expression. A bare ``str`` is a column.
            pattern: The pattern. A bare ``str`` is a string literal.
            kind: ``LIKE``, ``ILIKE``, ``SIMILAR TO``, or ``RLIKE``.
            negated: If ``True``, emit ``NOT LIKE`` (or the matching
                negated form).
            escape: Optional single-character ``ESCAPE`` clause.

        Returns:
            A new ``LikeExpr``.

        Raises:
            ValueError: If ``kind`` is not a known ``LIKE`` spelling.
        """

    def get_expr(self) -> Expr:
        """
        Return the tested expression.

        Returns:
            The left-hand expression.
        """

    def get_pattern(self) -> Expr:
        """
        Return the pattern expression.

        Returns:
            Usually a ``Value`` when constructed from a string.
        """

    def get_kind(self) -> str:
        """
        Return the operator kind.

        Returns:
            ``LIKE``, ``ILIKE``, ``SIMILAR TO``, or ``RLIKE``.
        """

    def get_negated(self) -> bool:
        """
        Return whether this is a negated match.

        Returns:
            ``True`` if the predicate is negated.
        """

    def get_escape(self) -> str | None:
        """
        Return the ``ESCAPE`` character, if any.

        Returns:
            A single character, or ``None``.
        """

@final
class CastExpr(Expr):
    """``CAST(expr AS type)``."""

    def __new__(cls, expr: object, to: str, /) -> Self:
        """
        Create a ``CAST`` expression.

        Args:
            expr: The expression to cast. A bare ``str`` is a column.
            to: SQL type name (``INT``, ``VARCHAR(255)``, …). Known
                names are canonicalized the same way as ``DataType``.

        Returns:
            A new ``CastExpr``.
        """

    def get_expr(self) -> Expr:
        """
        Return the expression being cast.

        Returns:
            The inner expression.
        """

    def get_data_type(self) -> str:
        """
        Return the target SQL type name.

        Returns:
            The canonical type spelling.
        """

@final
class CaseExpr(Expr):
    """
    ``CASE [operand] WHEN … THEN … [ELSE …] END``.

    Build searched or simple ``CASE`` by chaining ``when`` and
    ``else_``. Each call returns a new frozen instance.

    The ``when`` argument must already be a condition expression
    (``BinaryExpr``, ``IsExpr``, …), not an ``Ident`` or ``Value``.
    """

    def __new__(cls, operand: object | None = None, /) -> Self:
        """
        Create an empty ``CASE`` expression.

        Args:
            operand: Optional simple-``CASE`` operand. A bare ``str``
                is a column. Omit for a searched ``CASE``.

        Returns:
            A new ``CaseExpr`` with no ``WHEN`` clauses.
        """

    def when(self, when: Expr, then: object) -> Self:
        """
        Append a ``WHEN … THEN …`` clause.

        Args:
            when: A condition expression (not ``Ident`` or ``Value``).
            then: Result expression. A bare ``str`` is a column.

        Returns:
            A new ``CaseExpr`` with the clause appended.

        Raises:
            TypeError: If ``when`` is not a condition expression.
        """

    def else_(self, result: object) -> Self:
        """
        Set the ``ELSE`` result.

        Args:
            result: The else expression. A bare ``str`` is a column.

        Returns:
            A new ``CaseExpr`` with ``ELSE`` set.
        """

    def get_operand(self) -> Expr | None:
        """
        Return the simple-``CASE`` operand, if any.

        Returns:
            The operand expression, or ``None`` for a searched
            ``CASE``.
        """

    def get_when_list(self) -> tuple[tuple[Expr, Expr], ...]:
        """
        Return the ``WHEN`` / ``THEN`` pairs.

        Returns:
            A tuple of ``(when, then)`` expression pairs.
        """

    def get_else(self) -> Expr | None:
        """
        Return the ``ELSE`` expression, if any.

        Returns:
            The else expression, or ``None``.
        """

@final
class CustomExpr(Expr):
    """
    Escape-hatch SQL template.

    ``{}`` slots are filled by rendering the corresponding argument
    expression. ``{{`` / ``}}`` emit literal braces. Empty ``args``
    means a raw fragment. Argument count must match placeholder count
    when SQL is written.
    """

    def __new__(cls, sql: str, *args: object) -> Self:
        """
        Create a custom SQL fragment.

        Args:
            sql: Template string. Each ``{}`` is replaced by one
                argument. ``{{`` and ``}}`` emit ``{`` and ``}``.
            *args: Expressions interpolated at ``{}`` slots. A bare
                ``str`` is a column.

        Returns:
            A new ``CustomExpr``.
        """

    def get_sql(self) -> str:
        """
        Return the template string.

        Returns:
            The ``sql`` passed to the constructor.
        """

    def get_args(self) -> tuple[Expr, ...]:
        """
        Return the interpolated arguments.

        Returns:
            The argument expressions in slot order.
        """
