# RapidQuery

Python SQL query builder. Behavior lives in the Rust/PyO3 extension `rapidquery._core`. `rapidquery/*.py` re-exports it. `rapidquery/*.pyi` is hand-written.

This branch is incomplete. Shipped DDL is `CREATE SCHEMA` / `DROP SCHEMA`. DML, DCL, and TCL are base classes only.

## Commands

- Setup: `make install`
- Dev extension: `make build-dev` (`maturin develop`)
- Release extension: `make build-prod`
- Format: `make format` (ruff, then `cargo +nightly fmt`)
- Lint: `make lint` (ruff check/format, nightly rustfmt, `cargo clippy --tests -- -D warnings`)
- Rust tests: `make test-rust` (`--no-default-features --features auto-initialize`). Default features do not link libpython.
- Python tests: `make test-python` (`pytest -v -n auto`). Rebuild the extension first when Rust changed.
- Full local gate: `make` (format, lint, build-dev, both test targets).

CI (`.github/workflows/test.yml`) runs pytest only, on CPython 3.10–3.15, 3.14t/3.15t, and PyPy 3.11. Free-threaded jobs may fail. Rust tests and clippy are local.

## Editing

- Small diffs. Follow nearby modules. No new language, framework, or build tool.
- Python `>=3.10`. Ruff and rustfmt width are 120. Rustfmt uses `imports_granularity = "Item"` (`.rustfmt.toml`). Edition 2024. Nightly is for formatting only.
- A public type needs three matching pieces: `#[rapidquery_macros::pyclass("rapidquery.<mod>.Name", ...)]`, a re-export in `rapidquery/<mod>.py`, and the same shape in `rapidquery/<mod>.pyi`.
- New SQL should be Standard SQL or supported by at least three dialects (`CONTRIBUTING.md`).
- `sea-query` and `sqlparser` are unused inspiration dependencies. Leave them unused.
- PyO3 modules set `gil_used = false`.
- After a meaningful change, run the tests that cover it.

## Leave alone

- Generated or local artifacts: `target/`, `rapidquery/*.so`, `__pycache__/`, `.venv/`.
- Lockfiles (`uv.lock`, `Cargo.lock`) unless a dependency change is requested.
- Secrets and `.env`.
- Local scratch excluded in `.git/info/exclude`: `a.py`, `builtin.rs`.
