"""Data definition statements."""

import typing
from collections.abc import Sequence
from typing import Self, final

from rapidquery.dialect import Dialect
from rapidquery.expr import Expr, Ident, _IdentExtractor, _LiteralExprExtractor, _NameExprExtractor

_IndexColumnExtractor: typing.TypeAlias = _NameExprExtractor | IndexColumn
"""IndexColumn extractor.

- `str` becomes `IndexColumn(Ident(val), "ASC")`
- Others becomes `Expr` and then `IndexColumn(expr, "ASC")`
"""

class DataDefinitionLanguage:
    """Base of every DDL statement. Construct a subclass, not this class.

    Examples:
        >>> from rapidquery.ddl import CreateSchema
        >>> from rapidquery.dialect import PostgreSql
        >>> CreateSchema("app").to_sql(PostgreSql)
        'CREATE SCHEMA "app"'
    """

    def to_sql(self, dialect: Dialect) -> str:
        """Render this statement with values written as SQL literals.

        Args:
            dialect: Dialect that chooses quoting and literal spelling.

        Returns:
            The SQL text.

        Raises:
            rapidquery.exc.DialectError: The dialect cannot render this construct.
            ValueError: A `CustomExpr` template has a different number of `{}`
                slots than arguments.
        """

@final
class CreateSchema(DataDefinitionLanguage):
    """``CREATE SCHEMA`` statement.

    If the dialect sets ``OPT_SCHEMA_FEATURES_IS_DATABASE``, ``to_sql`` writes
    ``CREATE DATABASE`` instead.

    Examples:
        >>> from rapidquery.ddl import CreateSchema
        >>> from rapidquery.dialect import PostgreSql
        >>> CreateSchema("app").to_sql(PostgreSql)
        'CREATE SCHEMA "app"'
    """

    def __new__(
        cls,
        name: _IdentExtractor,
        *,
        if_not_exists: bool = False,
        authorization: str | None = None,
        location: str | None = None,
    ) -> Self:
        """Construct a ``CREATE SCHEMA`` statement.

        If the dialect sets ``OPT_SCHEMA_FEATURES_IS_DATABASE``, this writes
        ``CREATE DATABASE`` instead.

        Args:
            name: Schema name, such as ``"analysis"``. A multipart name such as
                ``"public"."analysis"`` is allowed.
            if_not_exists: Write ``IF NOT EXISTS``. The clause is omitted when
                the dialect does not support it.
            authorization: Write ``AUTHORIZATION name``. The clause names the
                owner of the new schema.
            location: Bind the schema to a directory. The path may be a local
                filesystem path, HDFS, or S3. The dialect escapes the value.
        """
        ...

    @property
    def name(self) -> Ident:
        """Schema name."""

    @property
    def if_not_exists(self) -> bool:
        """Whether the statement writes ``IF NOT EXISTS``."""

    @property
    def authorization(self) -> str | None:
        """Owner name, or ``None``."""

    @property
    def location(self) -> str | None:
        """Location value, or ``None``."""

@final
class DropSchema(DataDefinitionLanguage):
    """``DROP SCHEMA`` statement.

    If the dialect sets ``OPT_SCHEMA_FEATURES_IS_DATABASE``, ``to_sql`` writes
    ``DROP DATABASE`` instead.

    Examples:
        >>> from rapidquery.ddl import DropSchema
        >>> from rapidquery.dialect import PostgreSql
        >>> DropSchema("app").to_sql(PostgreSql)
        'DROP SCHEMA "app"'
    """

    def __new__(
        cls,
        name: _IdentExtractor,
        *,
        if_exists: bool = False,
        behavior: typing.Literal["CASCADE", "RESTRICT"] | str | None = None,
    ) -> Self:
        """Construct a ``DROP SCHEMA`` statement.

        If the dialect sets ``OPT_SCHEMA_FEATURES_IS_DATABASE``, this writes
        ``DROP DATABASE`` instead.

        Args:
            name: Schema name, such as ``"analysis"``. A multipart name is
                allowed.
            if_exists: Write ``IF EXISTS``. The clause is omitted when the
                dialect does not support it.
            behavior: Write ``CASCADE`` or ``RESTRICT``.
        """
        ...

    @property
    def name(self) -> Ident:
        """Schema name."""

    @property
    def if_exists(self) -> bool:
        """Whether the statement writes ``IF EXISTS``."""

    @property
    def behavior(self) -> typing.Literal["CASCADE", "RESTRICT"] | None:
        """``CASCADE``, ``RESTRICT``, or ``None``."""

@final
class IndexColumn:
    """One key in a ``CREATE INDEX`` column list.

    Examples:
        >>> from rapidquery.ddl import IndexColumn
        >>> IndexColumn("id", order="DESC").order
        'DESC'
    """

    def __new__(cls, column: _NameExprExtractor, *, order: typing.Literal["ASC", "DESC"] | str = "ASC") -> Self:
        """Construct an index key.

        Args:
            column: Column name or expression. A bare string is a column name,
                not a string literal.
            order: ``ASC`` or ``DESC``. Defaults to ``ASC``.
        """

    @property
    def column(self) -> Expr:
        """Indexed expression."""

    @property
    def order(self) -> typing.Literal["ASC", "DESC"]:
        """``ASC`` or ``DESC``."""

@final
class CreateIndex(DataDefinitionLanguage):
    """``CREATE INDEX`` statement.

    Examples:
        >>> from rapidquery.ddl import CreateIndex, IndexColumn
        >>> from rapidquery.dialect import PostgreSql
        >>> CreateIndex("idx", "users", [IndexColumn("id")]).to_sql(PostgreSql)
        'CREATE INDEX "idx" ON "users" ("id" ASC)'
    """

    def __new__(
        cls,
        name: str,
        table_name: _IdentExtractor,
        columns: typing.Iterable[_IndexColumnExtractor],
        *,
        unique: bool = False,
        if_not_exists: bool = False,
        include: typing.Iterable[str] | None = None,
        predicate: Expr | str | None = None,
        index_type: str | None = None,
    ) -> Self:
        """Construct a ``CREATE INDEX`` statement.

        Args:
            name: Index name. One identifier, not a multipart name.
            table_name: Table the index is created on.
            columns: Index keys. At least one.
            unique: Write ``UNIQUE``.
            if_not_exists: Write ``IF NOT EXISTS`` when the dialect supports it.
                Otherwise the clause is omitted.
            include: Columns for an ``INCLUDE`` clause. Each entry is one identifier.
            predicate: Partial-index ``WHERE`` expression. A bare string is a string literal.
            index_type: Access method written unchanged after ``USING``, such as
                ``btree`` or ``BTREE``.
        """

    def to_sql(self, dialect: Dialect) -> str:
        """Render this statement with values written as SQL literals.

        Args:
            dialect: Dialect that chooses quoting and literal spelling.

        Returns:
            The SQL text.

        Raises:
            rapidquery.exc.DialectError: The dialect cannot render this construct,
                including a clause its flags reject.
            ValueError: A required part is missing or empty.
        """

    @property
    def name(self) -> str:
        """Index name."""

    @property
    def unique(self) -> bool:
        """Whether the statement writes ``UNIQUE``."""

    @property
    def if_not_exists(self) -> bool:
        """Whether the statement writes ``IF NOT EXISTS``."""

    @property
    def table_name(self) -> Ident:
        """Table the index is created on."""

    @property
    def columns(self) -> Sequence[IndexColumn]:
        """Index keys."""

    @property
    def predicate(self) -> Expr | None:
        """Partial-index predicate, or ``None``."""

    @property
    def include(self) -> list[str]:
        """``INCLUDE`` column names."""

    @property
    def index_type(self) -> str | None:
        """Access method, or ``None``."""

@final
class DropIndex(DataDefinitionLanguage):
    """``DROP INDEX`` statement.

    Examples:
        >>> from rapidquery.ddl import DropIndex
        >>> from rapidquery.dialect import PostgreSql
        >>> DropIndex("idx").to_sql(PostgreSql)
        'DROP INDEX "idx"'
    """

    def __new__(
        cls,
        name: str,
        *,
        table_name: _IdentExtractor | None = None,
        if_exists: bool = False,
        behavior: typing.Literal["CASCADE", "RESTRICT"] | str | None = None,
    ) -> Self:
        """Construct a ``DROP INDEX`` statement.

        Args:
            name: Index name. One identifier, not a multipart name.
            table_name: Table that owns the index. Required when the dialect writes
                ``ON table`` or ``table.name``. Ignored when the dialect writes the name alone.
            if_exists: Write ``IF EXISTS`` when the dialect supports ``IF NOT EXISTS``.
                Otherwise the clause is omitted.
            behavior: Write ``CASCADE`` or ``RESTRICT``.
        """

    @property
    def name(self) -> str:
        """Index name."""

    @property
    def table_name(self) -> Ident | None:
        """Owning table, or ``None``."""

    @property
    def if_exists(self) -> bool:
        """Whether the statement writes ``IF EXISTS``."""

    @property
    def behavior(self) -> typing.Literal["CASCADE", "RESTRICT"] | None:
        """``CASCADE``, ``RESTRICT``, or ``None``."""

@final
class Column:
    # TODO: update docstrings
    """Represents column definition"""

    def __new__(
        cls,
        name: str,
        data_type: str,
        /,
        primary_key: bool = False,
        unique: bool = False,
        nullable: bool | None = None,
        default: _LiteralExprExtractor = ...,
        check: _LiteralExprExtractor = ...,
        comment: str | None = None,
        auto_increment: bool = False,
        generated: _LiteralExprExtractor = ...,
        generated_type: typing.Literal["STORED", "PERSIST", "VIRTUAL"] | str | None = None,
        *,
        extra: str | None = None,
    ) -> typing.Self: ...
    @property
    def name(self) -> str: ...
    @property
    def data_type(self) -> str: ...
    @property
    def primary_key(self) -> bool: ...
    @property
    def unique(self) -> bool: ...
    @property
    def nullable(self) -> bool: ...
    @property
    def default(self) -> Expr | None: ...
    @property
    def check(self) -> Expr | None: ...
    @property
    def comment(self) -> str | None: ...
    @property
    def auto_increment(self) -> bool: ...
    @property
    def generated(self) -> Expr | None: ...
    @property
    def generated_type(self) -> typing.Literal["STORED", "PERSIST", "VIRTUAL"] | None: ...
    @property
    def extra(self) -> str | None: ...
    @property
    def __expr__(self) -> Expr: ...
    @property
    def __ident__(self) -> Ident: ...
    def to_expr(self) -> Ident: ...
    def to_sql(self, dialect: Dialect) -> str: ...
