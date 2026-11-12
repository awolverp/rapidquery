"""Data definition statements."""

import typing
from collections.abc import Iterable, Sequence
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
    """Column definition inside ``CREATE TABLE``.

    Rendered order is the name, the data type, ``GENERATED ALWAYS AS``,
    ``PRIMARY KEY``, ``UNIQUE``, the dialect auto-increment keyword, ``NULL``
    or ``NOT NULL``, ``DEFAULT``, ``CHECK``, the dialect comment template, and
    ``extra``.

    Rendering the enclosing statement raises ``rapidquery.exc.DialectError``
    when ``generated`` is set and the dialect lacks
    ``OPT_TABLE_FEATURES_GENERATED_SUPPORTED``, or when ``auto_increment`` is
    set and the dialect has no auto-increment template. A comment is omitted
    when the dialect has no column comment template.

    Examples:
        >>> from rapidquery.ddl import Column, CreateTable
        >>> from rapidquery.dialect import PostgreSql
        >>> CreateTable("users", Column("id", "INTEGER", primary_key=True)).to_sql(PostgreSql)
        'CREATE TABLE "users" ("id" INTEGER PRIMARY KEY NOT NULL)'
    """

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
        generated_type: typing.Literal["STORED", "PERSISTED", "VIRTUAL"] | str | None = None,
        *,
        extra: str | None = None,
    ) -> Self:
        """Construct a column definition.

        Args:
            name: Column name. Quoted for the dialect.
            data_type: SQL type name. A known name such as ``integer`` is stored
                as ``INTEGER``. A parameterized name such as ``varchar(16)``
                keeps the caller's spelling.
            primary_key: Write ``PRIMARY KEY``. Implies ``NOT NULL`` unless
                ``nullable`` is set, and drops ``UNIQUE``.
            unique: Write ``UNIQUE``. Stored as false when ``primary_key`` is true.
            nullable: Write ``NULL`` or ``NOT NULL``. When omitted, the column is
                ``NOT NULL`` if it is a primary key or auto-increment, and
                ``NULL`` otherwise.
            default: ``DEFAULT`` expression. ``None`` writes ``NULL``. Omit the
                argument to leave the clause out.
            check: Predicate inside ``CHECK (...)``. ``None`` writes
                ``CHECK (NULL)``. Omit the argument to leave the clause out.
            comment: Inline comment. Omitted when the dialect has no column
                comment template.
            auto_increment: Write the dialect's auto-increment keyword. Rejected
                when the dialect has no template.
            generated: Expression inside ``GENERATED ALWAYS AS (...)``. Rejected
                when the dialect does not support generated columns. ``None``
                writes ``NULL``. Omit the argument to leave the clause out.
                ``generated_type`` is stored only when this argument is passed.
            generated_type: ``VIRTUAL``, ``STORED``, or ``PERSISTED``. Matching
                ignores case.
            extra: Raw SQL appended after the other clauses.

        Raises:
            ValueError: ``generated_type`` is not ``VIRTUAL``, ``STORED``, or
                ``PERSISTED``.
        """

    @property
    def name(self) -> str:
        """Column name."""

    @property
    def data_type(self) -> str:
        """Stored type name. Known names are uppercase."""

    @property
    def primary_key(self) -> bool:
        """Whether the column writes ``PRIMARY KEY``."""

    @property
    def unique(self) -> bool:
        """Whether the column writes ``UNIQUE``."""

    @property
    def nullable(self) -> bool:
        """Whether the column writes ``NULL`` rather than ``NOT NULL``."""

    @property
    def default(self) -> Expr | None:
        """``DEFAULT`` expression, or ``None`` when the argument was omitted."""

    @property
    def check(self) -> Expr | None:
        """``CHECK`` predicate, or ``None`` when the argument was omitted."""

    @property
    def comment(self) -> str | None:
        """Inline comment text, or ``None``."""

    @property
    def auto_increment(self) -> bool:
        """Whether the column writes the dialect auto-increment keyword."""

    @property
    def generated(self) -> Expr | None:
        """``GENERATED ALWAYS AS`` expression, or ``None`` when omitted."""

    @property
    def generated_type(self) -> typing.Literal["STORED", "PERSISTED", "VIRTUAL"] | None:
        """``VIRTUAL``, ``STORED``, ``PERSISTED``, or ``None``."""

    @property
    def extra(self) -> str | None:
        """Trailing raw SQL, or ``None``."""

    @property
    def __expr__(self) -> Expr:
        """Column name as an expression."""

    @property
    def __ident__(self) -> Ident:
        """Column name as an identifier."""

    def to_expr(self) -> Ident:
        """Return the column name as an identifier."""

class Constraint:
    """Base of every table-level constraint. Construct a subclass, not this class.

    Examples:
        >>> from rapidquery.ddl import UniqueConstraint
        >>> UniqueConstraint(["id"], name="users_id_key").name
        'users_id_key'
    """

    @property
    def name(self) -> str | None:
        """Constraint name, or ``None``."""

@final
class UniqueConstraint(Constraint):
    """Table-level ``UNIQUE`` constraint.

    Examples:
        >>> from rapidquery.ddl import UniqueConstraint
        >>> UniqueConstraint("email").columns
        ['email']
    """

    def __new__(cls, *columns: str, name: str | None = None) -> Self:
        """Construct a ``UNIQUE`` constraint.

        Args:
            columns: Constrained column names. At least one.
            name: Optional ``CONSTRAINT`` name.

        Raises:
            ValueError: If `columns` is empty
        """

    @property
    def columns(self) -> Sequence[str]:
        """Constrained column names."""

@final
class PrimaryKeyConstraint(Constraint):
    """Table-level ``PRIMARY KEY`` constraint.

    Examples:
        >>> from rapidquery.ddl import PrimaryKeyConstraint
        >>> PrimaryKeyConstraint(["id"], name="users_pk").columns
        ['id']
    """

    def __new__(cls, *columns: str, name: str | None = None) -> Self:
        """Construct a ``PRIMARY KEY`` constraint.

        Args:
            columns: Constrained column names. At least one.
            name: Optional ``CONSTRAINT`` name.

        Raises:
            ValueError: If `columns` is empty
        """

    @property
    def columns(self) -> Sequence[str]:
        """Constrained column names."""

@final
class ForeignKeyConstraint(Constraint):
    """Table-level ``FOREIGN KEY`` constraint.

    Examples:
        >>> from rapidquery.ddl import ForeignKeyConstraint
        >>> ForeignKeyConstraint(["org_id"], "orgs", on_delete="CASCADE").on_delete
        'CASCADE'
    """

    def __new__(
        cls,
        columns: Iterable[str],
        references: _IdentExtractor,
        ref_columns: Iterable[str] | None = None,
        *,
        name: str | None = None,
        on_delete: typing.Literal["RESTRICT", "CASCADE", "SET NULL", "NO ACTION", "SET DEFAULT"] | str | None = None,
        on_update: typing.Literal["RESTRICT", "CASCADE", "SET NULL", "NO ACTION", "SET DEFAULT"] | str | None = None,
    ) -> Self:
        """Construct a ``FOREIGN KEY`` constraint.

        Args:
            columns: Constrained column names. At least one.
            references: Referenced table. A multipart name is allowed.
            name: Optional ``CONSTRAINT`` name.
            ref_columns: Referenced column names. At least one.
            on_delete: ``ON DELETE`` action.
            on_update: ``ON UPDATE`` action.
        """

    @property
    def columns(self) -> Sequence[str]:
        """Constrained column names."""

    @property
    def references(self) -> Ident:
        """Referenced table."""

    @property
    def ref_columns(self) -> Sequence[str]:
        """Referenced column names. Empty when the list is omitted."""

    @property
    def on_delete(self) -> typing.Literal["RESTRICT", "CASCADE", "SET NULL", "NO ACTION", "SET DEFAULT"] | None:
        """``ON DELETE`` action, or ``None``."""

    @property
    def on_update(self) -> typing.Literal["RESTRICT", "CASCADE", "SET NULL", "NO ACTION", "SET DEFAULT"] | None:
        """``ON UPDATE`` action, or ``None``."""

@final
class CheckConstraint(Constraint):
    """Table-level ``CHECK`` constraint.

    Examples:
        >>> from rapidquery.ddl import CheckConstraint
        >>> from rapidquery.expr import Ident
        >>> CheckConstraint(Ident("active")).expression
        Ident("active")
    """

    def __new__(cls, expression: _LiteralExprExtractor, *, name: str | None = None) -> Self:
        """Construct a ``CHECK`` constraint.

        Args:
            expression: Predicate inside ``CHECK (...)``.
            name: Optional ``CONSTRAINT`` name.
        """

    @property
    def expression(self) -> Expr:
        """Predicate inside ``CHECK (...)``."""

@final
class CreateTable(DataDefinitionLanguage):
    """``CREATE TABLE`` statement.

    ``temporary=True`` writes the dialect's session-private keyword
    (``TEMPORARY``, ``VOLATILE``, or ``PRIVATE TEMPORARY``). SQL Server has no
    such keyword, so ``temporary=True`` is rejected there.

    Examples:
        >>> from rapidquery.ddl import Column, CreateTable
        >>> from rapidquery.dialect import PostgreSql
        >>> CreateTable("users", [Column("id", "INTEGER", nullable=False)]).to_sql(PostgreSql)
        'CREATE TABLE "users" ("id" INTEGER NOT NULL)'
    """

    def __new__(
        cls,
        name: _IdentExtractor,
        *args: Column | Constraint,
        if_not_exists: bool = False,
        temporary: bool = False,
        on_commit: typing.Literal["PRESERVE ROWS", "DELETE ROWS", "DROP"] | str | None = None,
        external: bool = False,
        comment: str | None = None,
        location: str | None = None,
        extra: str | None = None,
    ) -> Self:
        """Construct a ``CREATE TABLE`` statement.

        Args:
            name: Table name. A multipart name is allowed.
            args: Column definitions and table-level constraints. At least one ``Column`` is required.
            if_not_exists: Write ``IF NOT EXISTS``. The clause is omitted when
                the dialect does not support it.
            temporary: Write the dialect's session-private temporary keyword.
            on_commit: ``PRESERVE ROWS``, ``DELETE ROWS``, or ``DROP``. Requires
                ``temporary=True``.
            external: Write ``EXTERNAL`` before ``TABLE``.
            comment: Table comment. Omitted when the dialect has no comment template.
            location: External location. Rejected when the dialect has no location
                template.
            extra: Raw SQL appended after the other clauses. Use this for
                partitioning, distribution, and other dialect-specific tails.

        Note:
            ``OR REPLACE``, ``CREATE TABLE ... AS``, ``LIKE``, ``CLONE``, ``OF``,
            collation, and character set are not supported.
        """

    @property
    def name(self) -> Ident:
        """Table name."""

    @property
    def columns(self) -> Sequence[Column]:
        """Column definitions."""

    @property
    def constraints(self) -> Sequence[Constraint]:
        """Table-level constraints."""

    @property
    def if_not_exists(self) -> bool:
        """Whether the statement requested ``IF NOT EXISTS``."""

    @property
    def temporary(self) -> bool:
        """Whether the statement is a session-private temporary table."""

    @property
    def on_commit(self) -> typing.Literal["PRESERVE ROWS", "DELETE ROWS", "DROP"] | None:
        """``ON COMMIT`` action, or ``None``."""

    @property
    def external(self) -> bool:
        """Whether the statement writes ``EXTERNAL``."""

    @property
    def comment(self) -> str | None:
        """Table comment, or ``None``."""

    @property
    def location(self) -> str | None:
        """Location value, or ``None``."""

    @property
    def extra(self) -> str | None:
        """Trailing raw SQL, or ``None``."""
