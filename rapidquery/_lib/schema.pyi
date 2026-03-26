from __future__ import annotations

import typing

from .common import Column, Expr, ForeignKey, TableName, _ColumnRefNew, _TableNameNew

_IndexColumnValue: typing.TypeAlias = IndexColumn | _ColumnRefNew
_IndexColumnOrder: typing.TypeAlias = typing.Literal["ASC", "DESC"]
_BackendName: typing.TypeAlias = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]

class AlterTable(SchemaStatement):
    """
    Represents an `ALTER TABLE` SQL statement.

    Provides a flexible way to modify existing table structures by applying
    one or more alteration operations such as adding/dropping columns,
    modifying column definitions, or managing constraints.

    Multiple operations can be batched together in a single ALTER TABLE
    statement for efficiency.
    """

    def __init__(
        self,
        name: _TableNameNew,
        options: typing.Iterable[AlterTableBaseOption] = (),
    ) -> None:
        """
        Construct a new `ALTER TABLE` statement.

        Args:
            name: The name of the table to alter.
            options: Iterable of alteration operations to apply.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [
                rq.AlterTableDropColumnOption("updated_at"),
                rq.AlterTableRenameColumnOption("name", "username"),
            ],
        ).to_sql("postgres")
        # ALTER TABLE "users" DROP COLUMN "updated_at", RENAME COLUMN "name" TO "username"
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def add_option(self, opt: AlterTableBaseOption) -> typing.Self:
        """Add an alteration operation."""
        ...

    @property
    def name(self) -> TableName:
        """The name of the table to alter."""
        ...
    @name.setter
    def name(self, value: _TableNameNew) -> None: ...
    @property
    def options(self) -> typing.Sequence[AlterTableBaseOption]:
        """The list of alteration operations to apply."""
        ...
    @options.setter
    def options(self, value: typing.Iterable[AlterTableBaseOption]) -> None: ...

class AlterTableAddColumnOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to add a new column.

    Adds a column to an existing table with optional IF NOT EXISTS clause
    to prevent errors if the column already exists.

    NOTE: this class is immutable and frozen.
    """

    def __init__(self, column: Column, if_not_exists: bool = False) -> None:
        """
        Construct a new `AlterTableAddColumnOption` instance.

        Args:
            column: The column definition to add.
            if_not_exists: If `True`, uses `IF NOT EXISTS` clause.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [
                rq.AlterTableAddColumnOption(
                    rq.Column(
                        "updated_at",
                        rq.sqltypes.Timestamp(timezone=True),
                        default=rq.Expr.current_timestamp(),
                    )
                )
            ],
        ).to_sql("postgres")
        # ALTER TABLE "users" ADD COLUMN "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def column(self) -> Column:
        """The column definition to add."""
        ...

    @property
    def if_not_exists(self) -> bool:
        """Whether to use IF NOT EXISTS clause."""
        ...

class AlterTableAddForeignKeyOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to add a foreign key constraint.

    Adds referential integrity between tables by creating a foreign key
    relationship on an existing table.
    """

    def __init__(self, foreign_key: ForeignKey) -> None:
        """
        Construct a new `AlterTableAddForeignKeyOption` instance.

        Args:
            foreign_key: The foreign key definition to add.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [
                rq.AlterTableAddForeignKeyOption(
                    rq.ForeignKey(["font_id"], ["fonts.id"], on_delete="CASCADE")
                )
            ],
        ).to_sql("postgres")
        # ALTER TABLE "users" ADD CONSTRAINT "fk_fonts_font_id__id" FOREIGN KEY ("font_id") REFERENCES "fonts" ("id") ON DELETE CASCADE
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def foreign_key(self) -> ForeignKey:
        """The foreign key definition to add."""
        ...

class AlterTableBaseOption:
    """
    This abstract base class represents the different types of modifications
    that can be made to an existing table structure, such as adding/dropping
    columns, modifying column definitions, or managing foreign keys.

    NOTE: You cannot use this class as a parent (subclass), but you can use its children
        as a parent (subclass). e.g. `AlterTableDropColumnOption`.
    """

class AlterTableDropColumnOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to drop an existing column.

    Removes a column from the table. This operation may fail if the column
    is referenced by other database objects.
    """

    def __init__(self, name: _ColumnRefNew) -> None:
        """
        Construct a new `AlterTableDropColumnOption` instance.

        Args:
            name: The column name/reference to drop.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [rq.AlterTableDropColumnOption("updated_at")],
        ).to_sql("postgres")
        # ALTER TABLE "users" DROP COLUMN "updated_at"
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def name(self) -> str:
        """The column name to drop."""
        ...

class AlterTableDropForeignKeyOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to drop a foreign key constraint.

    Removes a foreign key relationship by its constraint name.
    """

    def __init__(self, name: str) -> None:
        """
        Construct a new `AlterTableDropForeignKeyOption` instance.

        Args:
            name: The foreign key constraint name to drop.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [rq.AlterTableDropForeignKeyOption("fk_users_id_fonts_id")],
        ).to_sql("postgres")
        # ALTER TABLE "users" DROP CONSTRAINT "fk_users_id_fonts_id"
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def name(self) -> str:
        """The foreign key constraint name to drop."""
        ...

class AlterTableModifyColumnOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to modify a column definition.

    Changes properties of an existing column such as type, nullability,
    default value, or other constraints.
    """

    def __init__(self, column: Column) -> None:
        """
        Construct a new `AlterTableModifyColumnOption` instance.

        Args:
            column: The column you want to modify with changes.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [
                rq.AlterTableModifyColumnOption(
                    rq.Column("updated_at", rq.sqltypes.Date(), default=None)
                )
            ],
        ).to_sql("postgres")
        # ALTER TABLE "users" ALTER COLUMN "updated_at" TYPE date, ALTER COLUMN "updated_at" SET DEFAULT NULL
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def column(self) -> Column: ...

class AlterTableRenameColumnOption(AlterTableBaseOption):
    """
    `ALTER TABLE` operation to rename a column.

    Changes the name of an existing column without modifying its type
    or constraints.
    """

    def __init__(self, from_name: _ColumnRefNew, to_name: _ColumnRefNew) -> None:
        """
        Construct a new `AlterTableRenameColumnOption` instance.

        Args:
            from_name: The column name/reference to rename.
            to_name: New column name.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.AlterTable(
            "users",
            [rq.AlterTableRenameColumnOption("hello", "world")],
        ).to_sql("postgres")
        # ALTER TABLE "users" RENAME COLUMN "hello" TO "world"
        ```
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def from_name(self) -> str:
        """The column name/reference to rename."""
        ...

    @property
    def to_name(self) -> str:
        """New column name."""
        ...

class DropIndex(SchemaStatement):
    """
    Represents a `DROP INDEX` SQL statement.

    Builds index deletion statements with support for:
    - Conditional deletion (IF EXISTS)
    - Table-specific index dropping
    """

    def __init__(
        self,
        name: str,
        table: _TableNameNew,
        if_exists: bool = False,
    ) -> None:
        """
        Construct a new `DROP INDEX` statement.

        Args:
            name: The name of the index to drop.
            table: The table from which to drop the index.
            if_exists: If `True`, uses `IF EXISTS` clause (is not supported by MySQL).

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.DropIndex("idx_glyph_aspect", "glyph").to_sql("mysql")
        # DROP INDEX `idx_glyph_aspect` ON `glyph`
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def if_exists(self) -> bool:
        """Whether to use IF EXISTS clause to avoid errors."""
        ...

    @if_exists.setter
    def if_exists(self, value: bool) -> None: ...
    @property
    def name(self) -> str:
        """The name of the index to drop."""
        ...
    @name.setter
    def name(self, value: str) -> None: ...
    @property
    def table(self) -> TableName:
        """The table from which to drop the index."""
        ...
    @table.setter
    def table(self, value: _TableNameNew) -> None: ...

class DropTable(SchemaStatement):
    """
    Represents a `DROP TABLE` SQL statement.

    Builds table deletion statements with support for:
    - Conditional deletion (IF EXISTS) to avoid errors
    - CASCADE to drop dependent objects
    - RESTRICT to prevent deletion if dependencies exist
    """

    def __init__(
        self,
        name: _TableNameNew,
        *,
        if_exists: bool = False,
        cascade: bool = False,
        restrict: bool = False,
    ) -> None:
        # TODO: complete docstring
        """
        Construct a new `DROP TABLE` statement.

        Args:
            name: The table name to drop.
            if_exists: If `True`, uses `IF EXISTS` clause.
            cascade: ...
            restrict: ...

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.DropTable("glyph", if_exists=True, cascade=True, restrict=True).to_sql(
            "mysql"
        )
        # DROP TABLE IF EXISTS `glyph` RESTRICT CASCADE
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def cascade(self) -> bool: ...
    @cascade.setter
    def cascade(self, value: bool) -> None: ...
    @property
    def if_exists(self) -> bool: ...
    @if_exists.setter
    def if_exists(self, value: bool) -> None: ...
    @property
    def name(self) -> TableName:
        """The table name to drop."""
        ...

    @name.setter
    def name(self, value: _TableNameNew) -> None: ...
    @property
    def restrict(self) -> bool: ...
    @restrict.setter
    def restrict(self, value: bool) -> None: ...

class Index(SchemaStatement):
    """
    Represents a database index specification.

    This class defines the structure and properties of a database index,
    including column definitions, uniqueness constraints, index type,
    and partial indexing conditions.

    You can use it to generate `CREATE INDEX` SQL expressions.
    """

    def __init__(
        self,
        name: str | None,
        columns: typing.Iterable[_IndexColumnValue],
        table: _TableNameNew | None = None,
        *,
        primary: bool = False,
        if_not_exists: bool = False,
        nulls_not_distinct: bool = False,
        unique: bool = False,
        index_type: str | None = None,
        where: Expr | None = None,
        include: typing.Iterable[str] = (),
    ) -> None:
        """
        Construct a new database index specification.
        Also you can use it to generate `CREATE INDEX` SQL expression.

        Args:
            name: The index name. You should always set for indexes that aren't primary or unique, or
                you want use them to generate `CREATE INDEX` statement.
            columns: Column expressions to include in the index. You can use `IndexColumn` class
                    if you want to specify prefix or order.
            table: The table on which to create the index. You should always set for indexes that
                you want use them to generate `CREATE INDEX` statement.
            primary: If `True`, means this is a primary key constraint.
            if_not_exists: If `True`, uses `IF NOT EXISTS` clause.
            nulls_not_distinct: If `True`, NULL values will be considered equal for uniqueness.
            unique: If `True`, means this is a unique key constraint.
            index_type: The type/algorithm for this index. e.g. `"HASH"`, `"BTREE"`, `"FULL TEXT"`.
            where: Condition for partial indexing.
            include: Additional columns to include in the index for covering queries.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.Index("idx_glyph_aspect", ["aspect"], "glyph", if_not_exists=True).to_sql(
            "sqlite"
        )
        # CREATE INDEX IF NOT EXISTS "idx_glyph_aspect" ON "glyph" ("aspect")

        stmt = rq.Index(
            "idx_glyph_aspect", [rq.IndexColumn("aspect", "ASC", 128)], "glyph"
        ).to_sql("mysql")
        # CREATE INDEX `idx_glyph_aspect` ON `glyph` (`aspect` (128) ASC)

        stmt = rq.Index(
            "idx_font_name_include_language",
            ["name"],
            "fonts",
            include=["language"],
            where=rq.Expr.col("aspect").in_([3, 4]),
        ).to_sql("postgresql")
        # CREATE INDEX "idx_font_name_include_language" ON "fonts" ("name") INCLUDE ("language")
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def columns(self) -> typing.Sequence[IndexColumn]:
        """The columns that make up this index."""
        ...
    @columns.setter
    def columns(self, value: typing.Iterable[_IndexColumnValue]) -> None: ...
    @property
    def if_not_exists(self) -> bool:
        """Whether to use IF NOT EXISTS clause."""
        ...
    @if_not_exists.setter
    def if_not_exists(self, value: bool) -> None: ...
    @property
    def include(self) -> typing.Sequence[str]:
        """Additional columns to include in the index for covering queries."""
        ...
    @include.setter
    def include(self, value: typing.Iterable[str]) -> None: ...
    @property
    def index_type(self) -> str | None:
        """The type/algorithm for this index."""
        ...
    @index_type.setter
    def index_type(self, value: str | None) -> None: ...
    @property
    def name(self) -> str | None:
        """Index name"""
        ...
    @name.setter
    def name(self, value: str | None) -> None: ...
    @property
    def nulls_not_distinct(self) -> bool:
        """Whether NULL values should be considered equal for uniqueness."""
        ...
    @nulls_not_distinct.setter
    def nulls_not_distinct(self, value: bool) -> None: ...
    @property
    def primary(self) -> bool:
        """Whether this is a primary key constraint."""
        ...
    @primary.setter
    def primary(self, value: bool) -> None: ...
    @property
    def table(self) -> TableName | None:
        """The table on which to create the index."""
        ...
    @table.setter
    def table(self, value: _TableNameNew | None) -> None: ...
    @property
    def unique(self) -> bool:
        """Whether this is a unique constraint."""
        ...
    @unique.setter
    def unique(self, value: bool) -> None: ...
    @property
    def where(self) -> Expr | None:
        """Condition for partial indexing."""
        ...
    @where.setter
    def where(self, value: object | None) -> None: ...

@typing.final
class IndexColumn:
    """
    Defines a column within an index specification.

    Represents a single column's participation in an index, including:
    - The column name
    - Optional prefix length (for partial indexing)
    - Sort order (ascending or descending)

    Used within `Index` to specify which columns are indexed
    and how they should be ordered.

    NOTE: this class is immutable and frozen.
    """

    def __new__(
        cls,
        name: str,
        order: _IndexColumnOrder | None = None,
        prefix: int | None = None,
    ) -> typing.Self:
        # TODO: complete this docstring
        """
        Construct a new `IndexColumn` instance.

        Args:
            name: The column name.
            order: ...
            prefix: ...
        """
        ...
    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def name(self) -> str:
        """The name of the column to include in the index."""
        ...

    @property
    def order(self) -> _IndexColumnOrder | None:
        """Sort order for this column."""
        ...

    @property
    def prefix(self) -> int | None:
        """Number of characters to index for string columns (prefix indexing)."""
        ...

class RenameTable(SchemaStatement):
    """
    Represents a `RENAME TABLE` SQL statement.

    Changes the name of an existing table to a new name. Both names can be
    schema-qualified if needed.
    """

    def __init__(self, from_name: _TableNameNew, to_name: _TableNameNew) -> None:
        """
        Construct a new `RENAME TABLE` statement.

        Args:
            from_name: The current name of the table.
            to_name: The new name for the table.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.RenameTable("users", "accounts").to_sql("mysql")
        # RENAME TABLE `users` TO `accounts`
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def from_name(self) -> TableName:
        """The current name of the table."""
        ...
    @from_name.setter
    def from_name(self, value: _TableNameNew) -> None: ...
    @property
    def to_name(self) -> TableName:
        """The new name for the table."""
        ...
    @to_name.setter
    def to_name(self, value: _TableNameNew) -> None: ...

class SchemaStatement:
    """Subclass of schema statements."""

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

class Table(SchemaStatement):
    """
    Represents a complete database table definition.

    This class encapsulates all aspects of a table structure including:
    - Column definitions with their types and constraints
    - Indexes for query optimization
    - Foreign key relationships for referential integrity
    - Check constraints for data validation
    - Table-level options like engine, collation, and character set

    Used to generate `CREATE TABLE` SQL statements with full schema specifications.
    """

    def __init__(
        self,
        name: TableName | str,
        *args: Column | Index | ForeignKey | Expr,
        if_not_exists: bool = False,
        temporary: bool = False,
        comment: str | None = None,
        engine: str | None = None,
        collate: str | None = None,
        character_set: str | None = None,
        extra: str | None = None,
    ) -> None:
        """
        Construct a new database table definition. Also you can use it
        to generate `CREATE TABLE` SQL statements with full schema specifications.

        Args:
            name: The name of this table as represented in the database. You can also specify database and schema here.
                If you're using `TableName`, you can use `database` and `schema` parameters; but you can also do it with `str`.
                Format: `"db_name.schema_name.table_name"`.
            args: List of columns (`Column`s), indexes (`Index`s), foreign keys (`ForeignKey`s), or
                check constraints (`Expr`s).
            if_not_exists: If `True`, uses `IF NOT EXISTS` clause.
            temporary: If `True`, marks the table as a temporary table.
            comment: Comment describing the purpose of this table.
            engine: Storage engine for the table (e.g., InnoDB, MyISAM for MySQL).
            collate: Collation for string comparisons and sorting in this table.
            character_set: Character set encoding for text data in this table.
            extra: Additional table-specific options for the statement.

        Examples:
        ```python
        import rapidquery as rq

        table = rq.Table(
            "public.characters",
            rq.Column("id", rq.sqltypes.BigInteger(), primary_key=True, auto_increment=True),
            rq.Column("character", rq.sqltypes.Char(1)),
            rq.Column("font_size", rq.sqltypes.SmallInteger()),
            rq.Column("font_id", rq.sqltypes.BigInteger()),
            rq.ForeignKey(["font_id"], ["fonts.id"], on_delete="CASCADE"),
            rq.Index("ix_characters_fonts_id", ["font_id"]),
            rq.Expr(rq.Func.char_length(rq.Expr.col("character"))) == 1,
            if_not_exists=True,
        )

        table.to_sql("postgres")
        # CREATE TABLE IF NOT EXISTS "public"."characters" (
        #   "id" bigserial PRIMARY KEY NOT NULL,
        #   "character" char(1) NOT NULL,
        #   "font_size" smallint NOT NULL,
        #   "font_id" bigint NOT NULL,
        #   CONSTRAINT "fk_fonts_font_id__id" FOREIGN KEY ("font_id") REFERENCES "fonts" ("id") ON DELETE CASCADE,
        #   CHECK (CHAR_LENGTH("character") = 1)
        # );
        # CREATE INDEX IF NOT EXISTS "ix_characters_fonts_id" ON "public"."characters" ("font_id")
        ```
        """
        ...

    @property
    def __table_name__(self) -> TableName: ...
    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def character_set(self) -> str | None:
        """Character set encoding for text data in this table."""
        ...
    @character_set.setter
    def character_set(self, value: str | None) -> None: ...
    @property
    def checks(self) -> typing.Sequence[Expr]:
        """Table check constraints."""
        ...
    @checks.setter
    def checks(self, value: typing.Iterable[Expr]) -> None: ...
    @property
    def collate(self) -> str | None:
        """Collation for string comparisons and sorting in this table."""
        ...
    @collate.setter
    def collate(self, value: str | None) -> None: ...
    @property
    def columns(self) -> typing.Sequence[Column]:
        """Table columns."""
        ...
    @columns.setter
    def columns(self, value: typing.Iterable[Column]) -> None: ...
    @property
    def comment(self) -> str | None:
        """Comment describing the purpose of this table."""
        ...
    @comment.setter
    def comment(self, value: str | None) -> None: ...
    @property
    def engine(self) -> str | None:
        """Storage engine for the table (e.g., InnoDB, MyISAM for MySQL)."""
        ...
    @engine.setter
    def engine(self, value: str | None) -> None: ...
    @property
    def extra(self) -> str | None:
        """Additional table-specific options for the CREATE TABLE statement."""
        ...
    @extra.setter
    def extra(self, value: str | None) -> None: ...
    @property
    def foreign_keys(self) -> typing.Sequence[ForeignKey]:
        """Table foreign keys."""
        ...
    @foreign_keys.setter
    def foreign_keys(self, value: typing.Iterable[ForeignKey]) -> None: ...
    @property
    def if_not_exists(self) -> bool:
        """Whether to use IF NOT EXISTS clause to avoid errors if table exists."""
        ...
    @if_not_exists.setter
    def if_not_exists(self, value: bool) -> None: ...
    @property
    def indexes(self) -> typing.Sequence[Index]:
        """Table indexes."""
        ...
    @indexes.setter
    def indexes(self, value: typing.Iterable[Index]) -> None: ...
    @property
    def name(self) -> TableName:
        """The name of this table."""
        ...

    @property
    def temporary(self) -> bool:
        """Whether this is a temporary table that exists only for the session."""
        ...
    @temporary.setter
    def temporary(self, value: bool) -> None: ...

class TruncateTable(SchemaStatement):
    """
    Represents a `TRUNCATE TABLE` SQL statement.

    Quickly removes all rows from a table, typically faster than DELETE
    and with different transaction and trigger behavior depending on the
    database system.

    NOTE: SQLite doesn't support TRUNCATE statement.
    """

    def __init__(self, name: _TableNameNew) -> None:
        """
        Construct a new `TRUNCATE TABLE` statement.

        Args:
            name: The name of the table to truncate.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.TruncateTable("users").to_sql("mysql")
        # TRUNCATE TABLE `users`
        ```

        NOTE: SQLite doesn't support TRUNCATE statement.
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def name(self) -> TableName:
        """The name of the table to truncate."""
        ...
    @name.setter
    def name(self, value: _TableNameNew) -> None: ...
