BUILD_CMD := maturin develop

.DEFAULT_GOAL := help
.PHONY: help build-dev build-prod test fmt stubgen

help:
	@echo "RapidQuery Project Management"
	@echo ""
	@echo -e "    build-dev     build source"
	@echo -e "    build-prod    build source (release mode)"
	@echo -e "    test          run clippy and pytest in debug mode"
	@echo -e "    test-full     run clippy and pytest in debug mode and release mode"
	@echo -e "    fmt           format rust and python code"
	@echo -e "    stubgen       Use pyo3-inspection to generate stubfiles"

build-dev:
	UV_OFFLINE=1 $(BUILD_CMD) --uv

build-prod:
	$(BUILD_CMD) --uv --release

test:
	$(BUILD_CMD) --uv
	pytest -s -vv
	-rm -rf .pytest_cache
	-ruff check .
	ruff clean
	mypy rapidquery --disable-error-code type-arg --strict

fmt:
	cargo fmt
	ruff format --line-length=100 .
	ruff clean

ready: fmt test-full

stubgen:
	python3 tools/stubgen.py rapidquery._lib
	ruff check --fix rapidquery/_lib
	ruff format --line-length=100 .
	mypy rapidquery --disable-error-code override --disable-error-code type-arg --disable-error-code no-untyped-def --strict
	ruff clean
	rm -rf .mypy_cache
