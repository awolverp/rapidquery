from __future__ import annotations

import typing

from .query import SelectStatement
from .sqltypes import SQLTypeAbstract

T = typing.TypeVar("T")
_ForeignKeyActions: typing.TypeAlias = typing.Literal[
    "CASCADE", "RESTRICT", "NO ACTION", "SET DEFAULT", "SET NULL"
]

class _ColumnRefProperty(typing.Protocol):
    __column_ref__: typing.Any

class _TableNameProperty(typing.Protocol):
    __table_name__: typing.Any

_ColumnRefNew = typing.Union[
    "ColumnRef",
    str,
    _ColumnRefProperty,
    type[_ColumnRefProperty],
]
_TableNameNew = typing.Union[
    "TableName",
    str,
    _TableNameProperty,
    type[_TableNameProperty],
]
_ExprNew = typing.Any

class Column(typing.Generic[T]):
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

    def __init__(
        self,
        name: str,
        type: SQLTypeAbstract[T],
        *,
        primary_key: bool = False,
        unique_key: bool = False,
        nullable: bool | None = None,
        auto_increment: bool = False,
        extra: str | None = None,
        comment: str | None = None,
        default: typing.Any = ...,
        generated: typing.Any = ...,
        stored_generated: bool = False,
    ) -> None:
        """
        Construct a table column with column type.

        Args:
            name: The name of this column as represented in the database.
            type: The column's type.
            primary_key: If `True`, marks this column as a primary key column.
            unique_key: If `True`, marks this column as a unique key column.
            nullable: If `False`, marks this column as "NOT NULL". If `True`, marks it as "NULL".
            auto_increment: If `True`, marks this column as auto increment. Auto increment phrase is
                            depended on backend/dialect.
            extra: Some extra options in custom string.
            comment: (MySQL only) Column comment.
            default: Represents default value for the column.
            generated: Sets the column as generated.
            stored_generated: Works with `generated` parameter together, which marks column as "STORED GENERATED".
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def adapt(self, object: T) -> Value[T]:
        """Shorthand for `Value(object, self.type)`."""
        ...

    @property
    def auto_increment(self) -> bool:
        """Whether this is a auto increment column."""
        ...
    @auto_increment.setter
    def auto_increment(self, value: bool) -> None: ...
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
    def default(self, value: _ExprNew) -> None: ...
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
    def generated(self, value: _ExprNew) -> None: ...
    @property
    def name(self) -> str:
        """Column name."""
        ...
    @name.setter
    def name(self, value: str) -> None: ...
    @property
    def nullable(self) -> bool | None:
        """Whether this is a nullable column."""
        ...
    @property
    def primary_key(self) -> bool:
        """Whether this is a nullable column."""
        ...
    @primary_key.setter
    def primary_key(self, value: bool) -> None: ...
    @property
    def stored_generated(self) -> bool:
        """Whether this is a stored generated column."""
        ...
    @stored_generated.setter
    def stored_generated(self, value: bool) -> None: ...
    @property
    def type(self) -> SQLTypeAbstract[T]:
        """Column type."""
        ...

    @property
    def unique_key(self) -> bool:
        """Whether this is a unique column."""
        ...
    @unique_key.setter
    def unique_key(self, value: bool) -> None: ...
    @property
    def __column_ref__(self) -> ColumnRef: ...

@typing.final
class ColumnRef:
    """
    Represents a reference to a database column with optional table and schema qualification.

    This class is used to uniquely identify columns in SQL queries, supporting
    schema-qualified and table-qualified column references.

    NOTE: this class is immutable and frozen, But you can use `.copy_with` method to make changes on it.
    """

    def __new__(
        cls,
        name: str,
        table: str | None = ...,
        schema: str | None = ...,
    ) -> typing.Self: ...
    def __copy__(self) -> typing.Self: ...
    def __eq__(self, value, /) -> bool: ...
    def __ge__(self, value, /) -> bool: ...
    def __gt__(self, value, /) -> bool: ...
    def __le__(self, value, /) -> bool: ...
    def __lt__(self, value, /) -> bool: ...
    def __ne__(self, value, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    def copy_with(
        self,
        *,
        name: str | None = ...,
        table: str | None = ...,
        schema: str | None = ...,
    ) -> typing.Self:
        """
        Returns a copy of this `ColumnRef`, but with the changes you want.
        """
        ...

    @property
    def name(self) -> str:
        """Column reference name."""
        ...

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
    def schema(self) -> str | None:
        """Schema name"""
        ...

    @property
    def table(self) -> str | None:
        """Table name"""
        ...

    __hash__ = None  # type: ignore

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

    def __new__(cls, value: _ExprNew, /) -> typing.Self: ...
    def __add__(self, other: _ExprNew) -> typing.Self:
        """Create an addition expression."""
        ...

    def __and__(self, other: _ExprNew) -> typing.Self:
        """Create a logical AND expression."""
        ...

    def __eq__(self, other: _ExprNew) -> typing.Self:  # type: ignore[override]
        """Create an equality comparison expression."""
        ...

    def __ge__(self, other: _ExprNew) -> typing.Self:
        """Create a greater-than-or-equal comparison expression."""
        ...

    def __gt__(self, other: _ExprNew) -> typing.Self:
        """Create a greater-than comparison expression."""
        ...

    def __le__(self, other: _ExprNew) -> typing.Self:
        """Create a less-than-or-equal comparison expression."""
        ...

    def __lshift__(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise left-shift expression."""
        ...

    def __lt__(self, other: _ExprNew) -> typing.Self:
        """Create a less-than comparison expression."""
        ...

    def __mod__(self, other: _ExprNew) -> typing.Self:
        """Create a modulo expression."""
        ...

    def __mul__(self, other: _ExprNew) -> typing.Self:
        """Create a multiplication expression."""
        ...

    def __ne__(self, other: _ExprNew) -> typing.Self:  # type: ignore[override]
        """Create an inequality comparison expression."""
        ...

    def __neg__(self) -> typing.Self:
        """Create a negation expression."""
        ...

    def __or__(self, other: _ExprNew) -> typing.Self:
        """Create a logical OR expression."""
        ...

    def __radd__(self, other: _ExprNew) -> typing.Self:
        """Create an addition expression."""
        ...

    def __rand__(self, other: _ExprNew) -> typing.Self:
        """Create a logical AND expression."""
        ...

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    def __rlshift__(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise left-shift expression."""
        ...

    def __rmod__(self, other: _ExprNew) -> typing.Self:
        """Create a modulo expression."""
        ...

    def __rmul__(self, other: _ExprNew) -> typing.Self:
        """Create a multiplication expression."""
        ...

    def __ror__(self, other: _ExprNew) -> typing.Self:
        """Create a logical OR expression."""
        ...

    def __rrshift__(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise right-shift expression."""
        ...

    def __rshift__(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise right-shift expression."""
        ...

    def __rsub__(self, other: _ExprNew) -> typing.Self:
        """Create a subtraction expression."""
        ...

    def __rtruediv__(self, other: _ExprNew) -> typing.Self:
        """Create a division expression."""
        ...

    def __sub__(self, other: _ExprNew) -> typing.Self:
        """Create a subtraction expression."""
        ...

    def __truediv__(self, other: _ExprNew) -> typing.Self:
        """Create a division expression."""
        ...

    @classmethod
    def all(cls, statement: SelectStatement) -> typing.Self:
        """Express a `ALL` sub-query expression."""
        ...

    @classmethod
    def any(cls, statement: SelectStatement) -> typing.Self:
        """Express a `ANY` sub-query expression."""
        ...

    @classmethod
    def asterisk(cls) -> typing.Self:
        """Returns asterisk '*' expression."""
        ...

    def between(self, a: _ExprNew, b: _ExprNew) -> typing.Self:
        """Create a BETWEEN range comparison expression."""
        ...

    def bit_and(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise AND expression."""
        ...

    def bit_or(self, other: _ExprNew) -> typing.Self:
        """Create a bitwise AND expression."""
        ...

    def cast_as(self, value: str) -> typing.Self:
        """Create a `CAST` expression to convert to a specific SQL type."""
        ...

    @classmethod
    def col(cls, value: _ColumnRefNew) -> typing.Self:
        """
        Tries to convert the `value` into `ColumnRef`, and then converts it to `Expr`.
        """
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

    @classmethod
    def exists(cls, statement: SelectStatement) -> typing.Self:
        """Express a `EXISTS` sub-query expression."""
        ...

    def in_(self, other: typing.Iterable[_ExprNew] | SelectStatement) -> typing.Self:
        """Express a `IN` expression."""
        ...

    def is_(self, other: _ExprNew) -> typing.Self:
        """Create an IS comparison expression."""
        ...

    def is_not(self, other: _ExprNew) -> typing.Self:
        """Create an IS NOT comparison expression."""
        ...

    def is_not_null(self) -> typing.Self:
        """Create an IS NOT NULL expression."""
        ...

    def is_null(self) -> typing.Self:
        """Create an IS NULL expression."""
        ...

    def like(self, pattern: str, escape: str | None = ...) -> typing.Self:
        """Create a `LIKE` pattern matching expression."""
        ...

    def not_between(self, a: _ExprNew, b: _ExprNew) -> typing.Self:
        """Create a NOT BETWEEN range comparison expression."""
        ...

    def not_in(self, other: typing.Iterable[_ExprNew] | SelectStatement) -> typing.Self:
        """Express a `NOT IN` expression."""
        ...

    def not_like(self, pattern: str, escape: str | None = ...) -> typing.Self:
        """Create a NOT LIKE pattern matching expression."""
        ...

    @classmethod
    def null(cls) -> typing.Self:
        """Create an expression representing the NULL value."""
        ...

    @classmethod
    def some(cls, statement: SelectStatement) -> typing.Self:
        """Express a `SOME` sub-query expression."""
        ...

    @classmethod
    def val(
        cls,
        /,
        value: T | None,
        sql_type: SQLTypeAbstract[T] | None = ...,
    ) -> typing.Self:
        """Shorthand for `Expr(Value(value, sql_type))`"""
        ...

    __hash__ = None  # type: ignore

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

    def __init__(
        self,
        from_columns: typing.Iterable[_ColumnRefNew],
        to_columns: typing.Iterable[_ColumnRefNew],
        to_table: _TableNameNew | None = None,
        name: str | None = None,
        *,
        on_delete: _ForeignKeyActions | None = None,
        on_update: _ForeignKeyActions | None = None,
    ) -> None:
        """
        Construct a foreign key constraint.

        Args:
            from_columns: An iterable of column references, or column names, or `Column` object.
                        The columns must defined and presented in the parent table. Cannot be empty.
            to_columns: An iterable of column references, or column names, or `Column` object.
                        The columns must defined and presented in the target table. Cannot be empty.
            to_table: The target table. If not specified, tries to detect it from `to_columns` argument.
            name: The constraint name.
            on_delete: The foreign key action for "ON DELETE".
            on_update: The foreign key action for "ON UPDATE".
        """
        ...
    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    @property
    def from_columns(self) -> typing.Sequence[str]:
        """Key columns."""
        ...
    @from_columns.setter
    def from_columns(self, value: typing.Iterable[_ColumnRefNew]) -> None: ...
    @property
    def name(self) -> str | None:
        """Foreign key constraint name"""
        ...
    @name.setter
    def name(self, value: str | None) -> None: ...
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
    def to_columns(self, value: typing.Iterable[_ColumnRefNew]) -> None: ...
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

    def __new__(cls, name: str, *args: _ExprNew) -> typing.Self:
        """
        Construct a function expression. You can use ready-to-use classmethods instead of this.

        Args:
            name: the function name in string.
            args: function arguments.
        """
        ...

    def __repr__(self, /) -> str: ...
    @classmethod
    def abs(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a ABS(expr) function call."""
        ...

    @classmethod
    def avg(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a AVG(expr) function call."""
        ...

    @classmethod
    def bit_and(cls, /, expr: _ExprNew) -> typing.Self:
        """
        Create a BIT_AND(expr) function call - this is not supported on SQLite.
        """
        ...

    @classmethod
    def bit_or(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a BIT_OR(expr) function call - this is not supported on SQLite."""
        ...

    @classmethod
    def char_length(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a CHAR_LENGTH(expr) function call."""
        ...

    @classmethod
    def coalesce(cls, /, *exprs: _ExprNew) -> typing.Self:
        """Create a COALESCE function call."""
        ...

    @classmethod
    def count(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a COUNT(expr) function call."""
        ...

    @classmethod
    def count_distinct(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a COUNT(DISTINCT expr) function call."""
        ...

    @classmethod
    def dense_rank(cls, /) -> typing.Self:
        """Create a DENSE_RANK() function call."""
        ...

    @classmethod
    def greatest(cls, /, *exprs: _ExprNew) -> typing.Self:
        """Create a GREATEST function call."""
        ...

    @classmethod
    def if_null(cls, /, a: _ExprNew, b: _ExprNew) -> typing.Self:
        """Create a IF_NULL(a, b) function call."""
        ...

    @classmethod
    def least(cls, /, *exprs: _ExprNew) -> typing.Self:
        """Create a LEAST function call."""
        ...

    @classmethod
    def lower(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a LOWER(expr) function call."""
        ...

    @classmethod
    def max(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a MAX(expr) function call."""
        ...

    @classmethod
    def md5(cls, /, expr: _ExprNew) -> typing.Self:
        """
        Create a MD5(expr) function call - this is only available in Postgres and MySQL.
        """
        ...

    @classmethod
    def min(cls, /, expr: _ExprNew) -> typing.Self:
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
    def round(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a ROUND(expr) function call."""
        ...

    @classmethod
    def round_with_precision(cls, /, a: _ExprNew, b: _ExprNew) -> typing.Self:
        """Create a ROUND(a, b) function call."""
        ...

    @classmethod
    def sum(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a SUM(expr) function call."""
        ...

    @classmethod
    def upper(cls, /, expr: _ExprNew) -> typing.Self:
        """Create a UPPER(expr) function call."""
        ...

    @classmethod
    def cast_as(cls, /, expr: _ExprNew, alias: str) -> typing.Self:
        """Call CAST function with a custom type."""
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

    def __ge__(self, value, /) -> bool: ...
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

class Value(typing.Generic[T]):
    """
    Bridges Python types, Rust types, and SQL types for seamless data conversion.

    This class handles validation, adaptation, and conversion between different
    type systems used in the application stack.

    NOTE: this class is immutable and frozen.
    """

    def __init__(
        self,
        value: T | None,
        sql_type: SQLTypeAbstract[T] | None = ...,
    ) -> None:
        """
        Construct a new `Value`.

        It can automatically detects the type of your value and selects appropriate Rust and SQL types.
        For example:
        - Python `int` becomes `BIGINT` SQL type (`BigIntegerType`)
        - Python `dict` or `list` becomes `JSON` SQL type (`JsonType`)
        - Python `float` becomes `DOUBLE` SQL type (`DoubleType`)

        However, for more accurate type selection, use the `sql_type` parameter.
        """
        ...

    def __hash__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...
    @property
    def sql_type(self) -> SQLTypeAbstract[T]:
        """The defined or detected SQL type."""
        ...

    @property
    def value(self) -> T | None:
        """Converts the adapted value back to Python."""
        ...

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
