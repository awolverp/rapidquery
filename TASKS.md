All six research tracks finished. Below is the WriteSql audit: what we emit today, where official docs disagree, and the work to do next. No code in this turn.

**Policy (already in `src/lib.rs`):** if a dialect does not document a construct, `to_sql` / `prepare` should raise. Do not invent `LOWER()`, `REGEXP_LIKE()`, or `CONCAT()` unless a dialect flag names that rewrite.

**WriteSql inventory**

| Writer                                | File                                                               |
| ------------------------------------- | ------------------------------------------------------------------ |
| `Function`                            | `src/expr/enum_/func.rs`                                           |
| `Ident` / `IdentPart`                 | `src/expr/enum_/ident.rs`                                          |
| `Value` → `SerializerEnum::write_sql` | `src/expr/enum_/value.rs`, `src/data_type/serializer/serialize.rs` |
| `DataTypeName` / `PyDataType`         | `src/data_type/name.rs`, `src/data_type/mod.rs`                    |
| `ExprEnum` arms                       | `src/expr/enum_/mod.rs`                                            |
| Placeholders                          | `ParameterizedSql::push_param` in `src/sql.rs`                     |
| Quote / escape / bytes / bits / bool  | `src/dialect/inner.rs`                                             |

---

## 0. Cross-cutting: identifiers and placeholders

**Emit:** always-quote `IdentPart::Quote`; Unquote = `[A-Za-z_][A-Za-z0-9_]*`; Asterisk = `*`; parts joined with `.`; params = `{sigil}param_n` + dict.

### P0

1. **Teradata identifiers** use `'name'` (`IdentifierFormat::SingleQuote`). Official delimited names use `"…"` (U+0022). `'users'` is a character literal.
2. **BigQuery** identifier escape does not emit `` \` ``. A backtick inside a name splits the identifier.
3. **PostgreSQL `:param_0` is not libpq SQL.** Native parameters are `$1`. MySQL/Hive PREPARE/JDBC use `?`.

### Tasks

**A.** Doccomments on `Ident` / `IdentPart` `WriteSql` and `Dialect::quote_identifier`: always-quote; Unquote is a conservative subset; Asterisk vs `"*"`; named `{sigil}param_n` is driver-named style, not wire protocol.

**B.**

1. Teradata: `identifier_format = DoubleQuote`, escape delimiters (`"` → `""`).
2. BigQuery: escape `` ` `` as `` \` `` (string-escape table).
3. Placeholder policy: either native (`$1` / `?`) or keep named and document that PG/MySQL/Hive SQL is not wire-native. Do not claim `:param_0` is libpq.

**C.** Optional `QUOTE_IDENT`; SQL Server empty qualifiers (`db..tbl`); per-dialect Unquote charset.

**D.** Teradata `"a""b"`; BigQuery name containing backtick; PG `$1` vs `:param_0` documentation tests.

Official: [BQ lexical](https://docs.cloud.google.com/bigquery/docs/reference/standard-sql/lexical) · [PG lexical](https://www.postgresql.org/docs/current/sql-syntax-lexical.html) · [PG PREPARE](https://www.postgresql.org/docs/current/sql-prepare.html) · [SQLite keywords](https://www.sqlite.org/lang_keywords.html) · [MySQL identifiers](https://dev.mysql.com/doc/refman/8.4/en/identifiers.html)

---

## 1. Function (`Function::write_sql`) — your template

**Emit:** quoted name; `( [DISTINCT\|ALL ] args [ IGNORE NULLS ] )` or postfix NT; then always ` FILTER (WHERE …)`. NT inside parens only for BigQuery and DuckDB.

### Official vs emit

| Dialect                 | DISTINCT / ALL              | FILTER                                            | IGNORE/RESPECT NULLS                 |
| ----------------------- | --------------------------- | ------------------------------------------------- | ------------------------------------ |
| PostgreSql              | D and ALL                   | `FILTER (WHERE)`                                  | none                                 |
| Sqlite                  | DISTINCT only               | `FILTER (WHERE)` on aggregates                    | none                                 |
| BigQuery                | DISTINCT only               | **in-call `WHERE`** (Preview)                     | **inside** parens                    |
| DuckDb                  | DISTINCT                    | `FILTER (pred)`; railroad also `FILTER (WHERE …)` | inside on window fns                 |
| Databricks / Spark      | D; Databricks COUNT has ALL | `FILTER (WHERE)`                                  | postfix, then OVER                   |
| Oracle 23+              | D and ALL                   | `FILTER (WHERE)`                                  | postfix (ANSI) and inside            |
| Snowflake / Redshift    | D; Redshift has ALL         | none                                              | postfix + OVER                       |
| MsSql                   | D and ALL                   | none                                              | postfix + OVER (2022+)               |
| MySql / Hive / Teradata | D; Teradata has ALL         | none                                              | Teradata **inside**; MySQL/Hive none |

Bare `CURRENT_TIMESTAMP` is a **keyword/literal token** on SQLite (and similar on PG/Oracle). Quoting it (`"CURRENT_TIMESTAMP"`) kills the special meaning.

### P0

1. `FILTER (WHERE)` on MySQL, MsSql, Hive, Snowflake, Redshift, Teradata.
2. BigQuery: official filter is `fn(x WHERE cond)`, not `fn(x) FILTER (WHERE cond)`.
3. `ALL` on BigQuery and SQLite (existing TODO).
4. NT on PostgreSQL, SQLite, MySQL, Hive.
5. Teradata NT is inside parens; builtin is `OutsideParens`.
6. Always-quoted niladic `CURRENT_TIMESTAMP` on SQLite.

### Tasks

**A.** Expand the Function `WriteSql` comment (keep BQ + SQLite links) with the URLs below.

**B.** (`func.rs`, dialect flags)

1. Skip `ALL` where not in syntax (BQ, SQLite; also Snowflake COUNT / Hive).
2. BigQuery FILTER → in-call `WHERE` after args/NT.
3. Omit FILTER unless dialect supports it (PG, SQLite, DuckDB, Spark, Databricks, Oracle 23+).
4. Omit NT unless dialect supports it (BQ, DuckDB, Snowflake, Spark, Databricks, Oracle, Redshift, Teradata, MsSql 2022+).
5. Teradata `null_treatment_syntax = InsideParens`.
6. Bare SQL-standard datetime keywords: do not quote.
7. DuckDB: railroad is `FILTER ( WHERE filter_expr )`; examples omit `WHERE`. Keep `WHERE` unless a rendered parse test fails.

Suggested flags: `duplicate_treatment_all: bool`, `filter_syntax: None | FilterWhere | FilterBare | InsideWhere`.

**C.** OVER, WITHIN GROUP, in-arg ORDER BY, named `=>` args, BQ HAVING MAX/MIN / LIMIT. Function-specific NT allowlists. `COUNT(*)` vs empty `COUNT()`.

**D.** ALL omitted on BQ/SQLite; FILTER spelling per dialect; NT in vs post vs omitted; unquoted `CURRENT_TIMESTAMP` on SQLite.

Official: [BQ aggregate calls](https://docs.cloud.google.com/bigquery/docs/reference/standard-sql/aggregate-function-calls) · [PG expressions](https://www.postgresql.org/docs/current/sql-expressions.html) · [SQLite agg](https://www.sqlite.org/lang_aggfunc.html) · [DuckDB FILTER](https://duckdb.org/docs/current/sql/query_syntax/filter.html) · [Oracle FILTER](https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/Aggregate-Functions.html) · [Spark windows](https://spark.apache.org/docs/latest/sql-ref-syntax-qry-select-window.html) · [Snowflake FIRST_VALUE](https://docs.snowflake.com/en/sql-reference/functions/first_value)

---

## 2. LIKE / ILIKE / LIKE ANY / SIMILAR TO / RLIKE

**Emit:** `LIKE`/`ILIKE` + inlined ESCAPE; ANY via ArrayConstructor / PatternList / OrExpansion; `SIMILAR TO` and `RLIKE` ungated.

### P0

1. **ILIKE** on BigQuery, Hive, MsSql, MySql, Oracle, Sqlite, Teradata.
2. **SIMILAR TO** except PostgreSql, Redshift, DuckDb.
3. **RLIKE** infix + ESCAPE: PG/Redshift want `~`; Oracle `REGEXP_LIKE()`; BigQuery `REGEXP_CONTAINS()`; MsSql none; SQLite user `REGEXP`. ESCAPE is not a regex clause even where RLIKE exists (MySQL/Hive/Spark/Databricks/Snowflake).
4. PostgreSql `LIKE ANY (ARRAY[…]) ESCAPE` — ESCAPE is not composable with ANY.
5. Spark/Databricks `NOT LIKE ANY` = “fails at least one”, not “matches none”.
6. BigQuery LIKE has **no ESCAPE clause**.

### Tasks

**A.** Doccomments on `write_like` / SimilarTo / RLike: ANY shapes; why PG wraps `NOT (LIKE ANY)`; ungated SIMILAR TO / RLIKE.

**B.**

| Node              | Gate                                                                                                                                                                                                  |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ILIKE             | Only PG, Redshift, DuckDB, Snowflake, Spark, Databricks. Else LIKE (no silent `LOWER()`).                                                                                                             |
| LIKE ANY + ESCAPE | Omit ESCAPE on PG ANY, Spark/Databricks quantified, BigQuery always. Keep on Snowflake ANY and scalar/OrExpansion.                                                                                    |
| LIKE ANY negation | PatternList: `NOT (x LIKE ANY (…))`, not `x NOT LIKE ANY`. Keep PG wrap and OrExpansion AND of NOT LIKE.                                                                                              |
| SIMILAR TO        | PG, Redshift, DuckDB only.                                                                                                                                                                            |
| RLIKE             | MySQL/Hive/Spark/Databricks/Snowflake: infix, **never ESCAPE**. Oracle: `REGEXP_LIKE`. PG/Redshift: `~`. DuckDB: `regexp_matches` or `~` (full match). BigQuery: `REGEXP_CONTAINS`. Else unsupported. |

**C.** ILIKE via COLLATE/LOWER; LIKE ALL; GLOB; Snowflake RLIKE anchoring.

**D.** Per-dialect LIKE / ILIKE / ANY negation / SIMILAR TO / RLIKE snapshots; PG ANY without ESCAPE; no `RLIKE … ESCAPE`.

Official: [PG matching](https://www.postgresql.org/docs/current/functions-matching.html) · [BQ operators](https://cloud.google.com/bigquery/docs/reference/standard-sql/operators) · [Snowflake LIKE ANY](https://docs.snowflake.com/en/sql-reference/functions/like_any) · [Spark LIKE](https://spark.apache.org/docs/latest/sql-ref-syntax-qry-select-like.html) · [MySQL REGEXP](https://dev.mysql.com/doc/refman/8.4/en/regexp.html)

---

## 3. Literals (`Value` → serializers + dialect formatters)

**Emit:** bool keyword or `1`/`0`; strings via wrap+escape; bytes via `BytesFormat`; bits or raise; date/time/json/uuid/inet/mac as **quoted strings**; arrays `ARRAY[…]` only if `supports_array`.

### P0

1. **PostgreSQL NUL:** code zero cannot appear in a string constant. `E'\x00'` is still NUL after unescape. **Reject NUL.**
2. **Snowflake `X'HEX'`** is not a documented BINARY constant (`TO_BINARY`).
3. **DuckDB `X'HEX'`** is not the documented blob form (`'\xAA'::BLOB`).
4. **Oracle untyped `'YYYY-MM-DD'`** follows `NLS_DATE_FORMAT`. Official DATE literal is `DATE 'YYYY-MM-DD'`.
5. **QuoteStripNul** deletes U+0000 (MsSql, Oracle, Redshift, Sqlite, Teradata) — payload change. Prefer reject.

Boolean `1`/`0` on MsSql/MySql/Oracle/Sqlite matches those engines’ usual constants. Keyword TRUE/FALSE is correct on the others.

### Tasks

**A.** Doccomments on `SerializerEnum::write_sql`, `write_string_literal`, `format_bytes`, `format_bits`: wrap/escape; PG `E` + NUL illegal; MsSql `N'` = nvarchar; bytes wrappers; temporals/json/uuid are string literals.

**B.**

1. Reject U+0000 on PostgreSQL (and any dialect that forbids NUL in strings).
2. Snowflake bytes → `TO_BINARY('…', 'HEX')`.
3. DuckDB bytes → `'\xHH…'::BLOB`.
4. Oracle dates → `DATE 'YYYY-MM-DD'`; timestamps → `TIMESTAMP '…'`.
5. Stop stripping NUL: reject.
6. High value: typed `DATE '…'` / `TIMESTAMP '…'` on BQ, Databricks, DuckDB, Hive, PG, Snowflake, Spark, Teradata ANSI.

**C.** JSON `JSON '…'` / `PARSE_JSON` / `'…'::jsonb`; UUID/`inet` casts; DuckDB LIST `[…]`; Spark/Hive `array()`; Postgres `'…'` + `''` with `E` only when backslash is needed.

**D.** Golden per dialect: bool, `O'Brien`, NUL (error), bytes, empty bytes, date, JSON, UUID, ARRAY or raise.

Official: [PG lexical](https://www.postgresql.org/docs/current/sql-syntax-lexical.html) · [BQ lexical](https://docs.cloud.google.com/bigquery/docs/reference/standard-sql/lexical) · [Snowflake binary I/O](https://docs.snowflake.com/en/sql-reference/binary-input-output) · [Oracle literals](https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/Literals.html) · [DuckDB blob](https://duckdb.org/docs/current/sql/data_types/blob.html)

---

## 4. Operators, tuples, predicates

**Emit:** unary spacing groups; binary `left OP right`; `!=` → `<>`; `XOR` keyword; `^` bitwise XOR; `||` concat; tuple `(a, b)`; `IN ()` for empty; `IS NORMALIZED`; no BETWEEN SYMMETRIC.

### P0

1. MySQL default `||` is **logical OR**, not concat.
2. SQL Server `||` concat is **2025+**; classic concat is `+`.
3. `XOR` keyword is MySQL logical XOR only.
4. `^` is **exponent** on PostgreSQL, Redshift, DuckDB. PG bitwise XOR is `#`.
5. `%` is not an operator on Oracle (`MOD()`) or Teradata (`MOD`).
6. `!` postfix factorial: **removed in PostgreSQL 14**; Hive/Spark/Databricks/MySQL `!` is logical NOT. DuckDB still has `4!`.
7. Geometric unaries `@-@` `@@` `#` `?-` `?|` are **PostgreSQL geometric**, not Redshift.
8. `IS [form] NORMALIZED` is PostgreSQL-only.
9. `IS DISTINCT FROM` undocumented on MySQL/Hive (use `<=>`), Oracle, Redshift, Teradata; MsSql from 2022.
10. Empty `IN ()` invalid except SQLite (and BQ empty-set _semantics_ via subquery).
11. Singleton `(x)` is grouping, not a 1-tuple, on PG/SQLite/Databricks/Spark/DuckDB.
12. Snowflake bitwise is functions (`BITAND`/`BITXOR`), not `& | ^ ~`.

`<>` is the portable inequality spelling on every opened comparison page.

### Tasks

**A.** Doccomments on unary/binary/tuple/Is/IsNormalized/IsDistinctFrom/InList/Between: ungated spellings; `!=`→`<>`; `!` factorial; empty `IN ()`. Do not call geometric unaries Redshift.

**B.**

| Node             | Action                                                                                  |
| ---------------- | --------------------------------------------------------------------------------------- |
| `\|\|`           | MySQL: `CONCAT()` (or require `PIPES_AS_CONCAT`). MsSql: `+` until 2025.                |
| `XOR`            | MySQL only.                                                                             |
| `BitXor`         | PG `#`; DuckDB `xor()`; Snowflake `BITXOR()`; BQ/Spark/Hive/MySQL/MsSql `^`.            |
| `Mod`            | Oracle `MOD()`; Teradata `MOD`.                                                         |
| PG unaries       | PG (+ Redshift numeric `@ \|/ \|\|/`). Factorial → `factorial()`. Geometric → PG only.  |
| `IsNormalized`   | PostgreSql only.                                                                        |
| `IsDistinctFrom` | `<=>` on MySQL/Hive; refuse Oracle/Redshift/Teradata; MsSql ≥2022.                      |
| Empty `IN`       | Rewrite to `FALSE`/`TRUE`; never emit `IN ()`.                                          |
| `Tuple`          | PG `ROW(...)` including 0/1 field; Spark/Databricks/Hive `struct()`; SQLite refuse 0/1. |
| Extra parens     | Always paren `\|\|` vs `+-`/`*/`; MsSql `IN`/`BETWEEN` vs `OR`.                         |

**C.** BETWEEN SYMMETRIC; integer `DIV`/`//`; SQLite row-IN via subquery once Query exists.

**D.** Concat on MySQL vs PG; `^` PG exponent vs MySQL bitwise; `UnaryExpr("!")` DuckDB vs Hive; empty IN; `IsNormalized` PG only; `<=>` on MySQL.

Official: [PG math](https://www.postgresql.org/docs/current/functions-math.html) · [PG 14 factorial](https://www.postgresql.org/docs/14/release-14.html) · [MySQL logical ops](https://dev.mysql.com/doc/refman/8.4/en/logical-operators.html) · [SQLite rowvalue](https://www.sqlite.org/rowvalue.html) · [BQ operators](https://cloud.google.com/bigquery/docs/reference/standard-sql/operators)

---

## 5. CAST, TRIM, COLLATE, CASE, Custom, DataTypeName

**Emit:** `KIND(expr AS type[ FORMAT … [ AT TIME ZONE …]])`; TRIM SQL FROM and comma-list; `expr COLLATE ident`; CASE simple/searched; Custom `{}`; type name verbatim.

### P0

1. **SAFE_CAST** is BigQuery-only. **TRY_CAST** is not on PG/MySQL/SQLite/Hive/Oracle. Teradata is **`TRYCAST`**. Snowflake TRY_CAST is **string source only**.
2. **FORMAT / AT TIME ZONE** is BigQuery CAST grammar (Hive datetime FORMAT; Teradata type FORMAT). Invalid on PG, MySQL (different AT TIME ZONE production), SQL Server, Snowflake, DuckDB, Spark CAST, SQLite, Oracle (positional _fmt_, not keyword FORMAT).
3. **TRIM(LEADING FROM …)** invalid on BigQuery, Snowflake, DuckDB, SQLite, Hive (`TRIM(value[, set])` / `trim(s[, chars])`).
4. **COLLATE ident** invalid on BigQuery (`COLLATE(s,'und:ci')`) and Redshift (`COLLATE(s,'ci')`). Snowflake spec is a **string**.

CASE simple + searched + omitted ELSE → NULL is portable.

### Type-name traps

- BQ: INT64 / STRING / BYTES / BOOL / FLOAT64 / `ARRAY<INT64>` (INT is an alias of INT64).
- MySQL CAST: `SIGNED`, not `INT`.
- MsSql bool: `bit`, not BOOLEAN.
- PG: `bytea`, `timestamp with time zone` (interned `TIMESTAMP WITH TIMEZONE` has no space in TIMEZONE).
- Spark/Hive/Databricks: `ARRAY<INT>`, not `INT ARRAY`.

### Tasks

**A.** Doccomments on Cast / Trim / Collate / Case / Custom / DataTypeName with the URLs in the CAST research (BQ conversion functions, PG type casts, TRIM string functions, COLLATE expressions).

**B.**

- Kind allow-list (table above). Do not map SAFE_CAST → TRY_CAST unless the dialect documents that twin.
- FORMAT / AT TIME ZONE only for BigQuery (Hive FORMAT only datetime↔string).
- TRIM: (1) SQL FROM — PG, MySQL, Redshift, Spark/Databricks, Oracle one-char, Teradata, MsSql 2022; (2) `trim(expr[, chars])` — BQ, Snowflake, DuckDB, SQLite; (3) Hive `trim(expr)` spaces only.
- COLLATE: infix ident (PG, MySQL, MsSql, Oracle, SQLite, DuckDB, Databricks); function+string (BQ, Redshift, Snowflake).
- DataTypeName map for CAST: INT→INT64 (BQ), SIGNED (MySQL), STRING (BQ/Spark/Hive/Databricks), BYTEA (PG), BINARY (Spark), bit (MsSql bool), `ARRAY<T>` vs `T[]`.

**C.** `::`, CONVERT, Oracle `DEFAULT … ON CONVERSION ERROR`, Databricks `?::`.

**D.** Per dialect: CAST kind accepted/rejected; FORMAT omitted except BQ; TRIM form; COLLATE form; `CAST(x AS INT64)` BQ vs `INTEGER` PG vs `SIGNED` MySQL.

Official: [BQ CAST](https://docs.cloud.google.com/bigquery/docs/reference/standard-sql/conversion_functions) · [PG CAST](https://www.postgresql.org/docs/current/sql-expressions.html#SQL-SYNTAX-TYPE-CASTS) · [BQ TRIM](https://docs.cloud.google.com/bigquery/docs/reference/standard-sql/string_functions#trim) · [PG COLLATE](https://www.postgresql.org/docs/current/sql-expressions.html#SQL-SYNTAX-COLLATE-EXPRS) · [Oracle TRIM](https://docs.oracle.com/en/database/oracle/oracle-database/23/sqlrf/TRIM.html)

---

## Suggested order (when you implement)

Do **A (doccomments)** first, Function-style, next to each `write_sql`. Then **B gates that omit invalid SQL** (FILTER, NT, ALL, ILIKE, SIMILAR TO, RLIKE, CAST FORMAT, TRIM FROM, COLLATE, empty IN, Teradata quotes, PG NUL). Then **spelling rewrites** (BQ in-call WHERE, RLIKE `~` / `REGEXP_LIKE`, LIKE ANY negation, Snowflake/DuckDB bytes, Oracle `DATE '…'`). **C** stays later (OVER, WITHIN GROUP, typed JSON, `::`).

**Do not guess.** If a dialect page does not show the production, raise. Silent `LOWER()` / `CONCAT()` / `REGEXP_LIKE()` only after an explicit dialect flag.

**UNVERIFIED leftovers:** Teradata Help Center is a JS SPA (identifier quoting, LIKE ANY, bit `'101'B`). DuckDB `FILTER (WHERE` vs `FILTER (` — railroad includes WHERE. Hive LIKE ESCAPE production not on the current UDF page. Spark OSS 4.2 catalog lists CAST/`::`; `TRY_CAST` is Databricks-documented.
