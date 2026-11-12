# RapidQuery

Python SQL query builder. Behavior lives in the Rust/PyO3 extension `rapidquery._core`. `rapidquery/*.py` re-exports it. `rapidquery/*.pyi` is hand-written.

This branch is incomplete.

## Commands

- Setup: `make install`
- Dev extension: `make build-dev` (`maturin develop`)
- Release extension: `make build-prod`
- Format: `make format` (ruff, then `cargo fmt`)
- Lint: `make lint` (ruff check/format, pyright, `cargo fmt --check`, `cargo clippy --tests -- -D warnings`)
- Rust tests: `make test-rust` (`--no-default-features --features auto-initialize,nightly`). Default features do not link libpython.
- Python tests: `make test-python` (`pytest -v`). Rebuild the extension first when Rust changed.
- Full local gate: `make` (format, build-dev, lint, both test targets).

CI (`.github/workflows/test.yml`) runs pytest only, on CPython 3.10–3.15, 3.14t/3.15t, and PyPy 3.11. Free-threaded jobs may fail. Rust tests and clippy are local.

## Editing

- Small diffs. Follow nearby modules. No new language, framework, or build tool.
- Python `>=3.10`. Ruff and rustfmt width are 120. Rustfmt uses `imports_granularity = "Item"` (`.rustfmt.toml`). Edition 2024. Nightly is for formatting only.
- A public type needs three matching pieces: `#[rapidquery_macros::pyclass("rapidquery.<mod>.Name", ...)]`, a re-export in `rapidquery/<mod>.py`, and the same shape in `rapidquery/<mod>.pyi`.
- New SQL should be Standard SQL or supported by at least three dialects (`CONTRIBUTING.md`).
- `sea-query` and `sqlparser` are unused inspiration dependencies. Leave them unused.
- PyO3 modules set `gil_used = false`.
- After a meaningful change, run the tests that cover it.

### Dialects

- A dialect parameter defaults to disabled or unsupported. Keep a default enabled only when rendering needs it on every dialect: identifier quoting, string literals, the placeholder, and other standard spelling.
- Keep parameters and `OPT_*` constants few. Extend an existing flag set or enum before adding another field. Do not add an `OPT_*` for a clause an existing flag already covers.

### Documentation

- Rust items need rustdoc. Add a usage example when it shows how to call the item. Doctests are not maintained: fence the example with `ignore`, or use a fence other than `rust`, so `cargo test` does not run it.
- On a PyO3 class, method, or function, a one-line summary is enough. The long docstring lives in the matching `.pyi`.
- Python docstrings are Google style for mkdocstrings: a summary line, a blank line, then `Args`, `Returns`, `Raises`, and `Examples` as needed. Put a usage example on a public class. Prose that follows a section belongs under `Note:`, not as a loose paragraph. Follow the `>>>` examples already in `rapidquery/expr.pyi`.

## Leave alone

- Generated or local artifacts: `target/`, `rapidquery/*.so`, `__pycache__/`, `.venv/`.
- Lockfiles (`uv.lock`, `Cargo.lock`) unless a dependency change is requested.
- Secrets and `.env`.
- Local scratch excluded in `.git/info/exclude`: `a.py`, `builtin.rs`.
