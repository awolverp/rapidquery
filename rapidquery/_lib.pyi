"""
RapidQuery core module written in Rust
"""

from __future__ import annotations

import typing
import decimal
import uuid
import datetime
import enum

__all__ = [
    "ASTERISK",
    "AlterTable",
    "AlterTableAddColumnOption",
    "AlterTableAddForeignKeyOption",
    "AlterTableBaseOption",
    "AlterTableDropColumnOption",
    "AlterTableDropForeignKeyOption",
    "AlterTableModifyColumnOption",
    "AlterTableRenameColumnOption",
    "ArrayType",
    "BigIntegerType",
    "BigUnsignedType",
    "BinaryType",
    "BitType",
    "BlobType",
    "BooleanType",
    "CharType",
    "Column",
    "ColumnRef",
    "DateTimeType",
    "DateType",
    "DecimalType",
    "Delete",
    "DoubleType",
    "DropIndex",
    "DropTable",
    "EnumType",
    "Expr",
    "FloatType",
    "ForeignKey",
    "Func",
    "INETType",
    "Index",
    "IndexColumn",
    "Insert",
    "IntegerType",
    "JSONBinaryType",
    "JSONType",
    "MacAddressType",
    "OnConflict",
    "QueryStatement",
    "RenameTable",
    "SQLTypeAbstract",
    "SchemaStatement",
    "SmallIntegerType",
    "SmallUnsignedType",
    "StringType",
    "Table",
    "TableName",
    "TextType",
    "TimeType",
    "TimestampType",
    "TinyIntegerType",
    "TinyUnsignedType",
    "TruncateTable",
    "UUIDType",
    "UnsignedType",
    "Update",
    "Value",
    "VarBinaryType",
    "VarBitType",
    "VectorType",
    "_AsteriskType",
    "all",
    "any",
    "not_",
]

I = typing.TypeVar("I")
O = typing.TypeVar("O")
_ForeignKeyActions: typing.TypeAlias = typing.Literal[
    "CASCADE", "RESTRICT", "NO ACTION", "SET DEFAULT", "SET NULL"
]
_IndexColumnValue: typing.TypeAlias = IndexColumn | Column | ColumnRef | str
_IndexColumnOrder: typing.TypeAlias = typing.Literal["ASC", "DESC"]
_BackendName: typing.TypeAlias = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]

ASTERISK: typing.Final[_AsteriskType] = ...

@typing.final
class AlterTable(SchemaStatement):
    """
    Represents an ALTER TABLE SQL statement.

    Provides a flexible way to modify existing table structures by applying
    one or more alteration operations such as adding/dropping columns,
    modifying column definitions, or managing constraints.

    Multiple operations can be batched together in a single ALTER TABLE
    statement for efficiency.
    """

    def __new__(
        cls, name: Table | TableName | str, options: typing.Iterable[AlterTableBaseOption] = ()
    ) -> typing.Self: ...
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

@typing.final
class AlterTableAddColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to add a new column.

    Adds a column to an existing table with optional IF NOT EXISTS clause
    to prevent errors if the column already exists.
    """

    def __new__(cls, column: Column, if_not_exists: bool = False) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def column(self) -> Column: ...
    @property
    def if_not_exists(self) -> bool: ...

@typing.final
class AlterTableAddForeignKeyOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to add a foreign key constraint.

    Adds referential integrity between tables by creating a foreign key
    relationship on an existing table.
    """

    def __new__(cls, foreign_key: ForeignKey) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def foreign_key(self) -> ForeignKey: ...

class AlterTableBaseOption:
    """
    Base class for all ALTER TABLE operation types.

    This abstract base class represents the different types of modifications
    that can be made to an existing table structure, such as adding/dropping
    columns, modifying column definitions, or managing foreign keys.
    """

@typing.final
class AlterTableDropColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to drop an existing column.

    Removes a column from the table. This operation may fail if the column
    is referenced by other database objects.
    """

    def __new__(cls, name: Column | ColumnRef | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def name(self) -> str: ...

@typing.final
class AlterTableDropForeignKeyOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to drop a foreign key constraint.

    Removes a foreign key relationship by its constraint name.
    """

    def __new__(cls, name: ForeignKey | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def name(self) -> str: ...

@typing.final
class AlterTableModifyColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to modify a column definition.

    Changes properties of an existing column such as type, nullability,
    default value, or other constraints.
    """

    def __new__(cls, column: Column) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def column(self) -> Column: ...

@typing.final
class AlterTableRenameColumnOption(AlterTableBaseOption):
    """
    ALTER TABLE operation to rename a column.

    Changes the name of an existing column without modifying its type
    or constraints.
    """

    def __new__(
        cls, from_name: Column | ColumnRef | str, to_name: Column | ColumnRef | str
    ) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def from_name(self) -> str: ...
    @property
    def to_name(self) -> str: ...

@typing.final
class ArrayType(SQLTypeAbstract[list[I] | tuple[I], list[O]]):
    """
    Array column type for storing arrays of elements.

    Represents a column that stores arrays of a specified element type.
    Useful in databases that support native array types (like PostgreSQL)
    for storing lists of values in a single column.
    """

    def __new__(cls, element: SQLTypeAbstract[I, O]) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def element(self) -> SQLTypeAbstract[I, O]: ...

@typing.final
class BigIntegerType(SQLTypeAbstract[int, int]):
    """
    Large integer column type (BIGINT).

    Stores 64-bit integers for very large numeric values. Essential for
    high-volume systems, timestamps, large counters, or when integer
    overflow is a concern.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class BigUnsignedType(SQLTypeAbstract[int, int]):
    """
    Unsigned big integer column type.

    Stores very large positive integers only. Provides the maximum positive
    integer range for high-volume systems or when very large positive
    values are required.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class BinaryType(SQLTypeAbstract[bytes, bytes]):
    """
    Fixed-length binary data column type (BINARY).

    Stores binary data of a fixed length. Values shorter than the specified
    length are padded. Useful for storing hashes, keys, or other binary
    data with consistent length.
    """

    def __new__(cls, length: int = 255) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class BitType(SQLTypeAbstract[bytes, bytes]):
    """
    Fixed-length bit string column type (BIT).

    Stores a fixed number of bits. Useful for storing boolean flags efficiently
    or binary data where individual bits have meaning.
    """

    def __new__(cls, length: int) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class BlobType(SQLTypeAbstract[bytes, bytes]):
    """
    Binary large object column type (BLOB).

    Stores large binary data such as images, documents, audio files, or
    any binary content. Size limits vary by database system.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class BooleanType(SQLTypeAbstract[bool, bool]):
    """
    Boolean column type (BOOLEAN).

    Stores true/false values. The standard way to store boolean data,
    though implementation varies by database (some use TINYINT(1) or
    similar representations).
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class CharType(SQLTypeAbstract[str, str]):
    """
    Fixed-length character string column type (CHAR).

    Represents a fixed-length character string. Values shorter than the
    specified length are padded with spaces. Suitable for storing data
    with consistent, known lengths like country codes or status flags.
    """

    def __new__(cls, length: int | None = ...) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...

@typing.final
class Column(typing.Generic[I, O]):
    """
    Defines a table column with its properties and constraints.

    Represents a complete column definition including:
    - Column name and data type
    - Constraints (primary key, unique, nullable)
    - Auto-increment behavior
    - Default values and generated columns
    - Comments and extra specifications

    This class is used within Table to specify the structure
    of table columns. It encapsulates all the properties that define how
    a column behaves and what data it can store.
    """

    def __new__(
        cls,
        name: str,
        type: SQLTypeAbstract[I, O],
        options: int = ...,
        *,
        extra: str | None = ...,
        comment: str | None = ...,
        default: typing.Any = ...,
        generated: typing.Any = ...,
    ) -> typing.Self: ...
    OPT_AUTO_INCREMENT: typing.ClassVar[int] = ...
    OPT_NULLABLE: typing.ClassVar[int] = ...
    OPT_PRIMARY_KEY: typing.ClassVar[int] = ...
    OPT_STORED_GENERATED: typing.ClassVar[int] = ...
    OPT_UNIQUE_KEY: typing.ClassVar[int] = ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def adapt(self, object: I) -> O:
        """Shorthand for `Value(object, self.type)`."""
        ...

    @property
    def comment(self) -> str | None:
        """Comment describing this column."""
        ...
    @comment.setter
    def comment(self, value: str | None) -> None: ...
    @property
    def default(self) -> Expr | None:
        """Default value for this column."""
        ...
    @default.setter
    def default(self, value: Expr | None) -> None: ...
    @property
    def extra(self) -> str | None:
        """Extra SQL specifications for this column."""
        ...
    @extra.setter
    def extra(self, value: str | None) -> None: ...
    @property
    def generated(self) -> Expr | None:
        """Expression for generated column values."""
        ...
    @generated.setter
    def generated(self, value: Expr | None) -> None: ...
    @property
    def is_auto_increment(self) -> bool:
        """Shorthand for `self.options & OPT_AUTO_INCREMENT > 0`."""
        ...

    @property
    def is_nullable(self) -> bool:
        """Shorthand for `self.options & OPT_NULLABLE > 0`."""
        ...

    @property
    def is_primary_key(self) -> bool:
        """Shorthand for `self.options & OPT_PRIMARY_KEY > 0`."""
        ...

    @property
    def is_stored_generated(self) -> bool:
        """Shorthand for `self.options & OPT_STORED_GENERATED > 0`."""
        ...

    @property
    def is_unique_key(self) -> bool:
        """Shorthand for `self.options & OPT_UNIQUE_KEY > 0`."""
        ...

    @property
    def name(self) -> str:
        """Column name."""
        ...

    @property
    def options(self) -> int:
        """Column specified options."""
        ...
    @options.setter
    def options(self, value: int) -> None: ...
    @property
    def type(self) -> SQLTypeAbstract[I, O]:
        """Column type."""
        ...

@typing.final
class ColumnRef:
    """
    Represents a reference to a database column with optional table and schema qualification.

    This class is used to uniquely identify columns in SQL queries, supporting
    schema-qualified and table-qualified column references.

    NOTE: this class is immutable and frozen.
    """

    def __new__(
        cls, name: str | _AsteriskType, table: str | None = ..., schema: str | None = ...
    ) -> typing.Self: ...
    def __copy__(self) -> typing.Self: ...
    def __eq__(self, value, /) -> bool:
        """Return self==value."""
        ...

    def __ge__(self, value, /) -> bool:
        """Return self>=value."""
        ...

    def __gt__(self, value, /) -> bool:
        """Return self>value."""
        ...

    def __le__(self, value, /) -> bool:
        """Return self<=value."""
        ...

    def __lt__(self, value, /) -> bool:
        """Return self<value."""
        ...

    def __ne__(self, value, /) -> bool:
        """Return self!=value."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def copy_with(
        self,
        *,
        name: str | _AsteriskType | None = ...,
        table: str | None = ...,
        schema: str | None = ...,
    ) -> typing.Self: ...
    @property
    def name(self) -> str: ...
    @classmethod
    def parse(cls, string: str) -> typing.Self:
        """
        Parse a string representation of a column reference.

        Supports formats like:
        - "column_name"
        - "table.column_name"
        - "schema.table.column_name"
        """
        ...

    @property
    def schema(self) -> str | None: ...
    @property
    def table(self) -> str | None: ...

    __hash__ = None  # type: ignore

@typing.final
class DateTimeType(SQLTypeAbstract[datetime.datetime, datetime.datetime]):
    """
    Date and time column type (DATETIME).

    Stores both date and time information without timezone awareness.
    Suitable for recording timestamps, event times, or scheduling information
    when timezone handling is managed at the application level.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class DateType(SQLTypeAbstract[datetime.date, datetime.date]):
    """
    Date-only column type (DATE).

    Stores date information without time component. Ideal for birth dates,
    deadlines, or any date-based data where time precision is not needed.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class DecimalType(SQLTypeAbstract[decimal.Decimal | int | float | str, decimal.Decimal]):
    """
    Exact numeric decimal column type (DECIMAL/NUMERIC).

    Stores exact numeric values with fixed precision and scale. Essential for
    financial calculations, currency values, or any situation where exact
    decimal representation is required without floating-point approximation.
    """

    def __new__(cls, context: tuple[int, int] | None = None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def context(self) -> tuple[int, int] | None: ...

@typing.final
class Delete(QueryStatement):
    """
    Builds DELETE SQL statements with a fluent interface.

    Provides a chainable API for constructing DELETE queries with support for:
    - WHERE conditions for filtering
    - LIMIT for restricting deletion count
    - ORDER BY for determining deletion order
    - RETURNING clauses for getting deleted data
    """

    def __new__(cls, table: Table | TableName | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: Table | TableName | str) -> typing.Self:
        """Specify the table to delete from."""
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to delete."""
        ...

    def order_by(
        self,
        target: Expr | Column | ColumnRef | str,
        order: typing.Literal["ASC", "DESC"] = "ASC",
        null_ordering: typing.Literal["FIRST", "LAST"] | None = None,
    ) -> typing.Self:
        """
        Specify the order in which to delete rows. Typically used with
        `.limit` method to delete specific rows.
        """
        ...

    def returning(self, *args: Column | ColumnRef | str) -> typing.Self:
        """Specify columns to return from the deleted rows."""
        ...

    def returning_all(self) -> typing.Self:
        """
        Return all columns from the deleted rows. Same as `self.returning("*")`.
        """
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """Add a WHERE condition to filter rows to delete."""
        ...

@typing.final
class DoubleType(SQLTypeAbstract[float | int, float]):
    """
    Double-precision floating point column type (DOUBLE).

    Stores approximate numeric values with double precision. Provides higher
    precision than FLOAT for scientific calculations or when more accuracy
    is required in floating-point operations.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class DropIndex(SchemaStatement):
    """
    Represents a DROP INDEX SQL statement.

    Builds index deletion statements with support for:
    - Conditional deletion (IF EXISTS)
    - Table-specific index dropping
    """

    def __new__(
        cls, name: str, table: Table | TableName | str, if_exists: bool = False
    ) -> typing.Self: ...
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

@typing.final
class DropTable(SchemaStatement):
    """
    Represents a DROP TABLE SQL statement.

    Builds table deletion statements with support for:
    - Conditional deletion (IF EXISTS) to avoid errors
    - CASCADE to drop dependent objects
    - RESTRICT to prevent deletion if dependencies exist
    """

    def __new__(cls, name: Table | TableName | str, options: int = 0) -> typing.Self: ...
    OPT_CASCADE: typing.ClassVar[int] = ...
    OPT_IF_EXISTS: typing.ClassVar[int] = ...
    OPT_RESTRICT: typing.ClassVar[int] = ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def cascade(self) -> bool:
        """Shorthand for `self.options & OPT_CASCADE > 0`."""
        ...

    @property
    def if_exists(self) -> bool:
        """Shorthand for `self.options & OPT_IF_EXISTS > 0`."""
        ...

    @property
    def name(self) -> TableName:
        """The table name to drop."""
        ...
    @name.setter
    def name(self, value: Table | TableName | str) -> None: ...
    @property
    def options(self) -> int:
        """Specified options."""
        ...
    @options.setter
    def options(self, value: int) -> None: ...
    @property
    def restrict(self) -> bool:
        """Shorthand for `self.options & OPT_RESTRICT > 0`."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

@typing.final
class EnumType(SQLTypeAbstract[str | enum.Enum, str]):
    """
    Enumeration column type (ENUM).

    Stores one value from a predefined set of allowed string values.
    Provides type safety and storage efficiency for categorical data
    with a fixed set of possible values.
    """

    def __new__(cls, name: str, variants: typing.Iterable[str]) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def name(self) -> str: ...
    @property
    def variants(self) -> typing.Sequence[str]: ...

@typing.final
class Expr:
    """
    Represents a SQL expression that can be built into SQL code.

    This class provides a fluent interface for constructing complex SQL expressions
    in a database-agnostic way. It supports arithmetic operations, comparisons,
    logical operations, and database-specific functions.

    The class automatically handles SQL injection protection and proper quoting
    when building the final SQL statement.

    NOTE: `Expr` is immutable, so by calling each method you will give a new instance
    of it which includes new change(s).
    """

    def __new__(cls, value, /) -> typing.Self: ...
    def __add__(self, other: object) -> typing.Self:
        """Create an addition expression."""
        ...

    def __and__(self, other: object) -> typing.Self:
        """Create a logical AND expression."""
        ...

    def __eq__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create an equality comparison expression."""
        ...

    def __ge__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create a greater-than-or-equal comparison expression."""
        ...

    def __gt__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create a greater-than comparison expression."""
        ...

    def __le__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create a less-than-or-equal comparison expression."""
        ...

    def __lshift__(self, other: object) -> typing.Self:
        """Create a bitwise left-shift expression."""
        ...

    def __lt__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create a less-than comparison expression."""
        ...

    def __mod__(self, other: object) -> typing.Self:
        """Create a modulo expression."""
        ...

    def __mul__(self, other: object) -> typing.Self:
        """Create a multiplication expression."""
        ...

    def __ne__(self, other: object) -> typing.Self:  # type: ignore[override]
        """Create an inequality comparison expression."""
        ...

    def __neg__(self) -> typing.Self:
        """Create a negation expression."""
        ...

    def __or__(self, other: object) -> typing.Self:
        """Create a logical OR expression."""
        ...

    def __radd__(self, other: object) -> typing.Self:
        """Create an addition expression."""
        ...

    def __rand__(self, other: object) -> typing.Self:
        """Create a logical AND expression."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def __rlshift__(self, other: object) -> typing.Self:
        """Create a bitwise left-shift expression."""
        ...

    def __rmod__(self, other: object) -> typing.Self:
        """Create a modulo expression."""
        ...

    def __rmul__(self, other: object) -> typing.Self:
        """Create a multiplication expression."""
        ...

    def __ror__(self, other: object) -> typing.Self:
        """Create a logical OR expression."""
        ...

    def __rrshift__(self, other: object) -> typing.Self:
        """Create a bitwise right-shift expression."""
        ...

    def __rshift__(self, other: object) -> typing.Self:
        """Create a bitwise right-shift expression."""
        ...

    def __rsub__(self, other: object) -> typing.Self:
        """Create a subtraction expression."""
        ...

    def __rtruediv__(self, other: object) -> typing.Self:
        """Create a division expression."""
        ...

    def __sub__(self, other: object) -> typing.Self:
        """Create a subtraction expression."""
        ...

    def __truediv__(self, other: object) -> typing.Self:
        """Create a division expression."""
        ...

    @classmethod
    def asterisk(cls) -> typing.Self:
        """Shorthand for `Expr(ASTERISK)`"""
        ...

    def between(self, a: object, b: object) -> typing.Self:
        """Create a BETWEEN range comparison expression."""
        ...

    def bit_and(self, other: object) -> typing.Self: ...
    def bit_or(self, other: object) -> typing.Self: ...
    def cast_as(self, value: str) -> typing.Self:
        """Create a CAST expression to convert to a specific SQL type."""
        ...

    @classmethod
    def col(cls, value: str | ColumnRef) -> typing.Self:
        """Shorthand for `Expr(ColumnRef.parse(value))`"""
        ...

    @classmethod
    def current_date(cls) -> typing.Self:
        """Create an expression for the CURRENT_DATE SQL function."""
        ...

    @classmethod
    def current_time(cls) -> typing.Self:
        """Create an expression for the CURRENT_TIME SQL function."""
        ...

    @classmethod
    def current_timestamp(cls) -> typing.Self:
        """Create an expression for the CURRENT_TIMESTAMP SQL function."""
        ...

    @classmethod
    def custom(cls, value: str) -> typing.Self:
        """
        Create an expression from a custom SQL string.

        Warning: This method does not escape the input, so it should only
        be used with trusted strings to avoid SQL injection vulnerabilities.
        """
        ...

    def is_(self, other: object) -> typing.Self:
        """Create an IS comparison expression."""
        ...

    def is_not(self, other: object) -> typing.Self:
        """Create an IS NOT comparison expression."""
        ...

    def is_not_null(self) -> typing.Self:
        """Create an IS NOT NULL expression."""
        ...

    def is_null(self) -> typing.Self:
        """Create an IS NULL expression."""
        ...

    def like(self, pattern: str, escape: str | None = ...) -> typing.Self:
        """Create a LIKE pattern matching expression."""
        ...

    def not_between(self, a: object, b: object) -> typing.Self:
        """Create a NOT BETWEEN range comparison expression."""
        ...

    def not_like(self, pattern: str, escape: str | None = ...) -> typing.Self:
        """Create a NOT LIKE pattern matching expression."""
        ...

    @classmethod
    def null(cls) -> typing.Self:
        """Create an expression representing the NULL value."""
        ...

    @classmethod
    def val(cls, /, value: I | None, sql_type: SQLTypeAbstract[I, O] | None = ...) -> typing.Self:
        """Shorthand for `Expr(Value(value, sql_type))`"""
        ...

    __hash__ = None  # type: ignore

@typing.final
class FloatType(SQLTypeAbstract[float | int, float]):
    """
    Single-precision floating point column type (FLOAT).

    Stores approximate numeric values with single precision. Suitable for
    scientific calculations, measurements, or any numeric data where some
    precision loss is acceptable in exchange for storage efficiency.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class ForeignKey:
    """
    Specifies a foreign key relationship between tables.

    Defines referential integrity constraints including:
    - Source columns (in the child table)
    - Target columns (in the parent table)
    - Actions for updates and deletes (CASCADE, RESTRICT, SET NULL, etc.)
    - Optional naming for the constraint

    Foreign keys ensure data consistency by requiring that values in the
    child table's columns match existing values in the parent table's columns.
    """

    def __new__(
        cls,
        from_columns: typing.Iterable[str | ColumnRef | Column],
        to_table: Table | TableName | str,
        to_columns: typing.Iterable[str | ColumnRef | Column],
        name: str | None = None,
        *,
        on_delete: _ForeignKeyActions | None = None,
        on_update: _ForeignKeyActions | None = None,
    ) -> typing.Self: ...
    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def from_columns(self) -> typing.Sequence[str]:
        """Key columns."""
        ...
    @from_columns.setter
    def from_columns(self, value: typing.Iterable[str | Column | ColumnRef]) -> None: ...
    @property
    def from_table(self) -> TableName | None:
        """Key table, if specified."""
        ...
    @from_table.setter
    def from_table(self, value: Table | TableName | None) -> None: ...
    @property
    def name(self) -> str:
        """Foreign key constraint name"""
        ...
    @name.setter
    def name(self, value: str) -> None: ...
    @property
    def on_delete(self) -> _ForeignKeyActions | None:
        """ON DELETE action."""
        ...
    @on_delete.setter
    def on_delete(self, value: _ForeignKeyActions | None) -> None: ...
    @property
    def on_update(self) -> _ForeignKeyActions | None:
        """ON UPDATE action."""
        ...
    @on_update.setter
    def on_update(self, value: _ForeignKeyActions | None) -> None: ...
    @property
    def to_columns(self) -> typing.Sequence[str]:
        """Referencing columns."""
        ...
    @to_columns.setter
    def to_columns(self, value: typing.Iterable[str | Column | ColumnRef]) -> None: ...
    @property
    def to_table(self) -> TableName:
        """Referencing table."""
        ...
    @to_table.setter
    def to_table(self, value: TableName) -> None: ...

@typing.final
class Func:
    """
    Represents a SQL function call that can be used in expressions.

    This class provides a type-safe way to construct SQL function calls
    with proper argument handling and database dialect support.
    """

    def __new__(cls, name: str, *args: object) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @classmethod
    def abs(cls, /, expr: object) -> typing.Self:
        """Create a ABS(expr) function call."""
        ...

    @classmethod
    def avg(cls, /, expr: object) -> typing.Self:
        """Create a AVG(expr) function call."""
        ...

    @classmethod
    def bit_and(cls, /, expr: object) -> typing.Self:
        """
        Create a BIT_AND(expr) function call - this is not supported on SQLite.
        """
        ...

    @classmethod
    def bit_or(cls, /, expr: object) -> typing.Self:
        """Create a BIT_OR(expr) function call - this is not supported on SQLite."""
        ...

    @classmethod
    def char_length(cls, /, expr: object) -> typing.Self:
        """Create a CHAR_LENGTH(expr) function call."""
        ...

    @classmethod
    def coalesce(cls, /, *exprs: object) -> typing.Self:
        """Create a COALESCE function call."""
        ...

    @classmethod
    def count(cls, /, expr: object) -> typing.Self:
        """Create a COUNT(expr) function call."""
        ...

    @classmethod
    def count_distinct(cls, /, expr: object) -> typing.Self:
        """Create a COUNT(DISTINCT expr) function call."""
        ...

    @classmethod
    def dense_rank(cls, /) -> typing.Self:
        """Create a DENSE_RANK() function call."""
        ...

    @classmethod
    def greatest(cls, /, *exprs: object) -> typing.Self:
        """Create a GREATEST function call."""
        ...

    @classmethod
    def if_null(cls, /, a: object, b: object) -> typing.Self:
        """Create a IF_NULL(a, b) function call."""
        ...

    @classmethod
    def least(cls, /, *exprs: object) -> typing.Self:
        """Create a LEAST function call."""
        ...

    @classmethod
    def lower(cls, /, expr: object) -> typing.Self:
        """Create a LOWER(expr) function call."""
        ...

    @classmethod
    def max(cls, /, expr: object) -> typing.Self:
        """Create a MAX(expr) function call."""
        ...

    @classmethod
    def md5(cls, /, expr: object) -> typing.Self:
        """
        Create a MD5(expr) function call - this is only available in Postgres and MySQL.
        """
        ...

    @classmethod
    def min(cls, /, expr: object) -> typing.Self:
        """Create a MIN(expr) function call."""
        ...

    @classmethod
    def now(cls) -> typing.Self:
        """Create a NOW() function call."""
        ...

    @classmethod
    def percent_rank(cls, /) -> typing.Self:
        """Create a PERCENT_RANK() function call."""
        ...

    @classmethod
    def random(cls, /) -> typing.Self:
        """Create a RANDOM() function call."""
        ...

    @classmethod
    def rank(cls, /) -> typing.Self:
        """Create a RANK() function call."""
        ...

    @classmethod
    def round(cls, /, expr: object) -> typing.Self:
        """Create a ROUND(expr) function call."""
        ...

    @classmethod
    def round_with_precision(cls, /, a: object, b: object) -> typing.Self:
        """Create a ROUND(a, b) function call."""
        ...

    @classmethod
    def sum(cls, /, expr: object) -> typing.Self:
        """Create a SUM(expr) function call."""
        ...

    @classmethod
    def upper(cls, /, expr: object) -> typing.Self:
        """Create a UPPER(expr) function call."""
        ...

@typing.final
class INETType(SQLTypeAbstract[str, str]):
    """
    Internet address column type (INET).

    Stores IPv4 or IPv6 addresses, with or without subnet specification.
    More flexible than CIDR type, allowing both host addresses and network ranges.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Index(SchemaStatement):
    """
    Represents a database index specification.

    This class defines the structure and properties of a database index,
    including column definitions, uniqueness constraints, index type,
    and partial indexing conditions.

    You can use it to generate `CREATE INDEX` SQL expressions.
    """

    def __new__(
        cls,
        columns: typing.Iterable[_IndexColumnValue],
        name: str | None = None,
        table: Table | TableName | str | None = None,
        options: int = 0,
        *,
        index_type: str | None = None,
        where: object | None = None,
        include: typing.Iterable[str] = (),
    ) -> typing.Self: ...
    OPT_IF_NOT_EXISTS: typing.ClassVar[int] = ...
    OPT_NULLS_NOT_DISTINCT: typing.ClassVar[int] = ...
    OPT_PRIMARY: typing.ClassVar[int] = ...
    OPT_UNIQUE: typing.ClassVar[int] = ...

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
        """
        Whether to use IF NOT EXISTS clause. Shorthand for `self.options & self.OPT_IF_NOT_EXISTS > 0`.
        """
        ...

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
        """
        Whether NULL values should be considered equal for uniqueness.
        Shorthand for `self.options & self.OPT_NULLS_NOT_DISTINCT > 0`.
        """
        ...

    @property
    def options(self) -> int:
        """Index specified options."""
        ...
    @options.setter
    def options(self, value: int) -> None: ...
    @property
    def primary(self) -> bool:
        """
        Whether this is a primary key constraint.
        Shorthand for `self.options & self.OPT_PRIMARY > 0`.
        """
        ...

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
        """
        Whether this is a unique constraint.
        Shorthand for `self.options & self.OPT_UNIQUE > 0`.
        """
        ...

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
        cls, name: str, order: _IndexColumnOrder | None = None, prefix: int | None = None
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

@typing.final
class Insert(QueryStatement):
    """
    Builds INSERT SQL statements with a fluent interface.

    Provides a chainable API for constructing INSERT queries with support for:
    - Single or multiple row insertion
    - Conflict resolution (UPSERT)
    - RETURNING clauses
    - REPLACE functionality
    - Default values
    """

    def __new__(cls, table: Table | TableName | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def columns(self, *args: Column | ColumnRef | str) -> typing.Self:
        """
        Specify the columns for insertion.

        There's no need to use this method when you're specifying column
        names in `.values` method.
        """
        ...

    def into(self, table: Table | TableName | str) -> typing.Self:
        """Specify the target table for insertion."""
        ...

    def on_conflict(self, action: OnConflict) -> typing.Self:
        """Specify conflict resolution behavior (UPSERT)."""
        ...

    def or_default_values(self, rows: int = 1) -> typing.Self:
        """
        Use DEFAULT VALUES if no values were specified. The `rows`
        Specifies number of rows to insert with default values.
        """
        ...

    def replace(self) -> typing.Self:
        """
        Convert this INSERT to a REPLACE statement.

        REPLACE will delete existing rows that conflict with the new row
        before inserting.
        """
        ...

    def returning(self, *args: Column | ColumnRef | str) -> typing.Self:
        """Specify columns to return from the inserted rows."""
        ...

    def returning_all(self) -> typing.Self:
        """
        Return all columns from the inserted rows. Same as `self.returning("*")`.
        """
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    @typing.overload
    def values(self, *args: object) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.
        """
        ...

    @typing.overload
    def values(self, **kwds: object) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.
        """
        ...

@typing.final
class IntegerType(SQLTypeAbstract[int, int]):
    """
    Standard integer column type (INTEGER/INT).

    The most common integer type, typically storing 32-bit integers in the
    range -2,147,483,648 to 2,147,483,647 (signed). Suitable for most
    numeric data including IDs, quantities, and counters.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class JSONBinaryType(SQLTypeAbstract[typing.Any, typing.Any]):
    """
    Binary JSON column type (JSONB).

    Stores JSON documents in a binary format for improved performance.
    Provides faster query and manipulation operations compared to text-based
    JSON storage, with additional indexing capabilities.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class JSONType(SQLTypeAbstract[typing.Any, typing.Any]):
    """
    JSON data column type (JSON).

    Stores JSON documents with validation and indexing capabilities.
    Allows for flexible schema design and complex nested data structures
    while maintaining some query capabilities.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class MacAddressType(SQLTypeAbstract[str, str]):
    """
    MAC address column type (MACADDR).

    Stores MAC (Media Access Control) addresses for network devices.
    Provides validation and formatting for 6-byte MAC addresses.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class OnConflict:
    """
    Specifies conflict resolution behavior for INSERT statements.

    Handles situations where an INSERT would violate a unique constraint
    or primary key. Supports various strategies:
    - DO NOTHING: Skip the conflicting row
    - DO UPDATE: Update the existing row with new values

    This corresponds to INSERT ... ON CONFLICT in PostgreSQL and
    INSERT ... ON DUPLICATE KEY UPDATE in MySQL.
    """

    def __new__(cls, *targets: Column | ColumnRef | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def action_where(self, condition: Expr) -> typing.Self:
        """Add a WHERE clause to the conflict action (conditional update)."""
        ...

    def do_nothing(self, *keys: Column | ColumnRef | str) -> typing.Self:
        """
        Specify DO NOTHING action for conflicts.

        When a conflict occurs, the conflicting row will be skipped.

        `keys` parameter provides primary keys if you are using MySQL, for MySQL specific polyfill.
        """
        ...

    @typing.overload
    def do_update(self, *args: Column | ColumnRef | str) -> typing.Self:
        """
        Specify DO UPDATE action for conflicts using column names, or with explicit values.
        """
        ...

    @typing.overload
    def do_update(self, **kwds: object) -> typing.Self:
        """
        Specify DO UPDATE action for conflicts using column names, or with explicit values.
        """
        ...

    def target_where(self, condition: Expr) -> typing.Self:
        """Add a WHERE clause to the conflict target (partial unique index)."""
        ...

class QueryStatement:
    """Subclass of query statements."""

    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

@typing.final
class RenameTable(SchemaStatement):
    """
    Represents a RENAME TABLE SQL statement.

    Changes the name of an existing table to a new name. Both names can be
    schema-qualified if needed.
    """

    def __new__(
        cls, from_name: Table | TableName | str, to_name: Table | TableName | str
    ) -> typing.Self: ...
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

class SQLTypeAbstract(typing.Generic[I, O]):
    """
    Base class for all SQL column data types.

    This abstract base class represents SQL data types that can be used in
    column definitions. Each subclass implements a specific SQL data type
    with its particular characteristics, constraints, and backend-specific
    representations.
    """

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

class SchemaStatement:
    """Subclass of schema statements."""

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

@typing.final
class SmallIntegerType(SQLTypeAbstract[int, int]):
    """
    Small integer column type (SMALLINT).

    Typically stores integers in the range -32,768 to 32,767 (signed) or
    0 to 65,535 (unsigned). Good for moderate-sized counters or numeric codes.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class SmallUnsignedType(SQLTypeAbstract[int, int]):
    """
    Unsigned small integer column type.

    Stores moderate positive integers only, typically 0 to 65,535. Good for
    larger counters or numeric codes that are always positive.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class StringType(SQLTypeAbstract[str, str]):
    """
    Variable-length character string column type (VARCHAR).

    Represents a variable-length character string with a maximum length limit.
    This is the most common string type for storing text data of varying lengths
    like names, descriptions, or user input.
    """

    def __new__(cls, length: int | None = ...) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...

@typing.final
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

    def __new__(
        cls,
        name: TableName | str,
        *args: Column | Index | ForeignKey | Expr,
        options: int = 0,
        comment: str | None = None,
        engine: str | None = None,
        collate: str | None = None,
        character_set: str | None = None,
        extra: str | None = None,
    ) -> typing.Self: ...
    OPT_IF_NOT_EXISTS: typing.ClassVar[int] = ...
    OPT_TEMPORARY: typing.ClassVar[int] = ...

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
        """
        Whether to use IF NOT EXISTS clause to avoid errors if table exists.
        Shorthand for `self.options & self.OPT_IF_NOT_EXISTS > 0`.
        """
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
    def options(self) -> int:
        """Specified options."""
        ...
    @options.setter
    def options(self, value: int) -> None: ...
    @property
    def temporary(self) -> bool:
        """
        Whether this is a temporary table that exists only for the session.
        Shorthand for `self.options & self.OPT_TEMPORARY > 0`.
        """
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...

@typing.final
class TableName:
    """
    Represents a table name reference with optional schema, database, and alias.

    This class encapsulates a table name that can include:
    - The base table name
    - Optional schema/namespace qualification
    - Optional database qualification (for systems that support it)

    The class provides parsing capabilities for string representations
    and supports comparison operations.

    NOTE: this class is immutable and frozen.
    """

    def __new__(
        cls,
        name: str,
        schema: str | None = None,
        database: str | None = None,
        alias: str | None = None,
    ) -> typing.Self: ...
    def __copy__(self) -> typing.Self: ...
    def __eq__(self, value, /) -> bool:
        """Return self==value."""
        ...

    def __ge__(self, value, /) -> bool:
        """Return self>=value."""
        ...

    def __gt__(self, value, /) -> bool:
        """Return self>value."""
        ...

    def __le__(self, value, /) -> bool:
        """Return self<=value."""
        ...

    def __lt__(self, value, /) -> bool:
        """Return self<value."""
        ...

    def __ne__(self, value, /) -> bool:
        """Return self!=value."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def alias(self) -> str | None: ...
    def copy_with(
        self,
        *,
        name: str = ...,
        schema: str | None = ...,
        database: str | None = ...,
        alias: str | None = ...,
    ) -> typing.Self:
        """Create a shallow copy of this TableName."""
        ...

    @property
    def database(self) -> str | None: ...
    @property
    def name(self) -> str: ...
    @classmethod
    def parse(cls, string: str) -> typing.Self:
        """
        Parse a string representation of a table name.

        Supports formats like:
        - "table_name"
        - "schema.table_name"
        - "database.schema.table_name"
        """
        ...

    @property
    def schema(self) -> str | None: ...

    __hash__ = None  # type: ignore

@typing.final
class TextType(SQLTypeAbstract[float | int, float]):
    """
    Large text column type (TEXT).

    Represents a large text field capable of storing long strings without
    a predefined length limit. Suitable for storing articles, comments,
    descriptions, or any text content that may be very long.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class TimeType(SQLTypeAbstract[datetime.time, datetime.time]):
    """
    Time-only column type (TIME).

    Stores time information without date component. Useful for storing
    daily schedules, opening hours, or any time-based data that repeats
    daily regardless of the specific date.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class TimestampType(SQLTypeAbstract[datetime.datetime | int | float, datetime.datetime]):
    """
    Timestamp column type (TIMESTAMP).

    Stores timestamp values, often with automatic update capabilities.
    Behavior varies by database system.
    """

    def __new__(cls, timezone=False) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def timezone(self) -> bool: ...

@typing.final
class TinyIntegerType(SQLTypeAbstract[int, int]):
    """
    Very small integer column type (TINYINT).

    Typically stores integers in the range -128 to 127 (signed) or 0 to 255
    (unsigned). Useful for flags, small counters, or enumerated values.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class TinyUnsignedType(SQLTypeAbstract[int, int]):
    """
    Unsigned tiny integer column type.

    Stores small positive integers only, typically 0 to 255. Useful for
    small counters, percentages, or enumerated values that are always positive.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class TruncateTable(SchemaStatement):
    """
    Represents a TRUNCATE TABLE SQL statement.

    Quickly removes all rows from a table, typically faster than DELETE
    and with different transaction and trigger behavior depending on the
    database system.
    """

    def __new__(cls, name: Table | TableName | str) -> typing.Self: ...
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

@typing.final
class UUIDType(SQLTypeAbstract[uuid.UUID, uuid.UUID]):
    """
    UUID column type (UUID).

    Stores universally unique identifiers. Ideal for distributed systems,
    primary keys, or any situation where globally unique identifiers are
    needed without central coordination.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class UnsignedType(SQLTypeAbstract[int, int]):
    """
    Unsigned integer column type.

    Stores positive integers only, typically 0 to 4,294,967,295. Doubles the
    positive range compared to signed integers, useful for IDs and counters
    that will never be negative.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Update(QueryStatement):
    """
    Builds UPDATE SQL statements with a fluent interface.

    Provides a chainable API for constructing UPDATE queries with support for:
    - Setting column values
    - WHERE conditions for filtering
    - LIMIT for restricting update count
    - ORDER BY for determining update order
    - RETURNING clauses for getting updated data
    """

    def __new__(cls, table: Table | TableName | str) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: Table | TableName | str) -> typing.Self:
        """
        Update using data from another table (`UPDATE .. FROM ..`).

        MySQL doesn't support the UPDATE FROM syntax. And the current implementation attempt to
        tranform it to the UPDATE JOIN syntax, which only works for one join target.
        """
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to update."""
        ...

    def order_by(
        self,
        target: Expr | Column | ColumnRef | str,
        order: typing.Literal["ASC", "DESC"] = "ASC",
        null_ordering: typing.Literal["FIRST", "LAST"] | None = None,
    ) -> typing.Self:
        """
        Specify the order in which to update rows. Typically used with
        `.limit` method to update specific rows.
        """
        ...

    def returning(self, *args: Column | ColumnRef | str) -> typing.Self:
        """Specify columns to return from the updated rows."""
        ...

    def returning_all(self) -> typing.Self:
        """
        Return all columns from the updated rows. Same as `self.returning("*")`.
        """
        ...

    def table(self, table: Table | TableName | str) -> typing.Self:
        """Specify the table to update."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    def values(self, **kwds: object) -> typing.Self:
        """Specify columns and their new values."""
        ...

    def where(self, condition: Expr) -> typing.Self:
        """Add a WHERE condition to filter rows to update."""
        ...

@typing.final
class Value(typing.Generic[I, O]):
    """
    Bridges Python types, Rust types, and SQL types for seamless data conversion.

    This class handles validation, adaptation, and conversion between different
    type systems used in the application stack.

    It can automatically detects the type of your value and selects appropriate Rust and SQL types.
    For example:
    - Python `int` becomes `BIGINT` SQL type (`BigIntegerType`)
    - Python `dict` or `list` becomes `JSON` SQL type (`JsonType`)
    - Python `float` becomes `DOUBLE` SQL type (`DoubleType`)

    However, for more accurate type selection, it's recommended to use the `sql_type` parameter.

    NOTE: this class is immutable and frozen.
    """

    def __new__(
        cls, value: I | None, sql_type: SQLTypeAbstract[I, O] | None = ...
    ) -> typing.Self: ...
    def __hash__(self, /) -> int:
        """Return hash(self)."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def sql_type(self) -> SQLTypeAbstract[I, O]: ...
    @property
    def value(self) -> I | O:
        """Converts the adapted value back to a Python type."""
        ...

@typing.final
class VarBinaryType(SQLTypeAbstract[bytes, bytes]):
    """
    Variable-length binary data column type (VARBINARY).

    Stores binary data of variable length up to a specified maximum.
    More storage-efficient than BINARY for binary data of varying lengths.
    """

    def __new__(cls, length=None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class VarBitType(SQLTypeAbstract[bytes, bytes]):
    """
    Variable-length bit string column type (VARBIT).

    Stores a variable number of bits up to a specified maximum. More flexible
    than fixed BIT type for bit strings of varying lengths.
    """

    def __new__(cls, length=255) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class VectorType(SQLTypeAbstract[list | tuple, list]):
    """
    Vector column type for storing mathematical vectors.

    Specialized type for storing vector data, often used in machine learning,
    similarity search, or mathematical applications.
    """

    def __new__(cls, length: int | None = None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...

@typing.final
class _AsteriskType:
    """Asterisk `"*"`"""

def all(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a logical AND condition that is true only if all conditions are true.

    This is equivalent to SQL's AND operator applied to multiple expressions.
    """
    ...

def any(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a logical OR condition that is true if any condition is true.

    This is equivalent to SQL's OR operator applied to multiple expressions.
    """
    ...

def not_(arg: Expr) -> Expr:
    """Create a logical NOT."""
    ...
