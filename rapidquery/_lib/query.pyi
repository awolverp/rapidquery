from __future__ import annotations

import typing

from .common import Expr, Func, Value, _ColumnRefNew, _ExprNew, _TableNameNew

_BackendName: typing.TypeAlias = typing.Literal[
    "sqlite", "postgresql", "postgres", "mysql"
]

class CaseStatement:
    """
    Represents a `CASE WHEN` SQL statement.
    """

    def __init__(self) -> None:
        """
        Construct a `CASE WHEN` statement.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(
                rq.SelectLabel(
                    rq.CaseStatement()
                        .when(rq.Expr.col("aspect").in_([2, 4]), True)
                        .else_(False),
                    "is_even",
                ),
            )
            .from_table("glyph")
            .to_sql("postgres")
        )
        # SELECT (CASE WHEN ("aspect" IN (2, 4)) THEN TRUE ELSE FALSE END) AS "is_even" FROM "glyph"
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def else_(self, result: _ExprNew) -> typing.Self:
        """
        Ends the case statement with the final ELSE result.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(
                rq.SelectLabel(
                    rq.CaseStatement()
                        .when(rq.Expr.col("aspect") > 0, "positive")
                        .when(rq.Expr.col("aspect") < 0, "negative")
                        .else_("zero"),
                    "polarity",
                ),
            )
            .from_table("glyph")
            .to_sql("postgres")
        )
        # SELECT (CASE WHEN ("aspect" > 0) THEN 'positive' WHEN ("aspect" < 0) THEN 'negative' ELSE 'zero' END) AS "polarity" FROM "glyph"
        ```
        """
        ...

    def when(self, condition: Expr, result: _ExprNew) -> typing.Self:
        """
        Adds new `WHEN` to existing case statement.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(
                rq.CaseStatement()
                .when(
                    rq.any(
                        rq.Expr.col("font_size") > 48,
                        rq.Expr.col("size_w") > 500,
                    ),
                    "large",
                )
                .when(
                    rq.any(
                        rq.Expr.col("font_size").between(24, 48),
                        rq.Expr.col("size_w").between(300, 500),
                    ),
                    "medium",
                )
                .else_("small")
                .label("char_size"),
            )
            .from_table("characters")
            .to_sql("postgres")
        )
        # SELECT
        # (CASE WHEN ("font_size" > 48 OR "size_w" > 500) THEN 'large'
        # WHEN (("font_size" BETWEEN 24 AND 48) OR ("size_w" BETWEEN 300 AND 500)) THEN 'medium'
        # ELSE 'small' END) AS "char_size"
        # FROM "characters"
        ```
        """
        ...

    def to_expr(self) -> Expr:
        """Shorthand for `Expr(self)`"""
        ...

    def label(
        self, alias: str, window: WindowStatement | str | None = None
    ) -> SelectLabel:
        """Shorthand for `SelectLabel(self, alias, window)`"""
        pass

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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.DeleteStatement("glyph")
            .where(rq.any(rq.Expr.col("id") < 1, rq.Expr.col("id") > 10))
            .to_sql("mysql")
        )
        # DELETE FROM `glyph` WHERE `id` < 1 OR `id` > 10
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """
        Override the table to delete from.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.DeleteStatement("name_1")
            .where(rq.Expr.col("id") == 1)
            .from_table("name_2")
            .to_sql("sqlite")
        )
        # DELETE FROM "name_2" WHERE "id" = 1
        ```
        """
        ...

    def limit(self, n: int) -> typing.Self:
        """Limit the number of rows to delete."""
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """
        Specify the order in which to delete rows. Typically used with
        `.limit` method to delete specific rows.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.DeleteStatement("glyph")
            .where(rq.Expr.col("id") < 100)
            .limit(20)
            .order_by(rq.Ordering("id", "DESC"))
            .to_sql("postgres")
        )
        # DELETE FROM "glyph" WHERE "id" < 100 ORDER BY "id" DESC LIMIT 20
        ```
        """
        ...

    def returning(self, clause: Returning) -> typing.Self:
        """
        Specify columns to return from the inserted rows.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.DeleteStatement("glyph")
            .where(rq.Expr.col("id") < 30)
            .returning(rq.Returning("id"))
            .to_sql("sqlite")
        )
        # DELETE FROM "glyph" WHERE "id" < 30 RETURNING "id"
        ```
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """
        Add a `WHERE` condition to filter rows to delete.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.DeleteStatement("glyph")
            .where(rq.Expr.col("id") < 100)
            .where(rq.Expr.col("name").like("%A"))
            .to_sql("postgres")
        )
        # DELETE FROM "glyph" WHERE "id" < 100 AND "name" LIKE '%A'
        ```
        """
        ...

@typing.final
class Frame:
    """
    Defines the window frame start and end boundaries for window functions.

    A frame allows you to constrain the set of rows used by the window function
    to a subset relative to the current row.

    Use the provided classmethods to construct specific frame boundaries.
    """

    @classmethod
    def current_row(cls) -> typing.Self:
        """The boundary is the current row being processed."""
        ...

    @classmethod
    def following(cls, val: int) -> typing.Self:
        """
        The boundary is a fixed number of rows/values after the current row.

        Args:
            val: The number of rows or the value range following the current row.
        """
        ...

    @classmethod
    def preceding(cls, val: int) -> typing.Self:
        """
        The boundary is a fixed number of rows/values before the current row.

        Args:
            val: The number of rows or the value range preceding the current row.
        """
        ...

    @classmethod
    def unbounded_following(cls) -> typing.Self:
        """The boundary is the last row of the partition."""
        ...

    @classmethod
    def unbounded_preceding(cls) -> typing.Self:
        """The boundary is the first row of the partition."""
        ...

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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=4.21, image="123")
            .to_sql("mysql")
        )
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (4.21, '123')
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def columns(self, *args: _ColumnRefNew) -> typing.Self:
        """
        Specify (override) the columns for insertion.

        There's no need to use this method when you're specifying column
        names in `.values` method.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .columns("aspect", "image")
            .values(5.15, "12A")
            .values(4.21, "123")
            .to_sql("mysql")
        )
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (5.15, '12A'), (4.21, '123')
        ```
        """
        ...

    def into(self, table: _TableNameNew) -> typing.Self:
        """
        Override the target table for insertion.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("name_1")
            .values(aspect=4.21, image="123")
            .into("name_2")
            .to_sql("sqlite")
        )
        # INSERT INTO "name_2" ("aspect", "image") VALUES (4.21, '123')
        ```
        """
        ...

    def on_conflict(self, action: OnConflict) -> typing.Self:
        """Specify conflict resolution behavior (UPSERT)."""
        ...

    def or_default_values(self, rows: int = 1) -> typing.Self:
        """
        Use DEFAULT VALUES if no values were specified. The `rows`
        Specifies number of rows to insert with default values.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.InsertStatement("glyph").or_default_values().to_sql("postgres")
        # INSERT INTO "glyph" VALUES (DEFAULT)

        stmt = (
            rq.InsertStatement("glyph")
            .or_default_values()
            .values(aspect=6.7)
            .to_sql("postgres")
        )
        # INSERT INTO "glyph" ("aspect") VALUES (6.7)
        ```
        """
        ...

    def replace(self) -> typing.Self:
        """
        Convert this INSERT to a REPLACE statement.

        REPLACE will delete existing rows that conflict with the new row
        before inserting.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .replace()
            .values(aspect=5.15, image="12A")
            .to_sql("sqlite")
        )
        # REPLACE INTO "glyph" ("aspect", "image") VALUES (5.15, '12A')
        ```
        """
        ...

    def returning(self, clause: Returning) -> typing.Self:
        """
        Specify columns to return from the inserted rows.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(image="12A")
            .returning(rq.Returning.all())
            .to_sql("sqlite")
        )
        # INSERT INTO "glyph" ("image") VALUES ('12A') RETURNING *

        stmt = (
            rq.InsertStatement("glyph")
            .values(image="12A")
            .returning(rq.Returning("id"))
            .to_sql("sqlite")
        )
        # INSERT INTO "glyph" ("image") VALUES ('12A') RETURNING "id"
        ```
        """
        ...

    def select_from(self, statement: SelectStatement) -> typing.Self:
        """
        Specify a select query whose values to be inserted. Raises `ValueError` if
        `self`s columns length and `statement`s columns length has mismatch.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .columns("aspect", "image")
            .select_from(
                rq.SelectStatement(rq.Expr.col("aspect"), rq.Expr.col("image"))
                .from_table("glyph")
                .where(rq.Expr.col("image").like("0%"))
            )
            .to_sql("mysql")
        )
        # INSERT INTO `glyph` (`aspect`, `image`)
        # SELECT `aspect` AS `aspect`, `image` AS `image` FROM `glyph` WHERE `image` LIKE '0%'

        stmt = (
            rq.InsertStatement("glyph")
            .columns("image")
            .select_from(
                rq.SelectStatement("hello").where(
                    rq.not_(rq.Expr.exists(rq.SelectStatement("world")))
                )
            )
            .to_sql("postgres")
        )
        # INSERT INTO "glyph" ("image") SELECT 'hello' WHERE NOT EXISTS(SELECT 'world')
        ```
        """
        ...

    @typing.overload
    def values(self, *args: _ExprNew) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .columns("aspect", "image")
            .values(2, rq.Func.cast_as("2020-02-02 00:00:00", "DATE"))
            .to_sql("postgres")
        )
        # INSERT INTO "glyph" ("aspect", "image") VALUES (2, CAST('2020-02-02 00:00:00' AS DATE))
        ```
        """
        ...

    @typing.overload
    def values(self, **kwds: _ExprNew) -> typing.Self:
        """
        Specify values to insert. Also you can specify columns using keyword arguments.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=2, image=rq.Func.cast_as("2020-02-02 00:00:00", "DATE"))
            .to_sql("postgres")
        )
        # INSERT INTO "glyph" ("aspect", "image") VALUES (2, CAST('2020-02-02 00:00:00' AS DATE))
        ```
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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_update(image="ex"))
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd')
        # ON CONFLICT ("id") DO UPDATE SET "image" = 'ex'
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def action_where(self, condition: Expr) -> typing.Self:
        """
        Add a `WHERE` clause to the conflict action (conditional update).

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(
                rq.OnConflict("id")
                .do_update(image="ex")
                .action_where(rq.Expr.col("aspect").is_null())
            )
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd')
        # ON CONFLICT ("id") DO UPDATE SET "image" = 'ex' WHERE "aspect" IS NULL

        stmt.to_sql("mysql")
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.1415, 'abcd')
        # ON DUPLICATE KEY UPDATE `image` = 'ex'
        ```
        """
        ...

    def do_nothing(self, *keys: _ColumnRefNew) -> typing.Self:
        """
        Specify `DO NOTHING` action for conflicts.

        When a conflict occurs, the conflicting row will be skipped.

        `keys` parameter provides primary keys if you are using MySQL, for MySQL specific polyfill.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_nothing())
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd') ON CONFLICT ("id") DO NOTHING

        stmt.to_sql("mysql")
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.1415, 'abcd') ON DUPLICATE KEY IGNORE

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_nothing("id"))
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd') ON CONFLICT ("id") DO NOTHING

        stmt.to_sql("mysql")
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.1415, 'abcd') ON DUPLICATE KEY UPDATE `id` = `id`
        ```
        """
        ...

    def do_update(self, *args: _ColumnRefNew, **kwds: _ExprNew) -> typing.Self:
        """
        Specify `DO UPDATE` action for conflicts using column names, or with explicit values.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_update("aspect", image=rq.Expr(1) + 2))
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd')
        # ON CONFLICT ("id") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = 1 + 2

        stmt.to_sql("mysql")
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.1415, 'abcd')
        # ON DUPLICATE KEY UPDATE `aspect` = VALUES(`aspect`), `image` = 1 + 2
        ```
        """
        ...

    def target_where(self, condition: Expr) -> typing.Self:
        """
        Add a `WHERE` clause to the conflict target (partial unique index).

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(
                rq.OnConflict("id")
                .do_update(image="ex")
                .target_where(rq.Expr.col("aspect").is_null())
            )
        )
        stmt.to_sql("postgres")
        # INSERT INTO "glyph" ("aspect", "image") VALUES (3.1415, 'abcd')
        # ON CONFLICT ("id") WHERE "aspect" IS NULL DO UPDATE SET "image" = 'ex'

        stmt.to_sql("mysql")
        # INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.1415, 'abcd')
        # ON DUPLICATE KEY UPDATE `image` = 'ex'
        ```
        """
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

    def __copy__(self) -> typing.Self: ...
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

    def __copy__(self) -> typing.Self: ...
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

    def __copy__(self) -> typing.Self: ...
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

    Examples:
    ```python
    import rapidquery as rq

    stmt = (
        rq.SelectStatement()
        .columns("character", "fonts.name")
        .from_table("characters")
        .join("fonts", rq.Expr.col("characters.font_id") == rq.Expr.col("fonts.id"), "LEFT")
        .where(rq.Expr.col("size_w").in_((3, 4)))
        .where(rq.Expr.col("character").like("A%"))
        .to_sql("postgres")
    )
    # SELECT "character" AS "character", "fonts"."name" AS "name" FROM "characters"
    # LEFT JOIN "fonts" ON "characters"."font_id" = "fonts"."id"
    # WHERE "size_w" IN (3, 4) AND "character" LIKE 'A%'
    ```
    """

    def __init__(self, *exprs: SelectLabel | _ExprNew) -> None:
        """
        Construct a new `SELECT` statement.

        Args:
            exprs: Select expressions.

        Examples:
        ```python
        import rapidquery as rq

        stmt = rq.SelectStatement(1, "hello", "font").to_sql("postgres")
        # SELECT 1, 'hello', 'font'
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def columns(self, *args: _ColumnRefNew) -> typing.Self:
        """
        Add select target expressions.

        Works same as `self.exprs(Expr.col(i) for i in args)`, but much easier and faster.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("character", "size_w", "size_h")
            .from_table("characters")
            .to_sql("mysql")
        )
        # SELECT `character` AS `character`, `size_w` AS `size_w`, `size_h` AS `size_h` FROM `characters`
        ```
        """
        ...

    def distinct(self, *on: _ColumnRefNew) -> typing.Self:
        """
        Changes `SELECT` statement into `SELECT DISTINCT` statement.

        Args:
            on: Column references. If specified, uses *Postgres-ONLY* `SELECT DISTINCT ON ...` syntax.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .distinct()
            .columns("character", "size_w", "size_h")
            .from_table("characters")
            .to_sql("mysql")
        )
        # SELECT DISTINCT `character` AS `character`, `size_w` AS `size_w`, `size_h` AS `size_h` FROM `characters`

        stmt = (
            rq.SelectStatement()
            .distinct("character")
            .columns("character", "size_w", "size_h")
            .to_sql("postgres")
        )
        # SELECT DISTINCT ON ("character") "character" AS "character", "size_w" AS "size_w", "size_h" AS "size_h"
        ```
        """
        ...

    def exprs(self, *args: SelectLabel | _ExprNew) -> typing.Self:
        """
        Add select target expressions.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .exprs(rq.Func.max(rq.Expr.col("id")), rq.Expr.val(0) + 1 + 2 + 3)
            .from_table("characters")
            .to_sql("sqlite")
        )
        # SELECT MAX("id"), 0 + 1 + 2 + 3 FROM "characters"
        ```
        """
        ...

    def from_function(self, function: Expr | Func, alias: str) -> typing.Self:
        """
        Select from function call.

        Args:
            function: An expression or a function. If it is `Expr`, it should be a function call
                    and nothing more, or `ValueError` will be raised.
            alias: label, i.e. `FROM <function> AS <alias>`.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(rq.Expr.asterisk())
            .from_function(rq.Func.random(), "func")
            .to_sql("postgres")
        )
        # SELECT * FROM RANDOM() AS "func"
        ```
        """
        ...

    def from_subquery(self, subquery: SelectStatement, alias: str) -> typing.Self:
        """
        Select from `SELECT` subquery.

        Args:
            subquery: A select statement.
            alias: label, i.e. `FROM <subquery> AS <alias>`.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("image")
            .from_subquery(
                rq.SelectStatement().columns("image", "aspect").from_table("glyph"),
                "subglyph",
            )
            .to_sql("postgres")
        )
        # SELECT "image" AS "image" FROM
        # (SELECT "image" AS "image", "aspect" AS "aspect" FROM "glyph") AS "subglyph"
        ```
        """
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """
        Select from table.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(rq.Expr.asterisk())
            .from_table("characters")
            .from_table("fonts")
            .where(rq.Expr.col("fonts.id") == rq.Expr.col("characters.font_id"))
            .to_sql("postgres")
        )
        # SELECT * FROM "characters", "fonts" WHERE "fonts"."id" = "characters"."font_id"
        ```
        """
        ...

    def group_by(self, *groups: Expr | _ColumnRefNew) -> typing.Self:
        """
        Add group by expressions or column references.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .group_by(rq.Expr.col("size_w"), rq.Expr.col("size_h"))
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" GROUP BY "size_w", "size_h"
        ```
        """
        ...

    def having(self, condition: Expr) -> typing.Self:
        """
        Add having condition expression.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(rq.Func.max(rq.Expr.col("image")))
            .columns("aspect")
            .from_table("glyph")
            .group_by(rq.Expr.col("aspect"))
            .having(rq.Expr.col("aspect") > 2)
            .having(rq.Expr.col("aspect") < 8)
            .to_sql("postgres")
        )
        # SELECT MAX("image"), "aspect" AS "aspect" FROM "glyph"
        # GROUP BY "aspect" HAVING "aspect" > 2 AND "aspect" < 8
        ```
        """
        ...

    def join(
        self,
        table: _TableNameNew,
        on: Expr | None = None,
        type: typing.Literal["CROSS", "FULL", "INNER", "LEFT", "RIGHT"] | None = None,
    ) -> typing.Self:
        """
        Join with other table.

        Args:
            table: The table to join with.
            on: Join condition.
            type: Join type.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("character", "fonts.name")
            .from_table("characters")
            .join(
                "fonts",
                rq.Expr.col("characters.font_id") == rq.Expr.col("fonts.id"),
                "RIGHT",
            )
            .to_sql("mysql")
        )
        # SELECT `character` AS `character`, `fonts`.`name` AS `name` FROM `characters`
        # RIGHT JOIN `fonts` ON `characters`.`font_id` = `fonts`.`id`
        ```
        """
        ...

    def join_function(
        self,
        function: Func | Expr,
        alias: str,
        on: Expr | None = None,
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
        on: Expr | None = None,
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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("name")
            .from_table("fonts")
            .join_subquery(
                rq.SelectStatement().columns("id").from_table("glyph"),
                "sub_glyph",
                rq.Expr.col("fonts.id") == rq.Expr.col("sub_glyph.id"),
                "LEFT",
            )
            .to_sql("mysql")
        )
        # SELECT `name` AS `name` FROM `fonts`
        # LEFT JOIN (SELECT `id` AS `id` FROM `glyph`) AS `sub_glyph` ON `fonts`.`id` = `sub_glyph`.`id`
        ```
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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .where(rq.Expr.col("id") == 5)
            .lock()
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" WHERE "id" = 5 FOR UPDATE

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .where(rq.Expr.col("id") == 5)
            .lock(tables=["glyph"])
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" WHERE "id" = 5 FOR UPDATE OF "glyph"

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .where(rq.Expr.col("id") == 5)
            .lock(behavior="NOWAIT")
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" WHERE "id" = 5 FOR UPDATE NOWAIT

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .where(rq.Expr.col("id") == 5)
            .lock("SHARE", behavior="SKIP", tables=["glyph"])
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" WHERE "id" = 5 FOR SHARE OF "glyph" SKIP LOCKED
        ```
        """
        ...

    def offset(self, n: int) -> typing.Self:
        """Set offset."""
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """Specify the order in which to delete rows."""
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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement("hello")
            .union(rq.SelectStatement("world"), "ALL")
            .to_sql("postgres")
        )
        # SELECT 'hello' UNION ALL (SELECT 'world')
        ```
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """
        Add select `WHERE` condition.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement()
            .columns("character")
            .from_table("characters")
            .where(rq.Expr.col("id") == 5)
            .where(rq.Expr.col("name").like("%A"))
            .to_sql("postgres")
        )
        # SELECT "character" AS "character" FROM "characters" WHERE "id" = 5 AND "name" LIKE '%A'
        ```
        """
        ...

    def window(self, name: str, statement: WindowStatement) -> typing.Self:
        """
        Add `WINDOW` to statement.

        Args:
            name: Window name.
            statement: Window statement.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.SelectStatement(
                rq.Expr.col("character").label("w", "C")
            )
            .window("C", rq.WindowStatement("font_size"))
            .from_table("characters")
            .to_sql("postgres")
        )
        # SELECT "character" OVER "C" AS "w" FROM "characters" WINDOW "C" AS (PARTITION BY "font_size")
        ```
        """
        ...

    def to_expr(self) -> Expr:
        """Shorthand for `Expr(self)`"""
        ...

    def label(
        self, alias: str, window: WindowStatement | str | None = None
    ) -> SelectLabel:
        """Shorthand for `SelectLabel(self, alias, window)`"""
        pass

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

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .where(rq.Expr.col("id") == 1)
            .to_sql("sqlite")
        )
        # UPDATE "glyph" SET "aspect" = 1.23, "image" = 123 WHERE "id" = 1
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def clear_order_by(self) -> typing.Self:
        """Remove orders from statement."""
        ...

    def clear_where(self) -> typing.Self:
        """Remove where conditions from statement."""
        ...

    def from_table(self, table: _TableNameNew) -> typing.Self:
        """
        Update using data from another table (`UPDATE .. FROM ..`).

        MySQL doesn't support the UPDATE FROM syntax. And the current implementation attempt to
        tranform it to the UPDATE JOIN syntax, which only works for one join target.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("glyph")
            .values(tokens=rq.Expr.col("characters.character"))
            .from_table("characters")
            .where(rq.Expr.col("glyph.image") == rq.Expr.col("characters.user_data"))
        )
        stmt.to_sql("postgres")
        # UPDATE "glyph" SET "tokens" = "characters"."character" FROM "characters" WHERE "glyph"."image" = "characters"."user_data"

        stmt.to_sql("mysql")
        # UPDATE `glyph` JOIN `characters` ON `glyph`.`image` = `characters`.`user_data` SET `glyph`.`tokens` = `characters`.`character`
        ```
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
        """
        Specify columns to return from the inserted rows.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .returning(rq.Returning.all())
            .to_sql("sqlite")
        )
        # UPDATE "glyph" SET "aspect" = 1.23, "image" = 123 RETURNING *

        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .returning(rq.Returning("id"))
            .to_sql("sqlite")
        )
        # UPDATE "glyph" SET "aspect" = 1.23, "image" = 123 RETURNING "id"
        ```
        """
        ...

    def table(self, table: _TableNameNew) -> typing.Self:
        """
        Override the table to update.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("name_1")
            .values(aspect=1.23, image=123)
            .table("name_2")
            .to_sql("sqlite")
        )
        # UPDATE "name_2" SET "aspect" = 1.23, "image" = 123
        ```
        """
        ...

    def values(self, **kwds: _ExprNew) -> typing.Self:
        """
        Specify columns and their new values.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .values(font_id=20)
            .to_sql("sqlite")
        )
        # UPDATE "name_2" SET "aspect" = 1.23, "image" = 123, "font_id" = 20
        ```
        """
        ...

    def where(self, condition: Expr) -> typing.Self:
        """
        Add a WHERE condition to filter rows to update.

        Examples:
        ```python
        import rapidquery as rq

        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .where(rq.Expr.col("id") > 10)
            .where(rq.Expr.col("id") < 20)
            .to_sql("sqlite")
        )
        # UPDATE "glyph" SET "aspect" = 1.23, "image" = 123 WHERE "id" > 10 AND "id" < 20
        ```
        """
        ...

class WindowStatement:
    """
    Represents an `OVER` clause used with window functions.

    Window functions perform calculations across a set of table rows that are
    somehow related to the current row. This class builds the specification
    of that set (the window), including partitioning, ordering, and framing.

    # References:
    1. <https://dev.mysql.com/doc/refman/8.0/en/window-function-descriptions.html>
    2. <https://www.sqlite.org/windowfunctions.html>
    3. <https://www.postgresql.org/docs/current/tutorial-window.html>
    """

    def __init__(self, *partition_by: Expr | _ColumnRefNew) -> None:
        """
        Construct a new `WINDOW` specification.

        Args:
            *partition_by: One or more columns or expressions to divide
                the result set into partitions.

        Examples:
        ```python
        import rapidquery as rq

        # Example 1: Defining a simple frame
        stmt = (
            rq.SelectStatement(
                rq.Expr.col("character").label(
                    "C",
                    rq.WindowStatement("font_size").frame(
                        "ROWS",
                        rq.Frame.preceding(10),
                    )
                )
            )
            .from_table("characters")
            .to_sql("postgres")
        )

        # Example 2: Defining a frame with start and end
        stmt = (
            rq.SelectStatement(
                rq.Expr.col("character").label(
                    "C",
                    rq.WindowStatement("font_size").frame(
                        "ROWS",
                        rq.Frame.unbounded_preceding(),
                        rq.Frame.unbounded_following(),
                    )
                )
            )
            .from_table("characters")
            .to_sql("postgres")
        )
        ```
        """
        ...

    def __copy__(self) -> typing.Self: ...
    def __repr__(self, /) -> str: ...
    def frame(
        self,
        frame_type: typing.Literal["ROWS", "RANGE"],
        frame_start: Frame,
        frame_end: Frame | None = None,
    ) -> typing.Self:
        """
        Set the window frame clause.

        Args:
            frame_type: Either "ROWS" (physical rows) or "RANGE" (logical values).
            frame_start: The starting boundary of the frame.
            frame_end: Optional ending boundary. If not provided, the frame
                typically defaults to the current row or the database default.
        """
        ...

    def order_by(self, clause: Ordering) -> typing.Self:
        """
        Specify the order of rows within each partition.

        Args:
            clause: An `Ordering` instance defining the sort key and direction.
        """
        ...

    def partition(self, partition_by: Expr | _ColumnRefNew) -> typing.Self:
        """
        Add a column or expression to the `PARTITION BY` clause.

        Args:
            partition_by: The column or expression to partition by.
        """
        ...

_CommonTableExpressionQuery = (
    SelectStatement | DeleteStatement | UpdateStatement | InsertStatement
)

class WithClause:
    """
    A WITH clause can contain one or multiple common table expressions (CTEs).

    These named queries can act as a "query local table" that are materialized during execution and
    then can be used by the query prefixed with the WITH clause.

    A CTE is a name, column names and a query returning data for those columns.

    Some databases (like sqlite) restrict the acceptable kinds of queries inside of the WITH clause
    CTEs. These databases only allow `SelectStatement`s to form a CTE.

    Other databases like postgres allow modification queries (UPDATE, DELETE) inside of the WITH
    clause but they have to return a table. (They must have a RETURNING clause).

    RapidQuery doesn't check this or restrict the kind of CTE that you can create
    in rust. This means that you can put an UPDATE or DELETE queries into WITH clause and RapidQuery
    will succeed in generating that kind of sql query but the execution inside the database will
    fail because they are invalid.

    It is your responsibility to ensure that the kind of WITH clause that you put together makes
    sense and valid for that database that you are using.

    NOTE that for recursive WITH queries (in sql: "WITH RECURSIVE") you can only have a
    single CTE inside of the WITH clause. That query must match certain
    requirements:
      * It is a query of UNION or UNION ALL of two queries.
      * The first part of the query (the left side of the UNION) must be executable first in itself.
        It must be non-recursive. (Cannot contain self reference)
      * The self reference must appear in the right hand side of the UNION.
      * The query can only have a single self-reference.
      * Recursive data-modifying statements are not supported, but you can use the results of a
        recursive SELECT query in a data-modifying statement. (like so: WITH RECURSIVE
        cte_name(a,b,c,d) AS (SELECT ... UNION SELECT ... FROM ... JOIN cte_name ON ... WHERE ...)
        DELETE FROM table WHERE table.a = cte_name.a)

    Note that this type is not a `QueryStatement`. To generate `WITH` statement, you must convert this type to
    `WithQuery` type, using `WithClause.query` method.
    """

    def __init__(self) -> None:
        """
        Construct a new `WithClause` instance.

        Note that this type is not a `QueryStatement`. To generate `WITH` statement, you must convert this type to
        `WithQuery` type, using `WithClause.query` method.

        Examples:
        ```python
        import rapidquery as rq

        select = (
            rq.SelectStatement()
            .columns("names.id", "names.name")
            .from_table("filtered_names AS fil")
            .join("names", rq.Expr.col("fil.name_id") == rq.Expr.col("names.id"))
        )
        clause = (
            rq.WithClause()
            .cte(
                "source_filtered_names",
                select.__copy__().where(rq.Expr.col("fil.user_id") == 5),
            )
            .cte(
                "dest_filtered_names",
                select.where(rq.Expr.col("fil.user_id") == 8),
            )
            .cte(
                "missing_names",
                rq.SelectStatement()
                .columns("dest.name")
                .from_table("dest_filtered_names AS dest")
                .join(
                    "source_filtered_names AS source",
                    rq.Expr.col("dest.id") == rq.Expr.col("source.id"),
                )
                .where(rq.Expr.col("source.id").is_null()),
            )
            .query(rq.SelectStatement().columns("name").from_table("missing_names"))
        )
        clause.to_sql("postgres")
        # WITH
        #   "source_filtered_names" ("id", "name") AS (
        #       SELECT "names"."id" AS "id", "names"."name" AS "name" FROM "filtered_names" AS "fil"
        #       JOIN "names" ON "fil"."name_id" = "names"."id" WHERE "fil"."user_id" = 5
        #   ) ,
        #   "dest_filtered_names" ("id", "name") AS (
        #       SELECT "names"."id" AS "id", "names"."name" AS "name" FROM "filtered_names" AS "fil"
        #       JOIN "names" ON "fil"."name_id" = "names"."id"
        #       WHERE "fil"."user_id" = 8
        #   ) ,
        #   "missing_names" ("name") AS (
        #       SELECT "dest"."name" AS "name" FROM "dest_filtered_names" AS "dest"
        #       JOIN "source_filtered_names" AS "source" ON "dest"."id" = "source"."id"
        #       WHERE "source"."id" IS NULL
        #   )
        #   SELECT "name" AS "name" FROM "missing_names"
        ```
        """
        ...

    def recursive(self) -> typing.Self:
        """
        Sets whether this clause is a recursive with clause of not.
        It will generate a 'WITH RECURSIVE' query.
        """
        ...

    def cte(
        self,
        name: str,
        query: _CommonTableExpressionQuery,
        columns: typing.Iterable[str] = (),
        materialized: bool | None = None,
    ) -> typing.Self:
        """
        Add a CTE to this with clause.

        Args:
            name: The common table expression (CTE) name.
            query: The query, which have to return data.
            columns: Named columns for the CTE table definition.
                     If empty, tries to detect them from the `query`.
            materialized: Some databases allow you to put “MATERIALIZED” or “NOT MATERIALIZED” in the
                          CTE definition. This will affect how during the execution of WithQuery the
                          CTE in the WithClause will be executed. If the database doesn’t support this
                          syntax this option specified here will be ignored and not appear in the
                          generated sql.
        """
        ...

    def cycle(
        self,
        expr: _ExprNew | None = None,
        set_as: str | None = None,
        using: str | None = None,
    ) -> typing.Self:
        """
        For recursive `WithQuery` `WithClause`s the CYCLE sql clause can be specified to avoid creating
        an infinite traversals that loops on graph cycles indefinitely. You specify an expression that
        identifies a node in the graph and that will be used to determine during the iteration of
        the execution of the query when appending of new values whether the new values are distinct new
        nodes or are already visited and therefore they should be added again into the result.

        A query can have both SEARCH and CYCLE clauses.

        This setting is not meaningful if the query is not recursive.
        Some databases don’t support this clause. In that case this option will be silently ignored.

        Examples:
        ```python
        base_query = rq.SelectStatement(
            rq.Expr.col("id"),
            1,
            rq.Expr.col("next"),
            rq.Expr.col("value"),
        ).from_table("table")

        cte_referencing = (
            rq.SelectStatement(
                rq.Expr.col("id"),
                rq.Expr.col("depth") + 1,
                rq.Expr.col("next"),
                rq.Expr.col("value"),
            )
            .from_table("table")
            .join(
                "cte_traversal",
                rq.Expr.col("cte_traversal.next") == rq.Expr.col("table.id"),
                "INNER",
            )
        )

        clause = (
            rq.WithClause()
            .recursive()
            .cte(
                "cte_traversal",
                base_query.union(cte_referencing, "ALL"),
                columns=["id", "depth", "next", "value"],
            )
            .cycle(rq.Expr.col("id"), set_as="looped", using="traversal_path")
            .query(rq.SelectStatement().columns("*").from_table("cte_traversal"))
        )
        clause.to_sql("postgres")
        # WITH RECURSIVE "cte_traversal" ("id", "depth", "next", "value") AS (
        #   SELECT "id" AS "id", 1, "next" AS "next", "value" AS "value" FROM "table"
        #   UNION ALL (
        #       SELECT "id" AS "id", "depth" + 1, "next" AS "next", "value" AS "value" FROM "table"
        #       INNER JOIN "cte_traversal" ON "cte_traversal"."next" = "table"."id"
        #   )
        # )
        # CYCLE "id" SET "looped" USING "traversal_path" SELECT * FROM "cte_traversal"
        ```
        """
        ...

    def search(
        self,
        expr: SelectLabel | None = None,
        order: typing.Literal["BREADTH", "DEPTH"] | str | None = None,
    ) -> typing.Self:
        """
        For recursive `WithQuery` `WithClause`s the traversing order can be specified in some databases
        that support this functionality.

        A query can have both SEARCH and CYCLE clauses.

        This setting is not meaningful if the query is not recursive.
        Some databases don’t support this clause. In that case this option will be silently ignored.

        The `SelectLabel` used must specify an alias which will be the name that you can use to order
        the result of the CTE.

        Examples:
        ```python
        base_query = (
            rq.SelectStatement()
            .columns("id", "parent_id")
            .from_table("nodes")
            .where(rq.Expr.col("parent_id").is_null())
        )
        reference_query = (
            rq.SelectStatement()
            .columns("n.id", "n.parent_id")
            .from_table("nodes as n")
            .join(
                "tree as t",
                rq.Expr.col("n.parent_id") == rq.Expr.col("t.id"),
            )
        )

        clause = (
            rq.WithClause()
            .recursive()
            .cte(
                "tree",
                base_query.union(reference_query, "ALL"),
                columns=["id", "parent_id"],
            )
            .search(rq.Expr.col("id").label("traversal_order"), "DEPTH")
            .query(
                rq.SelectStatement()
                .columns("*")
                .from_table("tree")
                .order_by(rq.Ordering("traversal_order"))
            )
        )
        clause.to_sql("postgresql")
        # WITH RECURSIVE "tree" ("id", "parent_id") AS (
        #   SELECT "id" AS "id", "parent_id" AS "parent_id" FROM "nodes"
        #   WHERE "parent_id" IS NULL
        #   UNION ALL (
        #       SELECT "n"."id" AS "id", "n"."parent_id" AS "parent_id"
        #       FROM "nodes" AS "n"
        #       JOIN "tree" AS "t" ON "n"."parent_id" = "t"."id"
        #   )
        # )
        # SEARCH DEPTH FIRST BY "id" SET "traversal_order"
        # SELECT * FROM "tree" ORDER BY "traversal_order" ASC
        ```
        """
        ...

    def query(self, val: _CommonTableExpressionQuery) -> WithQuery:
        """
        You can turn this into a `WithQuery` using this function.
        The resulting WITH query will execute the argument query with this WITH clause.
        """
        ...

class WithQuery(QueryStatement):
    """
    A WITH query. A simple SQL query that has a WITH clause (`WithClause`).

    It's recommended to use `WithClause.query` method instead of directly constructing
    this type.

    For full description, see `WithClause`'s documentation.
    """

    def __init__(
        self,
        clause: WithClause,
        query: _CommonTableExpressionQuery,
    ) -> None:
        """
        Construct a new `WithQuery` instance.

        Args:
            clause: The `WITH` clause.
            query: The final query.
        """
        ...
