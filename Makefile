# Customized version of https://github.com/pydantic/pydantic/raw/refs/heads/main/pydantic-core/Makefile

.DEFAULT_GOAL := all

uv-run-no-sync = uv run --no-sync

# using pip install cargo (via maturin via pip) doesn't get the tty handle
# so doesn't render color without some help
export CARGO_TERM_COLOR=$(shell (test -t 0 && echo "always") || echo "auto")

.PHONY: .uv  ## Check that uv is installed
.uv:
	@uv -V || echo 'Please install uv: https://docs.astral.sh/uv/getting-started/installation/'

.PHONY: install  ## Install the package, dependencies, and pre-commit for local development
install: .uv
	uv sync --frozen --all-groups
	uv run pre-commit install --install-hooks

.PHONY: rebuild-lockfiles  ## Rebuild lockfiles from scratch, updating all dependencies
rebuild-lockfiles: .uv
	uv lock --upgrade

.PHONY: build-dev  ## Build the development version of the package
build-dev:
	@rm -f rapidquery/*.so
	$(uv-run-no-sync) maturin develop --uv

.PHONY: build-prod  ## Build the production version of the package
build-prod:
	@rm -f rapidquery/*.so
	$(uv-run-no-sync) maturin develop --uv --release

.PHONY: format  ## Auto-format rust and python source files
format:
	uv run ruff check --fix .
	uv run ruff format .
	cargo +nightly fmt

.PHONY: lint-python  ## Lint python source files
lint-python:
	uv run ruff check .
	uv run ruff format --check .

.PHONY: lint-rust  ## Lint rust source files
lint-rust:
	cargo +nightly fmt --version
	cargo +nightly fmt --all -- --check
	cargo clippy --version
	cargo clippy --tests -- -D warnings

.PHONY: lint  ## Lint rust and python source files
lint: lint-python lint-rust

.PHONY: test  ## Run all tests
test:
	uv run pytest

.PHONY: all  ## Run the standard set of checks performed in CI
all: format lint build-dev test

.PHONY: clean  ## Clear local caches and build artifacts
clean:
	rm -rf `find . -name __pycache__`
	rm -f `find . -type f -name '*.py[co]' `
	rm -f `find . -type f -name '*~' `
	rm -f `find . -type f -name '.*~' `
	rm -rf .pytest_cache
	rm -rf .ruff_cache
	rm -rf *.egg-info
	rm -rf rapidquery/*.so

.PHONY: help  ## Display this message
help:
	@grep -E \
		'^.PHONY: .*?## .*$$' $(MAKEFILE_LIST) | \
		sort | \
		awk 'BEGIN {FS = ".PHONY: |## "}; {printf "\033[36m%-19s\033[0m %s\n", $$2, $$3}'
