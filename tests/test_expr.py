from rapidquery.dialect import PostgreSQL
from rapidquery.expr import Asterisk, BinaryExpr, Function, Ident, Unquote, Value


class TestFunction:
    def test_new(self):
        fn = Function("now")
        assert fn.get_name() == Ident("now")
        assert fn.get_distinct() is None
        assert fn.get_args() is None
        assert fn.get_filter() is None

        fn = Function("example", filter=BinaryExpr(1, ">", 2))
        assert fn.get_name() == Ident("example")
        assert fn.get_distinct() is None
        assert fn.get_args() is None
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = Function("count", ["id"], distinct=False, filter=BinaryExpr(1, ">", 2))
        assert fn.get_name() == Ident("count")
        assert fn.get_distinct() is False
        assert fn.get_args() == (Ident("id"),)
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = Function(Unquote("NOW"))
        assert fn.get_name() == Ident(Unquote("NOW"))

        fn = Function("schema.fn")
        assert fn.get_name() == Ident("schema", "fn")

    def test_to_sql(self):
        fn = Function("current_timestamp")
        assert fn.to_sql(PostgreSQL) == '"current_timestamp"'

        fn = Function(Unquote("CURRENT_TIMESTAMP"))
        assert fn.to_sql(PostgreSQL) == "CURRENT_TIMESTAMP"

        fn = Function(Unquote("NOW"), [])
        assert fn.to_sql(PostgreSQL) == "NOW()"

        fn = Function(Unquote("NOW"))
        assert fn.to_sql(PostgreSQL) == "NOW"

        fn = Function(
            "count", [Asterisk], filter=BinaryExpr("status", "=", Value("completed"))
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(*) FILTER (WHERE "status" = E\'completed\')'
        )

        fn = Function(
            "count",
            ["id"],
            distinct=False,
            filter=BinaryExpr("status", "=", Value("completed")),
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(ALL "id") FILTER (WHERE "status" = E\'completed\')'
        )

        fn = Function(
            "count",
            ["id"],
            distinct=True,
            filter=BinaryExpr("status", "=", Value("completed")),
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(DISTINCT "id") FILTER (WHERE "status" = E\'completed\')'
        )

        fn = Function(
            "now",
            [],
            distinct=True,
        )
        assert fn.to_sql(PostgreSQL) == '"now"()'

    def test_edge_cases(self):
        # TODO
        pass
