import sys
import time
import typing

import pypika
import sqlalchemy as sa
from sqlalchemy.dialects.postgresql import dialect

import rapidquery as rq

# Postgres dialect is faster than other dialects (according to benchmarks)
# and also providing dialect in SQLALchemy can make it faster
SA_DIALECT = dialect()

# Benchmark configuration
ITERATIONS = 100_000
WARMUP_ITERATIONS = 100


def benchmark(func: typing.Callable, number=ITERATIONS) -> float:
    for _ in range(min(WARMUP_ITERATIONS, number // 10)):
        func()

    perf = time.perf_counter_ns()
    for _ in range(number):
        func()
    perf = time.perf_counter_ns() - perf

    return perf / 1000000


def format_results(results: typing.Dict[str, float]) -> str:
    if not results:
        return "No results to display"

    # Find fastest time
    fastest = min(results.values())

    lines = []
    lines.append("-" * 70)
    lines.append(f"{'Library':<20} {'Time (ms)':<15} {'vs Fastest':<15} {'Status':<20}")
    lines.append("-" * 70)

    for lib, time_ms in sorted(results.items(), key=lambda x: x[1]):
        if time_ms == fastest:
            ratio = "1.00x (FASTEST)"
            status = "🏆"
        else:
            ratio = f"{time_ms / fastest:.2f}x slower"
            status = ""

        lines.append(f"{lib:<20} {time_ms:>10.2f}     {ratio:<15} {status}")

    lines.append("-" * 70)
    return "\n".join(lines)


# SELECT Query Benchmarks


def bench_select_rapidquery():
    query = (
        rq.SelectStatement(rq.Expr.asterisk())
        .from_table("users")
        .where(rq.Expr.col("name").like(r"%linus%"))
        .offset(20)
        .limit(20)
    )
    query.to_sql("postgresql")


_users_tb = sa.table("users")


def bench_select_sqlalchemy():
    query = (
        sa.select(sa.text("*"))
        .select_from(_users_tb)
        .where(sa.column("name").like(r"%linus%"))
        .offset(20)
        .limit(20)
    )
    str(query.compile(dialect=SA_DIALECT, compile_kwargs={"literal_binds": True}))


def bench_select_pypika():
    query = (
        pypika.Query.from_("users")
        .where(pypika.Field("name").like(r"%linus%"))
        .offset(20)
        .limit(20)
        .select("*")
    )
    str(query)


# INSERT Query Benchmarks


def bench_insert_rapidquery():
    query = (
        rq.InsertStatement("glyph")
        .columns("aspect", "image")
        .values(5.15, "12A")
        .values(16, "14A")
        .returning(rq.query.Returning("id"))
    )
    query.to_sql("postgresql")


sa_glyph = sa.table("glyph", sa.column("aspect", sa.Float), sa.column("image", sa.String))


def bench_insert_sqlalchemy():
    query = sa.insert(sa_glyph).values(
        [{"aspect": 5.15, "image": "12A"}, {"aspect": 16, "image": "14A"}]
    )
    str(query.compile(dialect=SA_DIALECT, compile_kwargs={"literal_binds": True}))


def bench_insert_pypika():
    query = (
        pypika.Query.into("glyph").columns("aspect", "image").insert(5.15, "12A").insert(16, "14A")
    )
    str(query)


# UPDATE Query Benchmarks


def bench_update_rapidquery():
    query = (
        rq.UpdateStatement("wallets")
        .values(amount=rq.Expr.col("amount") + 10)
        .where(rq.Expr.col("id").between(10, 30))
    )
    query.to_sql("postgresql")


sa_wallets = sa.table("wallets", sa.column("amount", sa.Integer), sa.column("id", sa.Integer))


def bench_update_sqlalchemy():
    query = (
        sa.update(sa_wallets)
        .values(amount=sa_wallets.c.amount + 10)
        .where(sa.between(sa_wallets.c.id, 10, 30))
    )
    str(query.compile(dialect=SA_DIALECT, compile_kwargs={"literal_binds": True}))


def bench_update_pypika():
    query = (
        pypika.Query.update("wallets")
        .set("amount", pypika.Field("amount") + 10)
        .where(pypika.Field("id").between(10, 30))
    )
    str(query)


# DELETE Query Benchmarks


def bench_delete_rapidquery():
    query = rq.DeleteStatement("users").where(
        rq.all(
            rq.Expr.col("id") > 10,
            rq.Expr.col("id") < 30,
        )
    )
    query.to_sql("postgresql")


sa_users = sa.table("users", sa.column("id", sa.Integer))


def bench_delete_sqlalchemy():
    query = sa.delete(sa_users).where(sa.and_(sa_users.c.id > 10, sa_users.c.id < 30))
    str(query.compile(dialect=SA_DIALECT, compile_kwargs={"literal_binds": True}))


def bench_delete_pypika():
    query = (
        pypika.Query.from_("users")
        .where((pypika.Field("id") > 10) & (pypika.Field("id") < 30))
        .delete()
    )
    str(query)


def run_benchmarks():
    print(f"Iterations per test: {ITERATIONS:,}")
    print(f"Python version: {sys.version.split()[0]}")
    print()

    print("\n📊 SELECT Query Benchmark")
    results = {
        "RapidQuery": benchmark(bench_select_rapidquery),
        "SQLAlchemy": benchmark(bench_select_sqlalchemy),
        "PyPika": benchmark(bench_select_pypika),
    }
    print(format_results(results))

    print("\n📊 INSERT Query Benchmark")
    results = {
        "RapidQuery": benchmark(bench_insert_rapidquery),
        "SQLAlchemy": benchmark(bench_insert_sqlalchemy),
        "PyPika": benchmark(bench_insert_pypika),
    }
    print(format_results(results))

    print("\n📊 UPDATE Query Benchmark")
    results = {
        "RapidQuery": benchmark(bench_update_rapidquery),
        "SQLAlchemy": benchmark(bench_update_sqlalchemy),
        "PyPika": benchmark(bench_update_pypika),
    }
    print(format_results(results))

    print("\n📊 DELETE Query Benchmark")
    results = {
        "RapidQuery": benchmark(bench_delete_rapidquery),
        "SQLAlchemy": benchmark(bench_delete_sqlalchemy),
        "PyPika": benchmark(bench_delete_pypika),
    }
    print(format_results(results))


if __name__ == "__main__":
    run_benchmarks()
