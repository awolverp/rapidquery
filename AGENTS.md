# RapidQuery — agent notes

High-performance Python SQL query builder. Public API is Python; the engine is Rust (PyO3 + maturin). Package: `rapidquery`. Native module: `rapidquery._core`.

This branch (`refactor/v1`) restarts the closed draft PR **[#2](https://github.com/awolverp/rapidquery/pull/2)** (`origin/migrate` @ `77c6267`). That PR is the **architectural target**. It is not a SeaQuery port and it is not a vendored `sqlparser-rs` — it is a custom engine *inspired by* sqlparser-rs.

Goals from the PR (keep these):
- No SeaQuery panics; production-safe SQL building.
- 13 dialects (BigQuery, Databricks, DuckDB, Hive, MSSQL, MySQL, Oracle, PostgreSQL, Redshift, Snowflake, Spark, SQLite, Teradata).
- Full SQL data-type coverage via interned `DataTypeName` + `DataTypeSerializer`.
- Native Rust vs Python bindings split (`src/native/` vs `src/pybindings/`). Native may use PyO3 where it holds or inspects Python objects (`Serializer`, `PrepareContext`).
- Dual SQL rendering via `WriteSql` + `SqlWriter`: raw (`SqlBuf`, literals) and parameterized (`PrepareContext`, placeholders).
- Ident / Asterisk with dialect quoting.
- Leave the door open for a SQL parser and an ORM later.
- Prefer backward compatibility with v0, but it is **not** guaranteed.

Do **not** restore v0 (`_lib`, SeaQuery, three backends) unless the user asks.

Keep these `refactor/v1` divergences from the PR (do not revert them):
- Module name `rapidquery._core` (PR used `_native`).
- Proc-macros `pyclass` / `pyerr` (PR used `pyclass_shortcut` + `new_pyerr!`).
- MIT license, Python `>=3.10`, crate version `1.0.0-alpha1`.
- `PrepareContext` / `PrepareError` naming in `sql.rs` (PR used `PrepareStatementContext` / `PrepareStatementError`).

## Stack

- Python `>=3.10` (local: 3.13). Rust edition **2024**. PyO3 **0.29**. Maturin **1.15.0**. uv for the venv.
- Workspace crate `rapidquery-macros` (proc-macros: `pyclass`, `pyerr`).
- Target stack from PR #2 also used: `arcstr`, `once_cell`, `chrono`, `ipnetwork`, `mac_address`. (`derive_more` is not on this branch.)
- Dialects (13), `DataType` + `Serializer`, `Ident`/`Unquote`/`Asterisk`, `SqlWriter`/`WriteSql` exist on this branch. Still missing vs the PR’s next layers: `Value`, query/schema AST (never in the PR either).
- `#[pyo3::pymodule(gil_used = false)]` — keep free-threaded Python in mind.
- Frozen/immutable PyO3 types: `rapidquery_macros::pyclass` already sets `module="rapidquery._core"`, `immutable_type`, `frozen`.

## Layout

```
src/native/           Rust engine (dialects, DataTypeName, Ident, Serializer, SqlWriter/WriteSql)
src/pybindings/       Python wrappers (pyclasses only)
src/internal/         aliases, string utils, submodule helper
src/typeref.rs        cached PyTypeObject pointers
rapidquery/           Python package (re-exports Ident, Unquote, Asterisk, data_type, dialect)
rapidquery-macros/    Proc-macros
tests/                Python tests
backup/, backup-py/   gitignored v0 snapshots — read-only
target/, .venv/       Generated — do not commit or edit
```

Reference the PR without checking it out as the working tree: `git show origin/migrate:<path>`. Do not copy files wholesale if this branch already renamed or cleaned them.

## Commands

```bash
uv sync --dev
maturin develop          # or: uv run maturin develop
cargo test --workspace --lib --bins --tests
cargo clippy --workspace --all-targets -- -D warnings   # project lints are warn-level
cargo fmt                # rustfmt.toml uses nightly-only options; CI uses nightly
uv run pytest -vv        # CI command; pytest is not in pyproject yet
```

There is no Python linter/typechecker configured. Do not add ruff/mypy/a new test runner unless asked.

After meaningful Rust or Python changes: run `cargo test --workspace --lib --bins --tests`. Run pytest only after Python tests exist.

`cargo test --workspace` (including doctests) currently fails: `pyerr` docs in `rapidquery-macros/lib.rs` are not valid rustdoc tests.

## How to edit

- Small, reviewable diffs. Match existing patterns on **this** branch first; for missing layers, match `origin/migrate` (native owns dialects, `DataTypeName`, `Ident`, `Serializer`+`write_sql`, `SqlWriter`/`WriteSql`; pybindings are pyclasses wrapping those; serializers subclass a base `Serializer`; dialects subclass `SqlDialect` and are `@final`). Native may use PyO3 where it holds or inspects Python objects.
- Do not introduce SeaQuery, a sqlparser-rs crate dependency, SQLAlchemy, another Python package layout, or a new build backend.
- Do not touch `Cargo.lock` / `uv.lock` unless the change requires a dependency update. Do not touch generated artifacts (`target/`, `*.so`, wheels), secrets, or `.env`.
- `backup/` and `backup-py/` are gitignored snapshots of v0. Do not edit them; do not copy them wholesale into `src/`.
- SQL injection safety is a product requirement. Prefer parameterized `PrepareContext::push_param` over interpolating values. Dialect `escape_*` / `quote_*` must not let payloads close literals.
- Unsafe is allowed where the crate already uses it (`from_utf8_unchecked` after ASCII-boundary splits, PyO3 FFI). Keep the safety invariant next to the block.
- Clippy: no `dbg!`, `println!`, or `eprintln!` in library code (warn). Release profile uses `panic = "abort"`.

## Docs drift

`README.md` still describes v0.1.0 (SeaQuery, three backends, `_lib`). Treat `Cargo.toml` + `src/` as source of truth for what is implemented; treat PR #2 / `origin/migrate` as source of truth for what comes next.
