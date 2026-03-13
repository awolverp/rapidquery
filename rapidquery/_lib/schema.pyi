from __future__ import annotations

import typing
from .common import Column, ColumnRef, TableName, ForeignKey, Expr

__all__ = [
    "AlterTable",
    "AlterTableAddColumnOption",
    "AlterTableAddForeignKeyOption",
    "AlterTableBaseOption",
    "AlterTableDropColumnOption",
    "AlterTableDropForeignKeyOption",
    "AlterTableModifyColumnOption",
    "AlterTableRenameColumnOption",
    "DropIndex",
    "DropTable",
    "Index",
    "IndexColumn",
    "RenameTable",
    "SchemaStatement",
    "Table",
    "TruncateTable",
]

_IndexColumnValue: typing.TypeAlias = IndexColumn | Column | ColumnRef | str
_IndexColumnOrder: typing.TypeAlias = typing.Literal["ASC", "DESC"]
_BackendName: typing.TypeAlias = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]

class AlterTable(SchemaStatement):
    """
    Represents an ALTER TABLE SQL statement.

    Provides a flexible way to modify existing table structures by applying
    one or more alteration operations such as adding/dropping columns,
    modifying column definitions, or managing constraints.

    Multiple operations can be batched together in a single ALTER TABLE
    statement for efficiency.
    """

    def __init__(
        self, name: Table | TableName | str, options: typing.Iterable[AlterTableBaseOption] = ()
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def add_option(self, opt: AlterTableBaseOption) -> typing.Self:
        """Add an alteration operation to this ALTER TABLE statement."""
        ...

    @property
    def name(self) -> TableName:
        """The name of the table to alter."""
        ...
    @name.setter
    def name(self, value: Table | TableName | str) -> None: ...
    @property
    def options(self) -> typing.Sequence[AlterTableBaseOption]:
        """The list of alteration operations to apply."""
        ...
    @options.setter
    def options(self, value: typing.Iterable[AlterTableBaseOption]) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

class AlterTableAddColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to add a new column.

    Adds a column to an existing table with optional IF NOT EXISTS clause
    to prevent errors if the column already exists.
    """

    def __init__(self, column: Column, if_not_exists: bool = False) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def column(self) -> Column: ...
    @property
    def if_not_exists(self) -> bool: ...

class AlterTableAddForeignKeyOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to add a foreign key constraint.

    Adds referential integrity between tables by creating a foreign key
    relationship on an existing table.
    """

    def __init__(self, foreign_key: ForeignKey) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def foreign_key(self) -> ForeignKey: ...

class AlterTableBaseOption:
    """
    This abstract base class represents the different types of modifications
    that can be made to an existing table structure, such as adding/dropping
    columns, modifying column definitions, or managing foreign keys.
    """

class AlterTableDropColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to drop an existing column.

    Removes a column from the table. This operation may fail if the column
    is referenced by other database objects.
    """

    def __init__(self, name: Column | ColumnRef | str) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def name(self) -> str: ...

class AlterTableDropForeignKeyOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to drop a foreign key constraint.

    Removes a foreign key relationship by its constraint name.
    """

    def __init__(self, name: ForeignKey | str) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def name(self) -> str: ...

class AlterTableModifyColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to modify a column definition.

    Changes properties of an existing column such as type, nullability,
    default value, or other constraints.
    """

    def __init__(self, column: Column) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def column(self) -> Column: ...

class AlterTableRenameColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to rename a column.

    Changes the name of an existing column without modifying its type
    or constraints.
    """

    def __init__(
        self, from_name: Column | ColumnRef | str, to_name: Column | ColumnRef | str
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def from_name(self) -> str: ...
    @property
    def to_name(self) -> str: ...

class DropIndex(SchemaStatement):
    """
    Represents a DROP INDEX SQL statement.

    Builds index deletion statements with support for:
    - Conditional deletion (IF EXISTS)
    - Table-specific index dropping
    """

    def __init__(self, name: str, table: Table | TableName | str, if_exists: bool = False) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

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
    def table(self, value: Table | TableName | str) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

class DropTable(SchemaStatement):
    """
    Represents a DROP TABLE SQL statement.

    Builds table deletion statements with support for:
    - Conditional deletion (IF EXISTS) to avoid errors
    - CASCADE to drop dependent objects
    - RESTRICT to prevent deletion if dependencies exist
    """

    def __init__(
        self,
        name: Table | TableName | str,
        *,
        if_exists: bool = False,
        cascade: bool = False,
        restrict: bool = False,
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

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
    def name(self, value: Table | TableName | str) -> None: ...
    @property
    def restrict(self) -> bool: ...
    @restrict.setter
    def restrict(self, value: bool) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

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
        columns: typing.Iterable[_IndexColumnValue],
        name: str | None = None,
        table: Table | TableName | str | None = None,
        *,
        primary: bool = False,
        if_not_exists: bool = False,
        nulls_not_distinct: bool = False,
        unique: bool = False,
        index_type: str | None = None,
        where: object | None = None,
        include: typing.Iterable[str] = (),
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

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
    def table(self) -> Table | TableName | None:
        """The table on which to create the index."""
        ...
    @table.setter
    def table(self, value: Table | TableName | str | None) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

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
        self, name: str, order: _IndexColumnOrder | None = None, prefix: int | None = None
    ) -> typing.Self: ...
    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

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
    Represents a RENAME TABLE SQL statement.

    Changes the name of an existing table to a new name. Both names can be
    schema-qualified if needed.
    """

    def __init__(
        self, from_name: Table | TableName | str, to_name: Table | TableName | str
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def from_name(self) -> TableName:
        """The current name of the table."""
        ...
    @from_name.setter
    def from_name(self, value: Table | TableName | str) -> None: ...
    @property
    def to_name(self) -> TableName:
        """The new name for the table."""
        ...
    @to_name.setter
    def to_name(self, value: Table | TableName | str) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

class SchemaStatement:
    """Subclass of schema statements."""

    def __init__(self) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

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

    Used to generate CREATE TABLE SQL statements with full schema specifications.
    """

    def __init__(
        self,
        name: TableName | str,
        *args: Column | Index | ForeignKey | Expr,
        options: int = 0,
        comment: str | None = None,
        engine: str | None = None,
        collate: str | None = None,
        character_set: str | None = None,
        extra: str | None = None,
    ) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

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

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

class TruncateTable(SchemaStatement):
    """
    Represents a TRUNCATE TABLE SQL statement.

    Quickly removes all rows from a table, typically faster than DELETE
    and with different transaction and trigger behavior depending on the
    database system.
    """

    def __init__(self, name: Table | TableName | str) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def name(self) -> TableName:
        """The name of the table to truncate."""
        ...
    @name.setter
    def name(self, value: Table | TableName | str) -> None: ...
    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...
