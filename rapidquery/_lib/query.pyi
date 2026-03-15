from __future__ import annotations

import typing

from .common import Expr, Func, Value, _ColumnRefNew, _ExprNew, _TableNameNew

_BackendName: typing.TypeAlias = typing.Literal[
    "sqlite", "postgresql", "postgres", "mysql"
]

class CaseStatement:
    def __init__(self) -> None:
        """Construct a `CASE WHEN` statement."""
        ...

    def __repr__(self, /) -> str: ...
    def else_(self, result: _ExprNew) -> typing.Self:
        """Ends the case statement with the final ELSE result."""
        ...

    def when(self, condition: Expr, result: _ExprNew) -> typing.Self:
        """Adds new `WHEN` to existing case statement."""
        ...

class DeleteStatement(QueryStatement):
    """
    Builds `DELETE` SQL statements with a fluent interface.

    Provides a chainable API for constructing DELETE queries with support for:
    - WHERE conditions for filtering
    - LIMIT for restricting deletion count
    - ORDER BY for determining deletion order
    - RETURNING clauses for getting deleted data
    """

    def __init__(self, table: _TableNameNew) -> None:
        """
        Construct a `DELETE` statement.

        Args:
            table: The table to delete from.
        """
        ...

    def __repr__(self, /) -> str: ...
    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """Override the table to delete from."""
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to delete."""
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """
        Specify the order in which to delete rows. Typically used with
        `.limit` method to delete specific rows.
        """
        ...

    def returning(self, clause: Returning) -> typing.Self:
        """Specify columns to return from the inserted rows."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """Add a `WHERE` condition to filter rows to delete."""
        ...

@typing.final
class Frame:
    """Window frame start and frame end clause. Use its classmethods."""

    @classmethod
    def current_row(cls) -> typing.Self: ...
    @classmethod
    def following(cls, val: int) -> typing.Self: ...
    @classmethod
    def preceding(cls, val: int) -> typing.Self: ...
    @classmethod
    def unbounded_following(cls) -> typing.Self: ...
    @classmethod
    def unbounded_preceding(cls) -> typing.Self: ...

class InsertStatement(QueryStatement):
    """
    Builds `INSERT` SQL statements with a fluent interface.

    Provides a chainable API for constructing INSERT queries with support for:
    - Single or multiple row insertion
    - Conflict resolution (UPSERT)
    - RETURNING clauses
    - REPLACE functionality
    - Default values
    """

    def __init__(self, table: _TableNameNew) -> None:
        """
        Construct a `INSERT` statement.

        Args:
            table: The target table for insertion.
        """
        ...

    def __repr__(self, /) -> str: ...
    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def columns(self, *args: _ColumnRefNew) -> typing.Self:
        """
        Specify the columns for insertion.

        There's no need to use this method when you're specifying column
        names in `.values` method.
        """
        ...

    def into(self, table: _TableNameNew) -> typing.Self:
        """Override the target table for insertion."""
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

    def returning(self, clause: Returning) -> typing.Self:
        """Specify columns to return from the inserted rows."""
        ...

    def select_from(self, statement: SelectStatement) -> typing.Self:
        """Specify a select query whose values to be inserted."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    @typing.overload
    def values(self, *args: _ExprNew) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.
        """
        ...

    @typing.overload
    def values(self, **kwds: _ExprNew) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.
        """
        ...

class OnConflict:
    """
    Specifies conflict resolution behavior for `INSERT` statements.

    Handles situations where an `INSERT` would violate a unique constraint
    or primary key.

    This corresponds to `INSERT ... ON CONFLICT` in PostgreSQL and
    `INSERT ... ON DUPLICATE KEY UPDATE` in MySQL.
    """

    def __init__(self, *targets: _ColumnRefNew) -> None:
        """
        Construct a new `OnConflict` instance.

        Args:
            targets: Target columns.
        """
        ...

    def __repr__(self, /) -> str: ...
    def action_where(self, condition: Expr) -> typing.Self:
        """Add a `WHERE` clause to the conflict action (conditional update)."""
        ...

    def do_nothing(self, *keys: _ColumnRefNew) -> typing.Self:
        """
        Specify `DO NOTHING` action for conflicts.

        When a conflict occurs, the conflicting row will be skipped.

        `keys` parameter provides primary keys if you are using MySQL, for MySQL specific polyfill.
        """
        ...

    @typing.overload
    def do_update(self, *args: _ColumnRefNew) -> typing.Self:
        """
        Specify `DO UPDATE` action for conflicts using column names, or with explicit values.
        """
        ...

    @typing.overload
    def do_update(self, **kwds: _ExprNew) -> typing.Self:
        """
        Specify `DO UPDATE` action for conflicts using column names, or with explicit values.
        """
        ...

    def target_where(self, condition: Expr) -> typing.Self:
        """Add a `WHERE` clause to the conflict target (partial unique index)."""
        ...

@typing.final
class Ordering:
    """
    Specifies ordering behavior statements.

    NOTE: this class is immutable and frozen.
    """

    def __new__(
        cls,
        target: Expr | _ColumnRefNew,
        order: typing.Literal["ASC", "DESC"] = "ASC",
        null_ordering: typing.Literal["FIRST", "LAST"] | None = None,
    ) -> typing.Self:
        """
        Construct a new `Ordering` instance.

        Args:
            target: Ordering target. Can be an expression (`Expr`) or a column.
            order: Ascendant (`"ASC"`) or descendant (`"DESC"`).
            null_ordering: Null ordering option. `"FIRST"` means `NULLS FIRST` SQL, and
                        `"LAST"` means `NULLS LAST` SQL.
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def null_order(self) -> typing.Literal["FIRST", "LAST"] | None: ...
    @property
    def order(self) -> typing.Literal["ASC", "DESC"]: ...
    @property
    def target(self) -> Expr:
        """Target expression."""
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
class Returning:
    """
    `RETURNING` clause.

    Works on PostgreSQL and SQLite>=3.35.0.
    """

    def __new__(cls, *args: Expr | _ColumnRefNew) -> typing.Self:
        """
        Construct a new `RETURNING` clause.

        Args:
            args: Returning expressions. Can be expressions (`Expr`) or column references.
                If any `"*"`, or `ColumnRef("*")` found, ignores all others and returns `RETURNING *` clause.
        """
        ...

    def __repr__(self, /) -> str: ...
    @classmethod
    def all(cls) -> typing.Self:
        """Same as `self("*")`."""
        ...

class SelectLabel:
    """
    Represents a column expression with an optional alias in a `SELECT` statement.

    Used to specify both the expression to select and an optional alias name
    for the result column.
    """

    def __init__(
        self,
        expr: _ExprNew,
        alias: str | None = None,
        window: WindowStatement | str | None = None,
    ) -> None:
        """
        Construct a new `SelectLabel` instance.

        Args:
            expr: Target expression to select.
            alias: Target label, i.e. `<expr> AS <alias>`.
            window: Window statement or window name.
        """
        ...

    def __repr__(self, /) -> str: ...
    @property
    def alias(self) -> str | None: ...
    @property
    def expr(self) -> Expr: ...
    @property
    def window(self) -> WindowStatement | str | None: ...

class SelectStatement(QueryStatement):
    """
    Builds SELECT SQL statements with a fluent interface.

    Provides a chainable API for constructing SELECT queries with support for:
    - Column selection with expressions and aliases
    - Table and subquery sources
    - Filtering with WHERE and HAVING
    - Joins (inner, left, right, full, cross, lateral)
    - Grouping and aggregation
    - Ordering and pagination
    - Set operations (UNION, EXCEPT, INTERSECT)
    - Row locking for transactions
    - DISTINCT queries
    """

    def __init__(self, *exprs: _ExprNew) -> None: ...
    def __repr__(self, /) -> str: ...
    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def columns(self, *args: _ColumnRefNew) -> typing.Self:
        """
        Override select target expressions.

        Works same as `self.exprs(Expr.col(i) for i in args)`, but much easier and faster.
        """
        ...

    def distinct(self, *on: _ColumnRefNew) -> typing.Self:
        """
        Changes `SELECT` statement into `SELECT DISTINCT` statement.

        Args:
            on: Column references. If specified, uses *Postgres-ONLY* `SELECT DISTINCT ON ...` syntax.
        """
        ...

    def exprs(self, *args: _ExprNew) -> typing.Self:
        """Override select target expressions."""
        ...

    def from_function(self, function: Expr | Func, alias: str) -> typing.Self:
        """
        Select from function call.

        Args:
            function: An expression or a function. If it is `Expr`, it should be a function call
                    and nothing more, or `ValueError` will be raised.
            alias: label, i.e. `FROM <function> AS <alias>`.
        """
        ...

    def from_subquery(self, subquery: SelectStatement, alias: str) -> typing.Self:
        """
        Select from `SELECT` subquery.

        Args:
            subquery: A select statement.
            alias: label, i.e. `FROM <subquery> AS <alias>`.
        """
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """Select from table."""
        ...

    def group_by(self, *groups: Expr | _ColumnRefNew) -> typing.Self:
        """Add group by expressions or column references."""
        ...

    def having(self, condition: Expr) -> typing.Self:
        """Add having condition expression."""
        ...

    def join(
        self,
        table: _TableNameNew,
        on: Expr,
        type: typing.Literal["CROSS", "FULL", "INNER", "LEFT", "RIGHT"] | None = None,
    ) -> typing.Self:
        """
        Join with other table.

        Args:
            table: The table to join with.
            on: Join condition.
            type: Join type.
        """
        ...

    def join_function(
        self,
        function: Func | Expr,
        alias: str,
        on: Expr,
        type: typing.Literal["CROSS", "FULL", "INNER", "LEFT", "RIGHT"] | None = None,
    ) -> typing.Self:
        """
        Join with a function call.

        Args:
            function: An expression or a function. If it is `Expr`, it should be a function call
                    and nothing more, or `ValueError` will be raised.
            alias: label.
            on: Join condition.
            type: Join type.
        """
        ...

    def join_subquery(
        self,
        subquery: SelectStatement,
        alias: str,
        on: Expr,
        type: typing.Literal["CROSS", "FULL", "INNER", "LEFT", "RIGHT"] | None = None,
        lateral: bool = False,
    ) -> typing.Self:
        """
        Join with other `SELECT` statement.

        Args:
            subquery: Select statement.
            alias: label.
            on: Join condition.
            type: Join type.
            literal: If `True`, uses `JOIN LATERAL` syntax, which *is not supported by SQLite*.
        """
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to select."""
        ...

    def lock(
        self,
        type: typing.Literal[
            "UPDATE", "NO KEY UPDATE", "SHARE", "KEY SHARE"
        ] = "UPDATE",
        behavior: typing.Literal["NOWAIT", "SKIP"] | None = None,
        tables: typing.Iterable[_TableNameNew] = (),
    ) -> typing.Self:
        """
        Row locking (if is supported by backend/dialect).

        Args:
            type: Row locking type.
            behavior: Row locking behavior.
            tables: Row locking tables.
        """
        ...

    def offset(self, n: int) -> typing.Self:
        """Set offset."""
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """Specify the order in which to delete rows."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    def union(
        self,
        statement: SelectStatement,
        type: typing.Literal["ALL", "INTERSECT", "DISTINCT", "EXCEPT"] = "DISTINCT",
    ) -> typing.Self:
        """
        Union with multiple `SELECT` statement that **must have the same selected fields**.

        Args:
            statement: Select statement.
            type: Union type.
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """Add select `WHERE` condition."""
        ...

    def window(self, name: str, statement: WindowStatement) -> typing.Self:
        """
        Add `WINDOW` to statement.

        Args:
            name: Window name.
            statement: Window statement.
        """
        ...

class UpdateStatement(QueryStatement):
    """
    Builds UPDATE SQL statements with a fluent interface.

    Provides a chainable API for constructing UPDATE queries with support for:
    - Setting column values
    - WHERE conditions for filtering
    - LIMIT for restricting update count
    - ORDER BY for determining update order
    - RETURNING clauses for getting updated data
    """

    def __init__(self, table: _TableNameNew) -> None:
        """
        Construct a new `UPDATE` statement.

        Args:
            table: The table to update.
        """
        ...

    def __repr__(self, /) -> str: ...
    def build(self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]:
        """Build the SQL statement with parameter values."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """
        Update using data from another table (`UPDATE .. FROM ..`).

        MySQL doesn't support the UPDATE FROM syntax. And the current implementation attempt to
        tranform it to the UPDATE JOIN syntax, which only works for one join target.
        """
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to update."""
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """
        Specify the order in which to delete rows. Typically used with
        `.limit` method to delete specific rows.
        """
        ...

    def returning(self, clause: Returning) -> typing.Self:
        """Specify columns to return from the inserted rows."""
        ...

    def table(self, table: _TableNameNew) -> typing.Self:
        """Specify the table to update."""
        ...

    def to_sql(self, backend: _BackendName, /) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...

    def values(self, **kwds: _ExprNew) -> typing.Self:
        """Specify columns and their new values."""
        ...

    def where(self, condition: Expr) -> typing.Self:
        """Add a WHERE condition to filter rows to update."""
        ...

class WindowStatement:
    """
    Window expression.

    # References:

    1. <https://dev.mysql.com/doc/refman/8.0/en/window-function-descriptions.html>
    2. <https://www.sqlite.org/windowfunctions.html>
    3. <https://www.postgresql.org/docs/current/tutorial-window.html>
    """

    def __init__(self, *partition_by: Expr | _ColumnRefNew) -> None:
        """
        Construct a `WINDOW` statement.
        """
        ...

    def __repr__(self, /) -> str: ...
    def frame(
        self,
        frame_type: typing.Literal["ROWS", "RANGE"],
        frame_start: Frame,
        frame_end: Frame | None = None,
    ) -> typing.Self: ...
    def order_by(self, clause: Ordering) -> typing.Self: ...
    def partition(self, partition_by: Expr | _ColumnRefNew) -> typing.Self:
        """Partition by column or custom expression."""
        ...
