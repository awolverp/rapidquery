# RapidQuery
__*RapidQuery: High-Performance SQL Query Builder for Python*__

RapidQuery is a powerful SQL query builder library designed for Python, combining the simplicity of Python with the raw speed and safety of **Rust**. Build complex SQL queries effortlessly and efficiently, with a library that prioritizes both performance and ease of use.

**Key Features:**
- 🚀 **Blazing Fast Performance**: Leveraging the power of Rust under the hood, RapidQuery ensures your query building process is as fast as possible.
- 🛡️ **SQL Injection Protection**: Built-in security measures to prevent SQL injection attacks by default.
- 📝 **Intuitive Pythonic API**: Write clean, readable code with an API that feels natural to Python developers.
- 🐍 **Seamless Python Integration**: Works perfectly with popular Python web frameworks and database drivers.

**Built on Solid Foundations** \
RapidQuery is built with **Rust** and powered by the robust **SeaQuery** crate, bringing enterprise-grade reliability and performance to your Python applications.

**Why RapidQuery Was Created** \
In a landscape filled with SQL libraries, we noticed a critical gap: **performance was often an afterthought**. That's why we built RapidQuery with speed as our primary and enduring focus.

**Our Core Mission:**
- **Performance First**: While other libraries compromise on speed, we engineered RapidQuery from the ground up for maximum performance.
- **Foundation for Future ORM**: RapidQuery serves as the foundational layer for building a next-generation, high-performance ORM for Python.
- **Meeting Python's Needs**: Python dominates backend development, particularly in web applications. Every backend deserves a fast, powerful database interaction layer — that's exactly what we're building.
- **Security by Design**: Unlike many alternatives, we bake security directly into our architecture with automatic SQL injection prevention.

Build your SQL queries faster, safer, and more efficiently than ever before. RapidQuery - where Python meets Rust's performance for database excellence.

## Installation
To install RapidQuery, run the following command:
```bash
pip3 install 'rapidquery<1'
```

> [!NOTE]\
> RapidQuery requires Python 3.10+. Supports CPython and PyPy.

## Backends
RapidQuery supports `PostgreSQL`, `MySQL`, and `SQLite` databases. In RapidQuery, these are referred to as `backend`s.
When building SQL statements, you should specify your target backend.

## Quick Example
```python
import rapidquery as rq
import datetime

stmt = rq.Insert().into("repositories").values(name="RapidQuery", created_at=datetime.datetime.now())
stmt.to_sql("postgres")
# INSERT INTO "repositories" ("name", "created_at") VALUES ('RapidQuery', '2025-11-14 16:18:59.188940')
```

## Usage

1. Core Concepts
    1. [**AdaptedValue**](#adaptedvalue)
    2. [**Expr**](#expr)
    3. [**Statement Builders**](#statement-builders)
2. Query Statements
    1. [**Query Select**](#query-select)
    2. [**Query Insert**](#query-insert)
    3. [**Query Update**](#query-update)
    4. [**Query Delete**](#query-delete)
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
    1. [**ORM-like**](#orm-like)
    2. [**Table Alias**](#table-alias)
    2. [**Complex Examples**](#complex-examples)
6. Performance
    1. [**Benchmarks**](#benchmarks)
    2. [**Performance Tips**](#performance-tips)

### Core Concepts
#### AdaptedValue
`AdaptedValue` bridges Python types, Rust types, and SQL types for seamless data conversion.

This class handles validation, adaptation, and conversion between different
type systems used in the application stack.

```python
import rapidquery as rq

# Let the system detect types automatically
rq.AdaptedValue(1)                    # -> INTEGER SQL type
rq.AdaptedValue(1.4)                  # -> DOUBLE SQL type
rq.AdaptedValue("127.0.0.1")          # -> VARCHAR SQL type
rq.AdaptedValue({"key": "value"})     # -> JSON SQL type

# Explicitly specify the type
rq.AdaptedValue(1, rq.TinyUnsignedType())      # -> TINYINT UNSIGNED SQL type
rq.AdaptedValue(1.4, rq.FloatType())           # -> FLOAT SQL type
rq.AdaptedValue("127.0.0.1", rq.InetType())    # -> INET SQL type (network address)
rq.AdaptedValue([4.3, 5.6], rq.VectorType())   # -> VECTOR SQL type (for AI embeddings)

# Also you can use `AdaptedValue.to_sql()` method to convert value into SQL
val = rq.AdaptedValue([2, 3, 4], rq.ArrayType(rq.IntegerType()))
val.to_sql("postgresql") # -> ARRAY [2,3,4]
```

As we said, `AdaptedValue` also validates your value:
```python
rq.AdaptedValue(4.5, rq.CharType()) # -> TypeError: expected str, got float
```

> [!TIP]\
> **Important**: `AdaptedValue` is lazy. This means it keeps your value and never converts it to Rust and then SQL until needed.

#### Expr
Represents a SQL expression that can be built into SQL code.

This class provides a fluent interface for constructing complex SQL expressions
in a database-agnostic way. It supports arithmetic operations, comparisons,
logical operations, and database-specific functions.

The class automatically handles SQL injection protection and proper quoting
when building the final SQL statement.

Everything can be converted into `Expr`, such as built-in types, `datetime`, `uuid`, `AdaptedValue`, `Select`, etc.

**Basic**
```python
import rapidquery as rp

rp.Expr(25)                         # -> 25  (literal value)
rp.Expr("Hello")                    # -> 'Hello'  (literal value)
rp.Expr(rq.AdaptedValue('World'))   # -> 'World'  (literal value)

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

- Use database-specific features when necessary: But understand portability trade-offs
```python
# PostgreSQL-specific but powerful
Expr.col("tags").pg_contains(["python"])

# More portable but may be less efficient
Expr.col("tags").like("%python%")
```

#### Statement Builders
Statements are divided into 2 categories: `QueryStatement`, and `SchemaStatement`.

Some statements like `Select`, `Update`, `Delete`, `Insert`, ... are `QueryStatement`.
Other statements like `Table`, `AlterTable`, `Index`, ... are `SchemaStatement`.

`QueryStatement` class interface is:
```python
class QueryStatement:
    def build(self, backend: _Backends) -> typing.Tuple[str, typing.Tuple[AdaptedValue, ...]]:
        """
        Build the SQL statement with parameter values.
        """
        ...

    def to_sql(self, backend: _Backends) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
        """
        ...
```

`SchemaStatement` class interface is:
```python
class SchemaStatement:
    def to_sql(self, backend: _Backends) -> str:
        """
        Build a SQL string representation.
        """
        ...
```

### Query Statements
#### Query Select
`Select` provides a chainable API for constructing SELECT queries with support for:
- Column selection with expressions and aliases
- Table and subquery sources
- Filtering with WHERE and HAVING
- Joins (inner, left, right, full, cross, lateral)
- Grouping and aggregation
- Ordering and pagination
- Set operations (UNION, EXCEPT, INTERSECT)
- Row locking for transactions
- DISTINCT queries

**Simple**
```python
query = (
    rq.Select(rq.Expr.asterisk()) # Or rq.Select(rq.ASTERISK)
        .from_table("users")
        .where(rq.Expr.col("name").like(r"%linus%"))
)

cte = (
    rq.With(
        rq.CommonTableExpression("a", query1, columns=["ali", "mmd"]),
        rq.CommonTableExpression("b", query2, columns=["ali", "mmd"]),
        rq.CommonTableExpression("c", query3, columns=["ali", "mmd"]),
    )
    .resursive(True)
    .cycle(...)
    .search(...)
    .query()
)

sql, params = query.build("postgresql")
# -> SELECT * FROM "users" WHERE "name" LIKE $1

query = (
    rq.Select(rq.Expr.col("product"), rq.Expr.col("price"), rq.Expr.col("category"))
        .from_table("products")
        .where(rq.Expr.col("price") > 50)
        .order_by(rq.Expr.col("price"), "desc")
)
sql, params = query.build("postgresql")
# -> SELECT "product", "price", "category" FROM "products" WHERE "price" > $1 ORDER BY "price" DESC

query = (
    rq.Select(
        rq.SelectCol(rq.FunctionCall.count(rq.ASTERISK), alias="total_customers"),
        rq.SelectCol(rq.FunctionCall.avg(rq.Expr.col("age")), alias="average_age"),
    )
        .from_table("customers")
)
sql, params = query.build("postgresql")
# -> SELECT COUNT(*) AS "total_customers", AVG("age") AS "average_age" FROM "customers"
```

#### Query Insert
`Insert` provides a chainable API for constructing INSERT queries with support for:
- Single or multiple row insertion
- Conflict resolution (UPSERT)
- RETURNING clauses
- REPLACE functionality
- Default values

```python
query = (
    rq.Insert()
        .replace()
        .into("glyph")
        .values(aspect=5.15, image="12A")
)
sql, params = query.build("postgresql")
# REPLACE INTO "glyph" ("aspect", "image") VALUES ($1, $2)

query = (
    rq.Insert()
        .into("glyph")
        .columns("aspect", "image")
        .values(5.15, "12A")
        .values(16, "14A")
        .returning("id")
)
sql, params = query.build("postgresql")
# INSERT INTO "glyph" ("aspect", "image") VALUES ($1, $2), ($3, $4) RETURNING "id"

query = (
    rq.Insert()
        .into("users")
        .values(username="awolverp", role="author")
        .on_conflict(
            rq.OnConflict("id")
                .do_update("username")
        )
)
sql, params = query.build("postgresql")
# INSERT INTO "users" ("username", "role") VALUES ($1, $2)
# ON CONFLICT ("id") DO UPDATE SET "username" = "excluded"."username"

query = (
    rq.Insert()
        .into("users")
        .values(username="awolverp", role="author")
        .on_conflict(
            rq.OnConflict("id")
                .do_update(role="member")
        )
)
sql, params = query.build("postgresql")
# INSERT INTO "users" ("username", "role") VALUES ($1, $2)
# ON CONFLICT ("id") DO UPDATE SET "role" = $3
```

#### Query Update
`Update` provides a chainable API for constructing UPDATE queries with support for:
- Setting column values
- WHERE conditions for filtering
- LIMIT for restricting update count
- ORDER BY for determining update order
- RETURNING clauses for getting updated data

```python
query = (
    rq.Update()
        .table("glyph")
        .values(aspect=5.15, image="12A")
        .returning_all()
        .order_by(rq.Expr.col("id"), "desc")
)
sql, params = query.build("postgresql")
# UPDATE "glyph" SET "aspect" = $1, "image" = $2 ORDER BY "id" DESC RETURNING *

query = (
    rq.Update()
        .table("wallets")
        .values(amount=rq.Expr.col("amount") + 10)
        .where(rq.Expr.col("id").between(10, 30))
)
sql, params = query.build("postgresql")
# UPDATE "wallets" SET "amount" = "amount" + $1 WHERE "id" BETWEEN $2 AND $3
```

#### Query Delete
`Delete` provides a chainable API for constructing DELETE queries with support for:
- WHERE conditions for filtering
- LIMIT for restricting deletion count
- ORDER BY for determining deletion order
- RETURNING clauses for getting deleted data

```python
query = (
    rq.Delete()
        .from_table("users")
        .where(
            rq.all(
                rq.Expr.col("id") > 10,
                rq.Expr.col("id") < 30,
            )
        )
        .limit(10)
)
sql, params = query.build("postgresql")
# DELETE FROM "users" WHERE "id" > $1 AND "id" < $2 LIMIT $3
```

### More About Queries
#### Custom Functions
For working with functions in RapidQuery, you have to use `FunctionCall` class.
A lot of functions such as `SUM`, `AVG`, `MD5`, ... is ready to use. For example:

```python
expr = rq.FunctionCall.sum(rq.Expr.col("amount"))
expr.to_sql("postgresql")   # -> SUM("amount")
```

But for functions not provided by the library, you can define custom functions.
Custom functions can be defined using the `FunctionCall` constructor:

```python
unknown = rq.FunctionCall("UNKNOWN").arg(rq.ASTERISK)
expr.to_sql("postgresql")   # -> UNKNOWN(*)
```

### Schema Statements
#### Table Create
`Table` represents a complete database table definition.

This class encapsulates all aspects of a table structure including:
- Column definitions with their types and constraints
- Indexes for query optimization
- Foreign key relationships for referential integrity
- Check constraints for data validation
- Table-level options like engine, collation, and character set

Used to generate CREATE TABLE SQL statements with full schema specifications.

```python
table = rq.Table(
    "users",
    [
        rq.Column("id", rq.BigIntegerType(), primary_key=True, auto_increment=True),
        rq.Column("name", rq.StringType(64), nullable=False),
        rq.Column("username", rq.StringType(64), nullable=True, default=None),
        rq.Column("subscription_id", rq.BigIntegerType(), nullable=False),
        rq.Column("created_at", rq.DateTimeType(), default=rq.FunctionCall.now()),
    ],
    indexes=[
        rq.Index(["created_at"], if_not_exists=True),
    ],
    foreign_keys=[
        rq.ForeignKey(
            from_columns=["subscription_id"],
            to_columns=["id"],
            to_table="subscriptions",
        ),
    ],
    if_not_exists=True,
)
table.to_sql("postgresql")
# CREATE TABLE IF NOT EXISTS "users" (
#   "id" bigserial PRIMARY KEY,
#   "name" varchar(64) NOT NULL,
#   "username" varchar(64) NULL DEFAULT NULL,
#   "subscription_id" bigint NOT NULL,
#   "created_at" datetime DEFAULT NOW(),
#   CONSTRAINT "fk__subscription_id_subscriptions_id" FOREIGN KEY ("subscription_id") REFERENCES "subscriptions" ("id")
# );
# CREATE INDEX IF NOT EXISTS "ix_users_created_at" ON "users" ("created_at");
```

> [!TIP]\
> We will use `Table` in [**ORM-like**](#orm-like) part of this page to create query statements.

#### Table Alter
`AlterTable` represents an ALTER TABLE SQL statement.

Provides a flexible way to modify existing table structures by applying
one or more alteration operations such as adding/dropping columns,
modifying column definitions, or managing constraints.

Multiple operations can be batched together in a single ALTER TABLE
statement for efficiency.

```python
stmt = rq.AlterTable(
    "users",
    [
        rq.AlterTableAddColumnOption(
            rq.Column("updated_at", rq.TimestampWithTimeZoneType(), default=rq.FunctionCall.now())
        ),
        rq.AlterTableAddForeignKeyOption(
            rq.ForeignKey(
                from_columns=["wallet_id"],
                to_columns=["id"],
                to_table="wallets",
                on_delete="CASCADE",
            )
        ),
        rq.AlterTableDropColumnOption("deprecated"),
        rq.AlterTableDropForeignKeyOption("fk__contraint_name"),
        rq.AlterTableModifyColumnOption(rq.Column("created_at", rq.TimestampType())),
        rq.AlterTableRenameColumnOption("oldname", "newname"),
    ],
)
stmt.to_sql("postgresql")
# ALTER TABLE "users" ADD COLUMN "updated_at" timestamp with time zone DEFAULT NOW(),
# ADD CONSTRAINT "fk__wallet_id_wallets_id" FOREIGN KEY ("wallet_id") REFERENCES "wallets" ("id") ON DELETE CASCADE,
# DROP COLUMN "deprecated", DROP CONSTRAINT "fk__contraint_name",
# ALTER COLUMN "created_at" TYPE timestamp,
# RENAME COLUMN "oldname" TO "newname"
```

#### Table Drop
`DropTable` represents a DROP TABLE SQL statement.

Builds table deletion statements with support for:
- Conditional deletion (IF EXISTS) to avoid errors
- CASCADE to drop dependent objects
- RESTRICT to prevent deletion if dependencies exist

```python
stmt = rq.DropTable("users", if_exists=True)
stmt.to_sql("postgresql")
# DROP TABLE IF EXISTS "users"
```

#### Table Rename
`RenameTable` represents a RENAME TABLE SQL statement.

Changes the name of an existing table to a new name. Both names can be
schema-qualified if needed.

```python
stmt = rq.RenameTable("public.old_users", "archive.users")
stmt.to_sql("postgresql")
# ALTER TABLE "public"."old_users" RENAME TO "archive"."users"
```

#### Table Truncate
`TruncateTable` Represents a TRUNCATE TABLE SQL statement.

Quickly removes all rows from a table, typically faster than DELETE
and with different transaction and trigger behavior depending on the
database system.

```python
stmt = rq.TruncateTable("temp_data")
stmt.to_sql("postgresql")
# TRUNCATE TABLE "temp_data"
```

#### Index Create
`Index` represents a database index specification.

This class defines the structure and properties of a database index,
including column definitions, uniqueness constraints, index type,
and partial indexing conditions.

```python
stmt = rq.Index(
    ["user_id", "reseller_id"],
    "ix_users_user_reseller_id",
    table="users",
    if_not_exists=True,
)
stmt.to_sql("postgresql")
# CREATE INDEX IF NOT EXISTS "ix_users_user_reseller_id" ON "users" ("user_id", "reseller_id")

stmt = rq.Index(
    [rq.IndexColumn("name", prefix=8, order="desc")],
    "ix_users_user_reseller_id",
    table="users",
    if_not_exists=True,
)
stmt.to_sql("postgresql")
# CREATE INDEX IF NOT EXISTS "ix_users_user_reseller_id" ON "users" ("name" (8) DESC)
```

#### Index Drop
`DropIndex` represents a DROP INDEX SQL statement.

Builds index deletion statements with support for:
- Conditional deletion (IF EXISTS)
- Table-specific index dropping (for databases that require it)
- Proper error handling for non-existent indexes

```python
stmt = rq.DropIndex("ix_users_user_reseller_id")
stmt.to_sql("postgresql")
# DROP INDEX "ix_users_user_reseller_id"
```

### Advanced Usage
#### ORM-like
`Table` class is not just for generating CREATE TABLE statements. It's designed to make developing
easier for you.

First you have to know some basics:
```python
users = rq.Table(
    "users",
    [
        rq.Column("id", rq.IntegerType()),
        rq.Column("name", rq.CharType(255)),
    ]
)

# You can access columns easily:
users.c.id         # -> <Column "id" type=<IntegerType >>
users.c.name       # -> <Column "name" type=<CharType length=255>>
users.c.not_exists # -> KeyError: 'not_exists'
```

Now you can use this structure to create `Select`, `Update`, `Delete`, and `Insert` queries:
```python
query = (
    rq.Select(users.c.name)
        .from_table(users)
        .where(users.c.id.to_expr() == 2)
)
sql, params = query.build("postgresql")
# SELECT "users"."name" FROM "users" WHERE "users"."id" = $1
```

#### Table Alias
Using `Table` for creating queries can help you to create queries easier, but again it's hard to
have aliases (e.g. `FROM users AS u`) in queries. So we have **`AliasedTable`** class to make it
easy.

Imagine this table:
```python
employees = rq.Table(
    "employees",
    [
        rq.Column("id", rq.IntegerType()),
        rq.Column("first_name", rq.CharType(255)),
        rq.Column("jon_title", rq.CharType(255)),
    ]
)
```

**Without AliasedTable**
```python
query = (
    rq.Select(
        employees.c.id.to_column_ref().copy_with(table="emp"),
        rq.SelectCol(
            employees.c.name.to_column_ref().copy_with(table="emp"),
            "employee_name",
        ),
        employees.c.job_title.to_column_ref().copy_with(table="emp"),
        rq.SelectCol(employees.c.id.to_column_ref().copy_with(table="mgr"), "manager_id"),
        rq.SelectCol(
            employees.c.name.to_column_ref().copy_with(table="mgr"),
            "employee_name",
        ),
        rq.SelectCol(
            employees.c.job_title.to_column_ref().copy_with(table="mgr"), "manager_title"
        ),
    )
    .from_table(employees.name.copy_with(alias="emp"))
    .join(
        employees.name.copy_with(alias="mgr"),
        (
            rq.Expr(employees.c.manager_id.to_column_ref().copy_with(table="emp"))
            == employees.c.id.to_column_ref().copy_with(table="mgr")
        ),
        type="inner"
    )
)
sql, params = query.build("postgresql")
# SELECT
#   "emp"."id", "emp"."name" AS "employee_name",
#   "emp"."job_title", "mgr"."id" AS "manager_id",
#   "mgr"."name" AS "employee_name", "mgr"."job_title" AS "manager_title"
# FROM "employees" AS "emp"
# INNER JOIN "employees" AS "mgr" ON "emp"."manager_id" = "mgr"."id"
```

It's so hard and unreadable.

**With AliasedTable**
```python
emp = rq.AliasedTable(employees, "emp")
mgr = rq.AliasedTable(employees, "mgr")

query = (
    rq.Select(
        emp.c.id,
        rq.SelectCol(emp.c.name, "employee_name"),
        emp.c.job_title,
        rq.SelectCol(emp.c.id, "manager_id"),
        rq.SelectCol(emp.c.name, "employee_name"),
        rq.SelectCol(emp.c.job_title, "manager_title"),
    )
    .from_table(emp)
    .join(
        mgr,
        rq.Expr(emp.c.manager_id) == mgr.c.id,
        type="inner",
    )
)
sql, params = query.build("postgresql")
# SELECT
#   "emp"."id", "emp"."name" AS "employee_name",
#   "emp"."job_title", "mgr"."id" AS "manager_id",
#   "mgr"."name" AS "employee_name", "mgr"."job_title" AS "manager_title"
# FROM "employees" AS "emp"
# INNER JOIN "employees" AS "mgr" ON "emp"."manager_id" = "mgr"."id"
```

As you saw, it's much simpler.


#### Complex Examples
There are some complex examples for `SELECT` query.

RapidQuery:
```python
rq.Select(
    rq.Expr.col("account_number"),
    rq.Expr.col("transaction_date"),
    rq.Expr.col("transaction_type"),
    rq.Expr.col("amount"),
    rq.SelectCol(
        rq.Case()
        .when(rq.Expr.col("transaction_type") == "DEBIT", -rq.Expr.col("amount"))
        .else_(rq.Expr.col("amount")),
        alias="signed_amount",
    ),
    rq.SelectCol(
        rq.FunctionCall.sum(
            rq.Case()
            .when(rq.Expr.col("transaction_type") == "DEBIT", -rq.Expr.col("amount"))
            .else_(rq.Expr.col("amount"))
        ),
        alias="running_balance",
        window="account_window",
    ),
    rq.SelectCol(
        rq.FunctionCall.avg(rq.Expr.col("amount")),
        alias="avg_transaction_by_type",
        window=rq.Window(rq.Expr.col("account_number"), rq.Expr.col("transaction_type")),
    ),
    rq.SelectCol(
        rq.FunctionCall.percent_rank(),
        alias="amount_percentile",
        window=rq.Window(rq.Expr.col("account_number")).order_by(rq.Expr.col("amount"), "desc"),
    ),
)
.from_table("bank_transactions")
.where(
    rq.Expr.col("transaction_date").between(
        rq.Expr.custom("'2024-01-01'"), rq.Expr.custom("'2024-12-31'")
    )
)
.window(
    "account_window",
    rq.Window(rq.Expr.col("account_number"))
    .order_by(rq.Expr.col("transaction_date"), "desc")
    .order_by(rq.Expr.col("transaction_id"), "desc")
    .frame("rows", rq.WindowFrame.unbounded_preceding(), rq.WindowFrame.current_row()),
)
```

SQL:
```sql
SELECT 
    "account_number",
    "transaction_date",
    "transaction_type",
    "amount",
    (CASE WHEN ("transaction_type" = 'DEBIT') THEN "amount" * -1 ELSE "amount" END) AS "signed_amount",
    SUM(
        (CASE WHEN ("transaction_type" = 'DEBIT') THEN "amount" * -1 ELSE "amount" END)
    ) OVER "account_window" AS "running_balance",
    AVG("amount") OVER (PARTITION BY "account_number", "transaction_type") AS "avg_transaction_by_type",
    PERCENT_RANK() OVER (
        PARTITION BY "account_number" ORDER BY "amount" DESC
    ) AS "amount_percentile"
FROM "bank_transactions"
WHERE
    "transaction_date" BETWEEN ('2024-01-01') AND ('2024-12-31')
WINDOW
    "account_window" AS (
        PARTITION BY "account_number"
        ORDER BY "transaction_date" DESC, "transaction_id" DESC
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    )
```

### Performance
#### Benchmarks

> [!NOTE]
> Benchmarks run on *Linux-6.15.11-2-MANJARO-x86_64-with-glibc2.42* with CPython 3.13. Your results may vary.

**Iterations per test:** 100,000  
**Python version:** 3.13.7

---

**📊 SELECT Query Benchmark**

| Library | Time (ms) | vs Fastest | Status |
|---------|-----------|------------|--------|
| RapidQuery | 247.79 | 1.00x (FASTEST) | 🏆 |
| PyPika | 4030.62 | 16.27x slower | |
| SQLAlchemy | 9238.36 | 37.28x slower | |

---

**📊 INSERT Query Benchmark**

| Library | Time (ms) | vs Fastest | Status |
|---------|-----------|------------|--------|
| RapidQuery | 275.13 | 1.00x (FASTEST) | 🏆 |
| PyPika | 4268.81 | 15.52x slower | |
| SQLAlchemy | 6849.45 | 24.90x slower | |

---

**📊 UPDATE Query Benchmark**

| Library | Time (ms) | vs Fastest | Status |
|---------|-----------|------------|--------|
| RapidQuery | 270.03 | 1.00x (FASTEST) | 🏆 |
| PyPika | 4450.08 | 16.48x slower | |
| SQLAlchemy | 11637.68 | 43.10x slower | |

---

**📊 DELETE Query Benchmark**

| Library | Time (ms) | vs Fastest | Status |
|---------|-----------|------------|--------|
| RapidQuery | 242.09 | 1.00x (FASTEST) | 🏆 |
| PyPika | 4154.24 | 17.16x slower | |
| SQLAlchemy | 7873.16 | 32.52x slower | |

#### Performance Tips
- Using [`ORM-like`](#orm-like) is always slower than using `Expr.col` and literal `str`
- "Less calls, more speed"; RapidQuery powered by Rust & SeaQuery, which made us very fast, and only thing that can effect speed, is object calls in Python.

## Known Issues
### Unmanaged Rust Panic Output in Error Handling
The library may encounter errors during SQL query construction, which are correctly raised as *RuntimeError* exceptions. For instance, this occurs when using a function that isn't supported by your target database. **While this error-raising behavior is intentional and logical, the issue is that unmanaged Rust panic information is also printed to stderr**. Currently, there is no way to suppress or manage this panic output. We are working to resolve this problem as much as possible in future updates.

```python
expr = rq.Expr.col("id").pg_contained("text")
expr.to_sql("sqlite")

thread '<unnamed>' (19535) panicked at sea-query-0.32.7/src/backend/query_builder.rs:665:22:
not implemented
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Traceback (most recent call last):
  File "<python-input-1>", line 1, in <module>
    expr.to_sql("sqlite")
    ~~~~~^^^^^^^^^^
RuntimeError: build failed
```

## License
This repository is licensed under the [GNU GPLv3 License](LICENSE)
