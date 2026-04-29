# RapidQuery
__*RapidQuery: High-Performance SQL Query Builder for Python*__

RapidQuery is a powerful SQL query builder library designed for Python, combining the simplicity of Python with the raw speed of **Rust**. Build complex SQL queries effortlessly and efficiently, with a library that prioritizes both performance and ease of use.

- [**Installation**](#install)
- [**Why RapidQuery?**](#why-rapidquery)
- [**Supported Backends**](#backends)
- [**Usage**](#usage)
- [**Performance**](#performance)
- [**Known Issues**](#known-issues)
- [**Supported Platforms**](#supported-platforms)
- [**License**](#license)

## Install

You can use **PIP**:
```bash
pip3 install rapidquery
```

Or you can use **UV** (recommended):
```bash
uv add rapidquery
```

## Why RapidQuery?
**Features**:
- **🧶 Thread safe**: It's completely thread-safe and uses locks in internal to prevent concurrency problems.
- **⚡ Blazing High Performance**: Leveraging the power of Rust under the hood, RapidQuery ensures your query building process is as fast as possible.
- 🛡️ **SQL Injection Protection**: Built-in security measures to prevent SQL injection attacks by default.
- 📝 **Intuitive Pythonic API**: Write clean, readable code with an API that feels natural to Python developers.
- 🔥 **Built on Solid Foundations**: RapidQuery is built with **Rust** and powered by the robust **SeaQuery** crate, bringing enterprise-grade reliability and performance to your Python applications.

> **Why RapidQuery Was Created** \
> In a landscape filled with SQL libraries, we noticed a critical gap: **performance was often an afterthought**. That's why we built RapidQuery with speed as our primary and enduring focus.

## Backends
RapidQuery supports `PostgreSQL`, `MySQL`, and `SQLite`. These are referred to as `backend`s. When building SQL statements, you must specify your target backend.

## Usage

1. Core Concepts
    1. [**Value**](#value)
    2. [**Expr**](#expr)
    3. [**Statement Builders**](#statement-builders)
2. Query Statements
    1. [**Query Select**](#query-select)
    2. [**Query Insert**](#query-insert)
    3. [**Query Update**](#query-update)
    4. [**Query Delete**](#query-delete)
    5. [**Query With**](#query-with)
3. More About Queries
    1. [**Custom Function**](#custom-functions)
4. Schema Statements
    1. [**Table Create**](#table-create)
    2. [**Table Alter**](#table-alter)
    3. [**Table Drop**](#table-drop)
    4. [**Table Rename**](#table-rename)
    5. [**Table Truncate**](#table-truncate)
    8. [**Index Create**](#index-create)
    9. [**Index Drop**](#index-drop)
5. Advanced Usage
    1. [**More About Column References**](#more-about-column-references)
    2. [**More About TableName**](#more-about-tablename)
    3. [**More About Expr**](#more-about-expr)

### Core Concepts
#### Value
Bridges Python types, Rust types, and SQL types for seamless data conversion.

This class handles validation, adaptation, and conversion between different
type systems used in the application stack.

> [!NOTE]\
> this class is immutable and frozen.

> [!TIP]\
> **Important**: `Value` is lazy. This means it keeps your value and never converts it to Rust and then SQL until needed.

#### Expr
Represents a SQL expression that can be built into SQL code.

This class provides a fluent interface for constructing complex SQL expressions
in a database-agnostic way. It supports arithmetic operations, comparisons,
logical operations, and database-specific functions.

The class automatically handles SQL injection protection and proper quoting
when building the final SQL statement.

> [!NOTE]\
> `Expr` is immutable, so by calling each method you will give a new instance
of it which includes new change(s).

**Basic**
```python
import rapidquery as rp

rp.Expr(25)                         # -> 25  (literal value)
rp.Expr("Hello")                    # -> 'Hello'  (literal value)
rp.Expr(rq.Value('World'))          # -> 'World'  (literal value)

rp.Expr.col("id")                             # -> "id" (column reference)
rp.Expr.col("users.name")                     # -> "users"."name" (column reference)
rp.Expr(rq.ColumnRef("name", table="users"))  # -> "users"."name" (column reference)
```

**Comparisons**
```python
rq.Expr.col("status") == "active"  # -> "status" == 'active'
rq.Expr.col("age") > 16           # -> "age" > 16

# Note that `rq.all` is different from built-in `all`
rq.all(
    rq.Expr.col("age") >= 18,
    rq.Expr.col("subscription").is_null(), # same as rq.Expr.col("subscription").is_(Expr.null())
    rq.Expr.col("status").in_(["pending", "approved", "active"])
)    # -> "age" >= 18 AND "subscription" IS NULL AND "status" IN ('pending', 'approved', 'active')

# Note that `rq.any` is different from built-in `any`
rq.any(
    rq.Expr.col("is_admin").is_(True),
    rq.Expr.col("is_moderator").is_not_null(), # same as rq.Expr.col("subscription").is_not(Expr.null())
    rq.Expr.col("price").between(10.00, 50.00)
)    # -> "is_admin" IS TRUE OR "is_moderator" IS NOT NULL OR "price" BETWEEN 10.00 AND 50.00
```

**Best Practices**
- Always use `Expr.col()` for column references: This ensures proper quoting for your target database
```python
# Column reference (properly quoted identifier)
rq.Expr.col("user_name")  # → "user_name"

# String literal (value)
rq.Expr("user_name")      # → 'user_name'
```

- Use `rapidquery.all()` and `rapidquery.any()` for logical combinations: More readable than chaining `&` and `|` operators
```python
# Good
all(condition1, condition2, condition3)
   
# Less readable
condition1 & condition2 & condition3
```

- Be careful with `Expr.custom()`: It bypasses all safety checks
```python
# Dangerous - vulnerable to SQL injection
user_input = "'; DROP TABLE users; --"
Expr.custom(f"name = '{user_input}'")

# Safe
Expr.col("name") == user_input
```

#### Statement Builders
Statements are divided into 2 categories: `QueryStatement`, and `SchemaStatement`.

Some statements like `Select`, `Update`, `Delete`, `Insert`, ... are `QueryStatement`.
Other statements like `Table`, `AlterTable`, `Index`, ... are `SchemaStatement`.

`QueryStatement` class interface is:
```python
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
```

`SchemaStatement` class interface is:
```python
class SchemaStatement:
    """Subclass of schema statements."""

    def to_sql(self, backend: _BackendName, /) -> str:
        """Build a SQL string representation."""
        ...
```

### Query Select
Use `rapidquery.SelectStatement` type to generate `SELECT` statements.

```python
import rapidquery as rq

stmt = (
    rq.SelectStatement()
    .columns("character", "fonts.name")
    .from_table("characters")
    .join("fonts", rq.Expr.col("characters.font_id") == rq.Expr.col("fonts.id"), "LEFT")
    .where(rq.Expr.col("size_w").in_([3, 4]))
    .where(rq.Expr.col("characters").like("A%"))
)
print(stmt.to_sql("postgres"))
# SELECT "character" AS "character", "fonts"."name" AS "name" FROM "characters"
# LEFT JOIN "fonts" ON "characters"."font_id" = "fonts"."id"
# WHERE "size_w" IN (3, 4) AND "characters" LIKE 'A%'

print(stmt.to_sql("mysql"))
# SELECT `character` AS `character`, `fonts`.`name` AS `name` FROM `characters`
# LEFT JOIN `fonts` ON `characters`.`font_id` = `fonts`.`id`
# WHERE `size_w` IN (3, 4) AND `characters` LIKE 'A%'

print(stmt.to_sql("sqlite"))
# SELECT "character" AS "character", "fonts"."name" AS "name" FROM "characters"
# LEFT JOIN "fonts" ON "characters"."font_id" = "fonts"."id" 
# WHERE "size_w" IN (3, 4) AND "characters" LIKE 'A%'
```

### Query Insert
Use `rapidquery.InsertStatement` type to generate `INSERT` statements.

```python
import rapidquery as rq

stmt = (
    rq.InsertStatement("glyph")
    .values(aspect=3.14, image="A4")
    .on_conflict(rq.OnConflict("id").do_update("image"))
)

print(stmt.to_sql("postgres"))
# INSERT INTO "glyph" ("aspect", "image") VALUES (3.14, 'A4')
# ON CONFLICT ("id") DO UPDATE SET "image" = "excluded"."image"
# 
print(stmt.to_sql("mysql"))
# INSERT INTO `glyph` (`aspect`, `image`) VALUES (3.14, 'A4')
# ON DUPLICATE KEY UPDATE `image` = VALUES(`image`)

print(stmt.to_sql("sqlite"))
# INSERT INTO "glyph" ("aspect", "image") VALUES (3.14, 'A4')
# ON CONFLICT ("id") DO UPDATE SET "image" = "excluded"."image"
```

### Query Update
Use `rapidquery.UpdateStatement` type to generate `UPDATE` statements.

```python
import rapidquery as rq

stmt = (
    rq.UpdateStatement("glyph")
    .values(aspect=1.23, image="123")
    .where(rq.Expr.col("id") == 1)
)

print(stmt.to_sql("postgres"))
# UPDATE "glyph" SET "aspect" = 1.23, "image" = '123' WHERE "id" = 1

print(stmt.to_sql("mysql"))
# UPDATE `glyph` SET `aspect` = 1.23, `image` = '123' WHERE `id` = 1

print(stmt.to_sql("sqlite"))
# UPDATE "glyph" SET "aspect" = 1.23, "image" = '123' WHERE "id" = 1
```

### Query Delete
Use `rapidquery.DeleteStatement` type to generate `DELETE` statements.

```python
import rapidquery as rq

stmt = rq.DeleteStatement("glyph").where(
    rq.any(
        rq.Expr.col("id") < 1,
        rq.Expr.col("id") > 10,
    )
)

print(stmt.to_sql("postgres"))
# DELETE FROM "glyph" WHERE "id" < 1 OR "id" > 10

print(stmt.to_sql("mysql"))
# DELETE FROM `glyph` WHERE `id` < 1 OR `id` > 10

print(stmt.to_sql("sqlite"))
# DELETE FROM "glyph" WHERE "id" < 1 OR "id" > 10
```

### Query With
We have two types here: `rapidquery.WithClause` and `rapidquery.WithQuery`.

```txt
         WithQuery
             |
|------------------------|
WITH [... CTEs ...] QUERY
|------------------|
         |
     WithClause
```


As you can see, `rapidquery.WithClause` includes common table expressions (CTEs),
and `rapidquery.WithQuery` includes `rapidquery.WithClause` and the final query.

```python
import rapidquery as rq

clause = (
    rq.WithClause()
    .cte(
        "users_count",
        (
            rq.UpdateStatement("users")
            .values(amount=rq.Expr.col("amount") + 10)
            .where(rq.Expr.col("id") > 50)
            .returning(rq.Returning(rq.Expr.val(1)))
        ),
    )
    .cte(
        "teams_count",
        (
            rq.UpdateStatement("teams")
            .values(amount=rq.Expr.col("amount") + 10)
            .where(rq.Expr.col("id") > 50)
            .returning(rq.Returning(rq.Expr.val(1)))
        ),
    )
)

users_count_select = rq.SelectStatement(rq.Func.count(rq.Expr.asterisk())).from_table("users_count")
teams_count_select = rq.SelectStatement(rq.Func.count(rq.Expr.asterisk())).from_table("teams_count")

query: WithQuery = clause.query(
    rq.SelectStatement(
        users_count_select.label("users"),
        teams_count_select.label("teams"),
    )
)
query.to_sql("postgres")
# WITH 
#   "users_count" AS (
#       UPDATE "users" SET "amount" = "amount" + 10 WHERE "id" > 50 RETURNING 1
#   ) ,
#   "teams_count" AS (
#       UPDATE "teams" SET "amount" = "amount" + 10 WHERE "id" > 50 RETURNING 1
#   )
#   SELECT
#       (SELECT COUNT(*) FROM "users_count") AS "users",
#       (SELECT COUNT(*) FROM "teams_count") AS "teams"
```

### Custom Functions
For working with functions in RapidQuery, you have to use `Func` class.
A lot of functions such as `SUM`, `AVG`, `MD5`, ... is ready to use. For example:

```python
stmt = rq.SelectStatement(rq.Func.sum(rq.Expr.col("amount")))
stmt.to_sql("postgres")
# SELECT SUM("amount")
```

But for functions not provided by the library, you can define custom functions.
Custom functions can be defined using the `Func` constructor:

```python
stmt = rq.SelectStatement(rq.Func("CUSTOM", 1, 'hello'))
stmt.to_sql("postgres")
# SELECT CUSTOM(1, 'hello')
```

### Table Create
`rapidquery.Table` represents a complete database table definition. Use it to generate `CREATE TABLE` statements.

```python
import rapidquery as rq

characters = rq.Table(
    "characters",
    rq.Column("id", rq.sqltypes.Integer(), primary_key=True, auto_increment=True),
    rq.Column("font_size", rq.sqltypes.Integer(), nullable=False),
    rq.Column("character", rq.sqltypes.String(), nullable=False),
    rq.Column("size_w", rq.sqltypes.Integer(), nullable=False),
    rq.Column("size_h", rq.sqltypes.Integer(), nullable=False),
    rq.Column("font_id", rq.sqltypes.Integer(), default=None),
    rq.ForeignKey(["font_id"], ["fonts.id"], on_delete="CASCADE", on_update="CASCADE"),
    rq.Index("idx_character", ["character"]),
    if_not_exists=True,
)

print(characters.to_sql("postgresql"))
# CREATE TABLE IF NOT EXISTS "characters" (
#   "id" serial PRIMARY KEY,
#   "font_size" integer NOT NULL,
#   "character" varchar NOT NULL,
#   "size_w" integer NOT NULL,
#   "size_h" integer NOT NULL,
#   "font_id" integer DEFAULT NULL,
#   FOREIGN KEY ("font_id") REFERENCES "fonts" ("id") ON DELETE CASCADE ON UPDATE CASCADE
# );
# CREATE INDEX IF NOT EXISTS "idx_character" ON "characters" ("character")
```

### Table Alter
Use `rapidquery.AlterTable` type to generate `ALTER TABLE` statements.

```python
import rapidquery as rq

stmt = rq.AlterTable(
    "fonts",
    [
        rq.AlterTableAddColumnOption(
            rq.Column(
                "new_col",
                rq.sqltypes.Integer(),
                nullable=False,
                default=100,
            )
        ),
        rq.AlterTableRenameColumnOption("hello", "world"),
    ],
)
print(stmt.to_sql("mysql"))
# ALTER TABLE `fonts` ADD COLUMN `new_col` int NOT NULL DEFAULT 100,
# RENAME COLUMN `hello` TO `world`
```

### Table Drop
Use `rapidquery.DropTable` type to generate `DROP TABLE` statements.

```python
import rapidquery as rq

stmt = rq.DropTable("glyph", if_exists=True)
print(stmt.to_sql("mysql"))
# DROP TABLE IF EXISTS `glyph`
```

### Table Rename
Use `rapidquery.RenameTable` type to generate `RENAME TABLE` statements.

```python
import rapidquery as rq

stmt = rq.RenameTable("old", "new")

print(stmt.to_sql("sqlite"))
# ALTER TABLE "old" RENAME TO "new"

print(stmt.to_sql("mysql"))
# RENAME TABLE `old` TO `new`
```

### Table Truncate
Use `rapidquery.TruncateTable` type to generate `TRUNCATE TABLE` statements.

```python
import rapidquery as rq

stmt = rq.TruncateTable("old")

print(stmt.to_sql("mysql"))
# TRUNCATE TABLE `old`
```

### Index Create

```python
import rapidquery as rq

idx = rq.Index("idx_glyph_aspect", ["aspect"], "glyph")

print(idx.to_sql("postgres"))
# CREATE INDEX "idx_glyph_aspect" ON "glyph" ("aspect")
```

### Index Drop
Use `rapidquery.DropIndex` type to generate `DROP INDEX` statements.

```python
import rapidquery as rq

idx = rq.DropIndex("idx_glyph_aspect", "glyph")

print(idx.to_sql("postgres"))
# DROP INDEX "idx_glyph_aspect"
```

### More About Column Reference
Let's learn some tricks about column references.

In RapidQuery, we have something called `ColumnRef`, which represents a reference to a database column with optional table and schema qualification.

> This type is a final type, which means you cannot use it as subclass.

In generating statements, we have a lot of situations that you need to work with column references.

**❗ The Trick** \
For the parameters which accept column references, you have 4 ways:

1. Use `ColumnRef`:
```python
col_ref = rq.ColumnRef("id", "characters")
# OR
col_ref = rq.ColumnRef.parse("characters.id")

stmt = rq.SelectStatement().columns(col_ref)
# SELECT "characters"."id" AS "id"
```

2. Use `str`: The easiest way
```python
stmt = rq.SelectStatement().columns("characters.id")
# SELECT "characters"."id" AS "id"
```

3. Use `__column_ref__` property: developer-friendly and expandable way.
```python
class IdColumnProperty:
    @property
    def __column_ref__(self):
        # Can return ColumnRef or str
        return "characters.id"

class IdColumnClassVar:
    __column_ref__ = "characters.id"

stmt = rq.SelectStatement().columns(IdColumnProperty())
# SELECT "characters"."id" AS "id"

stmt = rq.SelectStatement().columns(IdColumnClassVar)
# SELECT "characters"."id" AS "id"
```

4. Use `Column`: It's possible because `Column` has `__column_ref__` property.
```python
id = Column("id", rq.Integer())

stmt = rq.SelectStatement().columns(id)
# SELECT "id" AS "id"
```


### More About TableName
Let's learn some tricks about table name.

In RapidQuery, we have something called `TableName`, which represents a table name reference with optional schema, database, and alias.

> This type is a final type, which means you cannot use it as subclass.

In generating statements, we have situations that you need to specify table name.

**❗ The Trick** \
For the parameters which accept table name, you have 4 ways:

1. Use `TableName`:
```python
tbl = rq.TableName("users", schema="archive")
# OR
tbl = rq.TableName.parse("archive.users")

stmt = rq.DeleteStatement(tbl)
# DELETE FROM "archive"."users"
```

2. Use `str`: The easiest way
```python
stmt = rq.DeleteStatement("archive.users")
# DELETE FROM "archive"."users"
```

3. Use `__table_name__` property: developer-friendly and expandable way.
```python
class UsersProperty:
    @property
    def __table_name__(self):
        # Can return TableName or str
        return "archive.users"


class UsersClassVar:
    __table_name__ = "archive.users"


stmt = rq.DeleteStatement(UsersProperty())
# DELETE FROM "archive"."users"

stmt = rq.DeleteStatement(UsersClassVar)
# DELETE FROM "archive"."users"
```

4. Use `Table`: It's possible because `Table` has `__table_name__` property.
```python
users = Table(
    "archive.users",
    ...
)

stmt = rq.DeleteStatement(users)
# DELETE FROM "archive"."users"
```

### More About Expr
You learned [here](#expr) about `Expr` type.

**❗ The Trick** \
This 2 tricks are notable and very good to know.

1. First, like `ColumnRef` and `TableName`, `Expr` supports `__expr__` property, which should always return `Expr`.

```python
class TextClause:
    def __init__(self, expr: str) -> None:
        self.expr = expr

    @property
    def __expr__(self) -> rq.Expr:
        return rq.Expr.custom(self.expr)


stmt = rq.SelectStatement(TextClause("WOW!"))
# SELECT WOW!
```

2. Second, same as `ColumnRef`, `Expr` also supports `__column_ref__` property.

## Performance
### Benchmarks
Benchmarks run on *Linux 6.18.12-1-MANJARO x86_64* with CPython 3.14. Your results may vary.

Iterations per test: 100,000 \
Python version: 3.14.3

```
📊 SELECT Query Benchmark
----------------------------------------------------------------------
Library              Time (ms)       vs Fastest      Status
----------------------------------------------------------------------
RapidQuery               245.44     1.00x (FASTEST) 🏆
PyPika                  4327.18     17.63x slower
SQLAlchemy              8818.33     35.93x slower
----------------------------------------------------------------------

📊 INSERT Query Benchmark
----------------------------------------------------------------------
Library              Time (ms)       vs Fastest      Status
----------------------------------------------------------------------
RapidQuery               640.63     1.00x (FASTEST) 🏆
PyPika                  4655.51     7.27x slower
SQLAlchemy              7085.74     11.06x slower
----------------------------------------------------------------------

📊 UPDATE Query Benchmark
----------------------------------------------------------------------
Library              Time (ms)       vs Fastest      Status
----------------------------------------------------------------------
RapidQuery               557.21     1.00x (FASTEST) 🏆
PyPika                  4488.96     8.06x slower
SQLAlchemy             11839.85     21.25x slower
----------------------------------------------------------------------

📊 DELETE Query Benchmark
----------------------------------------------------------------------
Library              Time (ms)       vs Fastest      Status
----------------------------------------------------------------------
RapidQuery               441.38     1.00x (FASTEST) 🏆
PyPika                  4517.16     10.23x slower
SQLAlchemy              7924.52     17.95x slower
----------------------------------------------------------------------
```

## Known Issues
### Unmanaged Rust panic output in building SQL
The library may encounter errors during SQL query construction, which are correctly raised as *RuntimeError* exceptions. For instance, this occurs when using a function that isn't supported by your target database. **While this error-raising behavior is intentional and logical, the issue is that unmanaged Rust panic information is also printed to stderr**. Currently, there is no way to suppress or manage this panic output. We are working to resolve this problem as much as possible in future updates.

```python
>>> import rapidquery as rq
>>> stmt = rq.TruncateTable("users")
>>> print(stmt.to_sql("sqlite"))

thread '<unnamed>' (14206) panicked at sea-query-0.32.7/src/backend/sqlite/table.rs:58:9:
Sqlite doesn't support TRUNCATE statement
Traceback (most recent call last):
  File "<python-input-3>", line 1, in <module>
    print(stmt.to_sql("sqlite"))
          ~~~~~~~~~~~^^^^^^^^^^
RuntimeError: build failed
```

### Missing `__init__` Calls
If a RapidQuery object is instantiated without calling its `__init__` method (e.g., via certain serialization tricks or `__new__` alone), the internal Rust pointer will be null. Accessing methods on such objects will cause an unmanaged Rust panic. Always use the provided constructors.

### Join conditions are not optional
Currently, `JOIN` operations require an explicit ON or USING condition.

## Supported Platforms

Pre-built wheels are available for the following platforms and Python interpreters:

### Linux (manylinux)

| Architecture | Python Versions |
|---|---|
| x86_64 | 3.10 – 3.14, 3.14t, PyPy 3.11, GraalPy 3.11/3.12 |
| i686 | 3.10 – 3.14, 3.14t, PyPy 3.11 |
| aarch64 | 3.10 – 3.14, 3.14t, PyPy 3.11, GraalPy 3.11/3.12 |
| armv7 | 3.10 – 3.14, 3.14t |
| s390x | 3.10 – 3.14, 3.14t |
| ppc64le | 3.10 – 3.14, 3.14t |
| riscv64 | 3.10 – 3.14, 3.14t |

### Linux (musllinux 1.1)

| Architecture | Python Versions |
|---|---|
| x86_64 | 3.10 – 3.14, 3.14t, PyPy 3.11 |
| aarch64 | 3.10 – 3.14, 3.14t, PyPy 3.11 |
| armv7 | 3.10 – 3.14, 3.14t, PyPy 3.11 |

### Windows

| Architecture | Python Versions |
|---|---|
| x64 | 3.10 – 3.14, 3.14t |
| x86 | 3.10 – 3.14, 3.14t |

### macOS

| Architecture | Python Versions |
|---|---|
| x86_64 | 3.10 – 3.14, 3.13t, 3.14t, PyPy 3.11 |
| aarch64 (Apple Silicon) | 3.10 – 3.14, 3.13t, 3.14t, PyPy 3.11 |

A source distribution (sdist) is also published for platforms not listed above, which requires a Rust nightly toolchain to build from source.

## TODO
- [x] Write tests
- [x] Update & automate workflows
- [x] Write CTE
- [x] Complete README.md
- [x] Bump version to 0.1.0
- [ ] Complete backend-only functions

## License
This repository is licensed under the [GNU GPLv3 License](LICENSE)
