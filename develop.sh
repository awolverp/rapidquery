function help() {
    echo -e "RapidQuery Project Management\n"
    echo -e "    build    build source"
    echo -e "    test     run mypy & python tests"
    echo -e "    clippy   run rust tests (check, clippy)"
    echo -e "    fmt      format rust & python codes"
    echo -e ""
    echo -e "Uses debug mode on default, use -p command to switch to production mode"
}

function build() {
    if [[ "$1" == "-p" ]]; then
        echo "Building sources (production mode) ..."
        UV_OFFLINE=1 maturin develop --uv --release
    else
        echo "Building sources (debug mode) ..."
        UV_OFFLINE=1 maturin develop --uv
    fi
}

function test() {
    echo "Running tests ..."

    pytest -s -vv
    rm -rf .pytest_cache 2>/dev/null
}

function clippy() {
    if [[ "$1" == "-p" ]]; then
        echo "Running checks (production mode) ..."

        cargo check --release
        cargo clippy --release
        ruff check .
        mypy rapidquery --disable-error-code type-arg --strict
    else
        echo "Running checks (debug mode) ..."

        cargo check
        cargo clippy
        ruff check .
        mypy rapidquery --disable-error-code type-arg --strict
    fi
}

function fmt() {
    echo "Formatting codes ..."

    cargo fmt
	ruff format --line-length=100 .
	ruff clean
}

case "${1:-help}" in
    help)
        help
        ;;
    build)
        shift
        build $@
        ;;
    test)
        shift
        test $@
        ;;
    clippy)
        shift
        clippy $@
        ;;
    fmt)
        shift
        fmt $@
        ;;
    *)
        echo -e "Unknown command: $1. Use 'help' command to see help menu."
        exit 1
        ;;
esac
