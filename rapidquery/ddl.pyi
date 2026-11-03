"""Data Definition Language"""

import typing
from collections.abc import Sequence
from typing import Self, final

from rapidquery.dialect import Dialect
from rapidquery.expr import Expr, Ident, _IdentExtractor

class DataDefinitionLanguage:
    """Base of every DDL statement. Construct a subclass, not this class."""

    def to_sql(self, dialect: Dialect) -> str:
        """
        Render ``self`` with values written as SQL literals.

        Args:
            dialect: Dialect that chooses quoting and literal spelling.

        Returns:
            The SQL text.

        Raises:
            RuntimeError: The dialect rejects this construct.
            ValueError: A `CustomExpr` template has a different number of `{}`
                slots than arguments.
        """

@final
class CreateSchema(DataDefinitionLanguage):
    """
    Represents ``CREATE SCHEMA`` statement (and can be ``CREATE DATABASE`` statement).
    """

    def __new__(
        cls,
        name: _IdentExtractor,
        *,
        if_not_exists: bool = False,
        authorization: str | None = None,
        location: str | None = None,
    ) -> Self:
        """
        Construct a new ``CREATE SCHEMA`` statement. If the target dialect specified
        ``OPT_SCHEMA_FEATURES_IS_DATABASE`` option, it generates ``CREATE DATABASE`` instead.

        Args:
            name: schema name e.g. `"analysis"`. Some dialects supports multipart names (like `"public"."analysis"`)
                  so there's no limit and it is in your control.
            if_not_exists: Emits ``IF NOT EXISTS`` clause. Means do nothing if a schema with the same name
                           already exists. *If targeted dialect doesn't support this, it will be ignored.*
            authorization: Emits ``AUTHORIZATION name`` clause. Some dialects allow us to names the owner of the new
                           schema.
            location: Some dialects allow us to specify a location for the new schema. It binds the schema to a directory,
                      which can be a local FS path, HDFS, or even S3.
        """
        ...

    @property
    def name(self) -> Ident: ...
    @property
    def if_not_exists(self) -> bool: ...
    @property
    def authorization(self) -> str | None: ...
    @property
    def location(self) -> str | None: ...

@final
class DropSchema(DataDefinitionLanguage):
    """
    Represents ``DROP SCHEMA`` statement (and can be ``DROP DATABASE`` statement).
    """

    def __new__(
        cls,
        name: _IdentExtractor,
        *,
        if_exists: bool = False,
        behavior: typing.Literal["CASCADE", "RESTRICT"] | str | None = None,
    ) -> Self:
        """
        Construct a new ``DROP SCHEMA`` statement. If the target dialect specified
        ``OPT_SCHEMA_FEATURES_IS_DATABASE`` option, it generates ``DROP DATABASE`` instead.

        Args:
            name: schema name e.g. `"analysis"`. Some dialects supports multipart names (like `"public"."analysis"`)
                  so there's no limit and it is in your control.
            if_exists: Emits ``IF EXISTS`` clause. Means do nothing if the schame doesn't exist.
                       *If targeted dialect doesn't support this, it will be ignored.*
            behavior: Emits ``CASCADE`` or ``RESTRICT`` keywords.
        """
        ...

    @property
    def name(self) -> Ident: ...
    @property
    def if_exists(self) -> bool: ...
    @property
    def behavior(self) -> typing.Literal["CASCADE", "RESTRICT"] | None: ...

@final
class IndexColumn:
    """One key in a ``CREATE INDEX`` column list."""

    def __new__(cls, column: Expr | str, *, order: typing.Literal["ASC", "DESC"] | str = "ASC") -> Self:
        """
        Construct an index key.

        Args:
            column: Column name or expression. A bare string is a column name, not a string literal.
            order: ``ASC`` or ``DESC``. Defaults to ``ASC``.
        """

    @property
    def column(self) -> Expr: ...
    @property
    def order(self) -> typing.Literal["ASC", "DESC"]: ...

@final
class CreateIndex(DataDefinitionLanguage):
    """Represents a ``CREATE INDEX`` statement."""

    def __new__(
        cls,
        name: str,
        table_name: _IdentExtractor,
        columns: Sequence[IndexColumn],
        *,
        unique: bool = False,
        if_not_exists: bool = False,
        include: Sequence[str] | None = None,
        predicate: Expr | str | None = None,
        index_type: str | None = None,
    ) -> Self:
        """
        Construct a ``CREATE INDEX`` statement.

        Args:
            name: Index name. One identifier, not a multipart name.
            table_name: Table the index is created on.
            columns: Index keys. At least one.
            unique: Emits ``UNIQUE``.
            if_not_exists: Emits ``IF NOT EXISTS`` when the dialect supports it. Otherwise the clause is omitted.
            include: Columns for an ``INCLUDE`` clause. Each entry is one identifier.
            predicate: Partial-index ``WHERE`` expression. A bare string is a string literal.
            index_type: Access method written unchanged after ``USING``, such as ``btree`` or ``BTREE``.
        """

    def to_sql(self, dialect: Dialect) -> str:
        """
        Render this statement with values written as SQL literals.

        Raises:
            RuntimeError: The dialect rejects this construct.
        """

    @property
    def name(self) -> str: ...
    @property
    def unique(self) -> bool: ...
    @property
    def if_not_exists(self) -> bool: ...
    @property
    def table_name(self) -> Ident: ...
    @property
    def columns(self) -> list[IndexColumn]: ...
    @property
    def predicate(self) -> Expr | None: ...
    @property
    def include(self) -> list[str]: ...
    @property
    def index_type(self) -> str | None: ...
