import pytest

from rapidquery.dialect import BigQuery, DuckDB, PostgreSQL, SQLite, Snowflake, SqlDialect
from rapidquery.expr import (
    Asterisk,
    BetweenExpr,
    BinaryExpr,
    CaseExpr,
    CastExpr,
    CustomExpr,
    Expr,
    Function,
    Ident,
    InListExpr,
    IsDistinctFromExpr,
    IsExpr,
    LikeExpr,
    RLikeExpr,
    SimilarToExpr,
    TupleExpr,
    UnaryExpr,
    Unquote,
    Value,
)


class TestExpr:
    def test_cannot_instantiate_base(self):
        with pytest.raises(TypeError):
            Expr()  # pyright: ignore

    def test_nodes_are_expr_subclasses(self):
        nodes = [
            Ident("id"),
            Value(1),
            Function("now"),
            TupleExpr(1),
            UnaryExpr("-", 1),
            BinaryExpr(1, "+", 2),
            IsExpr(Ident("id"), "NULL"),
            IsDistinctFromExpr(1, 2),
            InListExpr(Ident("id"), [1]),
            BetweenExpr(Ident("id"), 0, 1),
            LikeExpr(Ident("name"), "%a%"),
            SimilarToExpr(Ident("name"), "%a%"),
            RLikeExpr(Ident("name"), "%a%"),
            CastExpr(Ident("id"), "int"),
            CaseExpr(),
            CustomExpr("1"),
        ]
        for node in nodes:
            assert isinstance(node, Expr)

    def test_to_sql_and_prepare_accept_class_or_instance(self):
        expr = BinaryExpr(Ident("id"), ">", 10)
        assert expr.to_sql(PostgreSQL) == expr.to_sql(PostgreSQL())
        assert expr.prepare(PostgreSQL)[0] == expr.prepare(PostgreSQL())[0]

    def test_render_succeeds_on_every_dialect(self, dialect_cls: type[SqlDialect]):
        expr = BinaryExpr(Ident("id"), ">", 10)
        sql = expr.to_sql(dialect_cls)
        assert "id" in sql
        assert "10" in sql


class TestFunction:
    def test_new(self):
        fn = Function("now")
        assert fn.get_name() == Ident("now")
        assert fn.get_distinct() is None
        assert fn.get_args() is None
        assert fn.get_filter() is None
        assert fn.get_null_treatment() is None

        fn = Function("example").filter(BinaryExpr(1, ">", 2))
        assert fn.get_name() == Ident("example")
        assert fn.get_distinct() is None
        assert fn.get_args() is None
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = (
            Function("count")
            .call(Ident("id"), distinct=False)
            .filter(BinaryExpr(1, ">", 2))
        )
        assert fn.get_name() == Ident("count")
        assert fn.get_distinct() is False
        assert fn.get_args() == (Ident("id"),)
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = Function(Unquote("NOW"))
        assert fn.get_name() == Ident(Unquote("NOW"))

        fn = Function("schema.fn")
        assert fn.get_name() == Ident("schema", "fn")

        bare = Function("now")
        called = bare.call()
        assert called is not bare
        assert called.get_args() == ()
        assert bare.get_args() is None

    def test_to_sql(self):
        fn = Function("current_timestamp")
        assert fn.to_sql(PostgreSQL) == '"current_timestamp"'

        fn = Function(Unquote("CURRENT_TIMESTAMP"))
        assert fn.to_sql(PostgreSQL) == "CURRENT_TIMESTAMP"

        fn = Function(Unquote("NOW")).call()
        assert fn.to_sql(PostgreSQL) == "NOW()"

        fn = Function(Unquote("NOW"))
        assert fn.to_sql(PostgreSQL) == "NOW"

        fn = (
            Function("count")
            .call(Asterisk)
            .filter(BinaryExpr(Ident("status"), "=", "completed"))
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(*) FILTER (WHERE "status" = E\'completed\')'
        )

        fn = (
            Function("count")
            .call(Ident("id"), distinct=False)
            .filter(BinaryExpr(Ident("status"), "=", "completed"))
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(ALL "id") FILTER (WHERE "status" = E\'completed\')'
        )

        fn = (
            Function("count")
            .call(Ident("id"), distinct=True)
            .filter(BinaryExpr(Ident("status"), "=", "completed"))
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"(DISTINCT "id") FILTER (WHERE "status" = E\'completed\')'
        )

        fn = Function("now").call(distinct=True)
        assert fn.to_sql(PostgreSQL) == '"now"()'

        fn = Function("custom").call(1, 2, Ident("id"), distinct=True)
        assert fn.to_sql(PostgreSQL) == '"custom"(DISTINCT 1, 2, "id")'

        fn = Function("lag").call(Ident("x")).ignore_nulls()
        assert fn.to_sql(PostgreSQL) == '"lag"("x") IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(`x` IGNORE NULLS)"
        assert fn.to_sql(DuckDB) == '"lag"("x" IGNORE NULLS)'
        assert fn.get_null_treatment() == "IGNORE NULLS"

        fn = Function("lag").call(Ident("x")).respect_nulls()
        assert fn.to_sql(PostgreSQL) == '"lag"("x") RESPECT NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(`x` RESPECT NULLS)"
        assert fn.to_sql(DuckDB) == '"lag"("x" RESPECT NULLS)'
        assert fn.get_null_treatment() == "RESPECT NULLS"

        fn = Function("lag").call().ignore_nulls()
        assert fn.to_sql(PostgreSQL) == '"lag"() IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(IGNORE NULLS)"

        fn = Function("lag").ignore_nulls()
        assert fn.to_sql(PostgreSQL) == '"lag" IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`lag` IGNORE NULLS"

        fn = (
            Function("count")
            .call(Ident("id"))
            .ignore_nulls()
            .filter(BinaryExpr(Ident("status"), "=", "completed"))
        )
        assert (
            fn.to_sql(PostgreSQL)
            == '"count"("id") IGNORE NULLS FILTER (WHERE "status" = E\'completed\')'
        )
        assert (
            fn.to_sql(BigQuery)
            == "`count`(`id` IGNORE NULLS) FILTER (WHERE `status` = 'completed')"
        )

    def test_edge_cases(self):
        with pytest.raises(TypeError):
            Function(1)  # pyright: ignore

        with pytest.raises(ValueError, match="empty"):
            Function("")

        with pytest.raises(TypeError, match="condition"):
            Function("now").filter(Ident("x"))  # pyright: ignore

        with pytest.raises(TypeError, match="condition"):
            Function("now").filter(1)  # pyright: ignore

        fn = Function("f").call("id")
        assert fn.get_args() == (Value("id"),)
        assert fn.to_sql(PostgreSQL) == "\"f\"(E'id')"

        fn = Function(Ident("schema.fn")).call()
        assert fn.get_name() == Ident("schema.fn")
        assert fn.to_sql(PostgreSQL) == '"schema.fn"()'

        first = Function("f").call(1)
        second = first.call(2, 3)
        assert first.get_args() == (Value(1),)
        assert second.get_args() == (Value(2), Value(3))

        filtered = Function("f").filter(BinaryExpr(1, ">", 0))
        replaced = filtered.filter(BinaryExpr(2, ">", 0))
        assert filtered.to_sql(PostgreSQL) == '"f" FILTER (WHERE 1 > 0)'
        assert replaced.to_sql(PostgreSQL) == '"f" FILTER (WHERE 2 > 0)'

        ignored = Function("lag").call(Ident("x")).ignore_nulls()
        respected = ignored.respect_nulls()
        assert ignored.get_null_treatment() == "IGNORE NULLS"
        assert respected.get_null_treatment() == "RESPECT NULLS"
        assert ignored.to_sql(PostgreSQL) == '"lag"("x") IGNORE NULLS'
        assert respected.to_sql(PostgreSQL) == '"lag"("x") RESPECT NULLS'

        fn = Function("count").call(1).filter(BinaryExpr(Ident("x"), ">", 2))
        sql, params = fn.prepare(PostgreSQL)
        assert sql == '"count"(:param_0) FILTER (WHERE "x" > :param_1)'
        assert params == {"param_0": 1, "param_1": 2}

        assert (
            repr(Function("lag").call(Ident("x")).ignore_nulls())
            == 'Function(Ident("lag")).call(Ident("x")).ignore_nulls()'
        )


class TestTupleExpr:
    def test_new(self):
        tpl = TupleExpr(1, 2, "A")
        assert tpl.get_items() == (Value(1), Value(2), Value("A"))

        tpl = TupleExpr(1, 2, Ident("A"))
        assert tpl.get_items() == (Value(1), Value(2), Ident("A"))

        empty = TupleExpr()
        assert empty.get_items() == ()

        nested = TupleExpr((1, 2), Ident("A"))
        assert isinstance(nested.get_items()[0], TupleExpr)
        assert nested.get_items()[0].get_items() == (Value(1), Value(2))
        assert nested.get_items()[1] == Ident("A")

    def test_to_sql(self):
        tpl = TupleExpr(1, 2, "A")
        assert tpl.to_sql(PostgreSQL) == "(1, 2, E'A')"

        sql, params = tpl.prepare(PostgreSQL)
        assert sql == "(:param_0, :param_1, :param_2)"
        assert params == {"param_0": 1, "param_1": 2, "param_2": "A"}

        assert TupleExpr().to_sql(PostgreSQL) == "()"
        assert TupleExpr(1).to_sql(PostgreSQL) == "(1)"
        assert TupleExpr((1, 2), Ident("A")).to_sql(PostgreSQL) == '((1, 2), "A")'


class TestUnaryExpr:
    def test_new(self):
        un = UnaryExpr("NOT", 1)
        assert un.get_expr() == Value(1)
        assert un.get_operator() == "NOT"

        un = UnaryExpr("Not", 1)
        assert un.get_operator() == "NOT"

        un = UnaryExpr("-", 1)
        assert un.get_operator() == "-"

        un = UnaryExpr("+", Ident("id"))
        assert un.get_operator() == "+"
        assert un.get_expr() == Ident("id")

        un = UnaryExpr("~", 1)
        assert un.get_operator() == "~"

        with pytest.raises(ValueError, match="unknown unary operator"):
            UnaryExpr("!", 1)

    def test_to_sql(self):
        un = UnaryExpr("-", Function("now").call())
        assert un.to_sql(PostgreSQL) == '-"now"()'

        assert UnaryExpr("NOT", 1).to_sql(PostgreSQL) == "NOT 1"
        assert UnaryExpr("NOT", Ident("id")).to_sql(PostgreSQL) == 'NOT "id"'
        assert UnaryExpr("+", 1).to_sql(PostgreSQL) == "+1"
        assert UnaryExpr("~", Ident("id")).to_sql(PostgreSQL) == '~"id"'
        assert UnaryExpr("-", UnaryExpr("-", 1)).to_sql(PostgreSQL) == "-(-1)"
        assert UnaryExpr("NOT", UnaryExpr("NOT", 1)).to_sql(PostgreSQL) == "NOT (NOT 1)"
        assert UnaryExpr("-", BinaryExpr(1, "+", 2)).to_sql(PostgreSQL) == "-(1 + 2)"

        sql, params = UnaryExpr("-", 1).prepare(PostgreSQL)
        assert sql == "-:param_0"
        assert params == {"param_0": 1}


class TestBinaryExpr:
    def test_new(self):
        bin = BinaryExpr(Ident("id"), ">", 10)
        assert bin.get_left() == Ident("id")
        assert bin.get_operator() == ">"
        assert bin.get_right() == Value(10)

        bin = BinaryExpr("str", "-custom-", 10)
        assert bin.get_left() == Value("str")
        assert bin.get_operator() == "-custom-"
        assert bin.get_right() == Value(10)

        bin = BinaryExpr(1, "!=", 2)
        assert bin.get_operator() == "<>"

        bin = BinaryExpr(1, "and", 2)
        assert bin.get_operator() == "AND"
        bin = BinaryExpr(1, "Or", 2)
        assert bin.get_operator() == "OR"
        bin = BinaryExpr(1, "xor", 2)
        assert bin.get_operator() == "XOR"

    def test_to_sql(self):
        bin = BinaryExpr(Ident("id"), ">", 10)
        assert bin.to_sql(PostgreSQL) == '"id" > 10'

        bin = BinaryExpr("str", "-custom-", 10)
        assert bin.to_sql(PostgreSQL) == "E'str' -custom- 10"

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "&", 10))
        assert bin.to_sql(PostgreSQL) == '"id" > 456 & 10'

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "CusTom", 10))
        assert bin.to_sql(PostgreSQL) == '"id" > (456 CusTom 10)'

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "<", 10))
        assert bin.to_sql(PostgreSQL) == '"id" > (456 < 10)'

        assert BinaryExpr(1, "!=", 2).to_sql(PostgreSQL) == "1 <> 2"
        assert BinaryExpr(1, "AND", 2).to_sql(PostgreSQL) == "1 AND 2"
        assert (
            BinaryExpr(BinaryExpr(1, "+", 2), "AND", 3).to_sql(PostgreSQL)
            == "1 + 2 AND 3"
        )
        assert (
            BinaryExpr(1, "+", BinaryExpr(2, "AND", 3)).to_sql(PostgreSQL)
            == "1 + (2 AND 3)"
        )

        sql, params = BinaryExpr(Ident("id"), ">", 10).prepare(PostgreSQL)
        assert sql == '"id" > :param_0'
        assert params == {"param_0": 10}

    def test_repr(self):
        assert (
            repr(BinaryExpr(Ident("id"), ">", 10))
            == 'BinaryExpr(Ident("id"), ">", Value(10, IntegerSerializer()))'
        )


class TestIsExpr:
    def test_new(self):
        is_ = IsExpr(Ident("id"), "null")
        assert is_.get_expr() == Ident("id")
        assert is_.get_kind() == "NULL"
        assert is_.get_negated() is False

        is_ = IsExpr(None, "UNKNOWN", negated=True)
        assert is_.get_expr() == Value(None)
        assert is_.get_kind() == "UNKNOWN"
        assert is_.get_negated() is True

        with pytest.raises(ValueError, match="unknown IS kind"):
            IsExpr(Ident("id"), "MAYBE")

    def test_to_sql(self):
        is_ = IsExpr(Ident("id"), "NUll")
        assert is_.to_sql(PostgreSQL) == '"id" IS NULL'

        is_ = IsExpr(Ident("id"), "FalSE")
        assert is_.to_sql(PostgreSQL) == '"id" IS FALSE'

        is_ = IsExpr(Ident("id"), "tRuE")
        assert is_.to_sql(PostgreSQL) == '"id" IS TRUE'

        is_ = IsExpr(Ident("id"), "unknown", negated=True)
        assert is_.to_sql(PostgreSQL) == '"id" IS NOT UNKNOWN'

        sql, params = IsExpr(Ident("id"), "NULL").prepare(PostgreSQL)
        assert sql == '"id" IS NULL'
        assert params == {}

        sql, params = IsExpr(1, "TRUE").prepare(PostgreSQL)
        assert sql == ":param_0 IS TRUE"
        assert params == {"param_0": 1}

    def test_repr(self):
        assert repr(IsExpr(Ident("id"), "NULL")) == 'IsExpr(Ident("id"), "NULL")'
        assert (
            repr(IsExpr(Ident("id"), "NULL", negated=True))
            == 'IsExpr(Ident("id"), "NULL", negated=True)'
        )


class TestIsDistinctFromExpr:
    def test_new(self):
        idf = IsDistinctFromExpr(2, 3)
        assert idf.get_left() == Value(2)
        assert idf.get_right() == Value(3)
        assert idf.get_negated() is False

        idf = IsDistinctFromExpr("A", "B")
        assert idf.get_left() == Value("A")
        assert idf.get_right() == Value("B")
        assert idf.get_negated() is False

        idf = IsDistinctFromExpr(Ident("id"), 10, negated=True)
        assert idf.get_left() == Ident("id")
        assert idf.get_right() == Value(10)
        assert idf.get_negated() is True

        idf = IsDistinctFromExpr(Ident("a"), Ident("b"))
        assert idf.get_left() == Ident("a")
        assert idf.get_right() == Ident("b")

    def test_to_sql(self):
        idf = IsDistinctFromExpr(2, 3)
        assert idf.to_sql(PostgreSQL) == "2 IS DISTINCT FROM 3"

        idf = IsDistinctFromExpr(Ident("id"), 10, negated=True)
        assert idf.to_sql(PostgreSQL) == '"id" IS NOT DISTINCT FROM 10'

        sql, params = idf.prepare(PostgreSQL)
        assert sql == '"id" IS NOT DISTINCT FROM :param_0'
        assert params == {"param_0": 10}

        assert (
            IsDistinctFromExpr(Ident("a"), Ident("b")).to_sql(PostgreSQL)
            == '"a" IS DISTINCT FROM "b"'
        )


class TestInListExpr:
    def test_new(self):
        lst = InListExpr(Ident("id"), [1, 2, 3])
        assert lst.get_expr() == Ident("id")
        assert lst.get_list() == (Value(1), Value(2), Value(3))
        assert lst.get_negated() is False

        lst = InListExpr("A", [1, Ident("id"), 3], negated=True)
        assert lst.get_expr() == Value("A")
        assert lst.get_list() == (Value(1), Ident("id"), Value(3))
        assert lst.get_negated() is True

        lst = InListExpr(Ident("id"), [])
        assert lst.get_list() == ()

        with pytest.raises(TypeError):
            InListExpr(Ident("id"), 1)  # pyright: ignore

    def test_to_sql(self):
        lst = InListExpr(Ident("id"), [1, 2, 3])
        assert lst.to_sql(PostgreSQL) == '"id" IN (1, 2, 3)'

        sql, params = lst.prepare(PostgreSQL)
        assert sql == '"id" IN (:param_0, :param_1, :param_2)'
        assert params == {"param_0": 1, "param_1": 2, "param_2": 3}

        assert InListExpr(Ident("id"), [], negated=True).to_sql(PostgreSQL) == (
            '"id" NOT IN ()'
        )
        assert InListExpr(Ident("id"), [1], negated=True).to_sql(PostgreSQL) == (
            '"id" NOT IN (1)'
        )


class TestBetweenExpr:
    def test_new(self):
        bw = BetweenExpr(Ident("id"), 0, 10)
        assert bw.get_expr() == Ident("id")
        assert bw.get_low() == Value(0)
        assert bw.get_high() == Value(10)
        assert bw.get_negated() is False

        bw = BetweenExpr(
            Ident("id"), BinaryExpr(Ident("old"), "|", 0), 10, negated=True
        )
        assert bw.get_expr() == Ident("id")
        assert isinstance(bw.get_low(), BinaryExpr)
        assert bw.get_high() == Value(10)
        assert bw.get_negated() is True

    def test_to_sql(self):
        bw = BetweenExpr(
            Ident("id"), BinaryExpr(Ident("old"), "|", 0), 10, negated=True
        )
        assert bw.to_sql(PostgreSQL) == '"id" NOT BETWEEN "old" | 0 AND 10'

        bw = BetweenExpr(Ident("id"), 1, 10)
        assert bw.to_sql(PostgreSQL) == '"id" BETWEEN 1 AND 10'

        bw = BetweenExpr(Ident("id"), BinaryExpr(1, "AND", 2), 10)
        assert bw.to_sql(PostgreSQL) == '"id" BETWEEN (1 AND 2) AND 10'

        sql, params = BetweenExpr(Ident("id"), 1, 10).prepare(PostgreSQL)
        assert sql == '"id" BETWEEN :param_0 AND :param_1'
        assert params == {"param_0": 1, "param_1": 10}


class TestLikeExpr:
    def test_new(self):
        like = LikeExpr(Ident("name"), "%a%")
        assert like.get_expr() == Ident("name")
        assert like.get_pattern() == Value("%a%")
        assert like.get_case_sensitive() is True
        assert like.get_negated() is False
        assert like.get_escape() is None

        like = LikeExpr(
            Ident("name"),
            Ident("pattern"),
            case_sensitive=False,
            negated=True,
            escape="!",
        )
        assert like.get_expr() == Ident("name")
        assert like.get_pattern() == Ident("pattern")
        assert like.get_case_sensitive() is False
        assert like.get_negated() is True
        assert like.get_escape() == "!"

        any_like = LikeExpr(Ident("name"), ["%a%", "%b%"])
        assert any_like.get_pattern() == (Value("%a%"), Value("%b%"))

        one_any = LikeExpr(Ident("name"), ["%a%"])
        assert one_any.get_pattern() == (Value("%a%"),)

        tup = LikeExpr(Ident("name"), ("%a%", Ident("p")))
        assert tup.get_pattern() == (Value("%a%"), Ident("p"))

        with pytest.raises(ValueError, match="empty"):
            LikeExpr(Ident("name"), [])

        with pytest.raises((TypeError, ValueError)):
            LikeExpr(Ident("n"), "p", escape="ab")  # pyright: ignore

    def test_to_sql(self):
        like = LikeExpr(Ident("name"), "%a%")
        assert like.to_sql(PostgreSQL) == "\"name\" LIKE E'%a%'"

        assert (
            LikeExpr(Ident("name"), "%a%", case_sensitive=False).to_sql(PostgreSQL)
            == "\"name\" ILIKE E'%a%'"
        )
        assert (
            LikeExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSQL)
            == "\"name\" NOT LIKE E'%a%'"
        )
        assert (
            LikeExpr(Ident("name"), "%a%", escape="!").to_sql(PostgreSQL)
            == "\"name\" LIKE E'%a%' ESCAPE E'!'"
        )

        sql, params = LikeExpr(Ident("name"), "%a%").prepare(PostgreSQL)
        assert sql == '"name" LIKE :param_0'
        assert params == {"param_0": "%a%"}

        any_like = LikeExpr(Ident("name"), ["%a%", "%b%"])
        assert (
            any_like.to_sql(PostgreSQL)
            == "\"name\" LIKE ANY (ARRAY[E'%a%', E'%b%'])"
        )
        assert (
            any_like.to_sql(Snowflake)
            == "\"name\" LIKE ANY ('%a%', '%b%')"
        )
        assert any_like.to_sql(SQLite) == (
            "(\"name\" LIKE '%a%' OR \"name\" LIKE '%b%')"
        )

        sql, params = any_like.prepare(PostgreSQL)
        assert sql == '"name" LIKE ANY (ARRAY[:param_0, :param_1])'
        assert params == {"param_0": "%a%", "param_1": "%b%"}

        negated = LikeExpr(Ident("name"), ["%a%", "%b%"], negated=True)
        assert (
            negated.to_sql(PostgreSQL)
            == "NOT (\"name\" LIKE ANY (ARRAY[E'%a%', E'%b%']))"
        )
        assert (
            negated.to_sql(Snowflake)
            == "\"name\" NOT LIKE ANY ('%a%', '%b%')"
        )
        assert negated.to_sql(SQLite) == (
            "(\"name\" NOT LIKE '%a%' AND \"name\" NOT LIKE '%b%')"
        )

        ilike_any = LikeExpr(
            Ident("name"), ["%a%", "%b%"], case_sensitive=False
        )
        assert (
            ilike_any.to_sql(PostgreSQL)
            == "\"name\" ILIKE ANY (ARRAY[E'%a%', E'%b%'])"
        )

        escaped = LikeExpr(Ident("name"), ["%a%", "%b%"], escape="!")
        assert (
            escaped.to_sql(PostgreSQL)
            == "\"name\" LIKE ANY (ARRAY[E'%a%', E'%b%']) ESCAPE E'!'"
        )
        assert escaped.to_sql(SQLite) == (
            "(\"name\" LIKE '%a%' ESCAPE '!' OR \"name\" LIKE '%b%' ESCAPE '!')"
        )

        assert (
            repr(LikeExpr(Ident("name"), ["%a%", "%b%"], negated=True))
            == "LikeExpr(Ident(\"name\"), [Value('%a%', StringSerializer()), "
            "Value('%b%', StringSerializer())], negated=True)"
        )


class TestSimilarToExpr:
    def test_new(self):
        sim = SimilarToExpr(Ident("name"), "%a%")
        assert sim.get_expr() == Ident("name")
        assert sim.get_pattern() == Value("%a%")
        assert sim.get_negated() is False
        assert sim.get_escape() is None

        sim = SimilarToExpr(
            Ident("name"),
            Ident("pattern"),
            negated=True,
            escape="!",
        )
        assert sim.get_expr() == Ident("name")
        assert sim.get_pattern() == Ident("pattern")
        assert sim.get_negated() is True
        assert sim.get_escape() == "!"

        with pytest.raises((TypeError, ValueError)):
            SimilarToExpr(Ident("n"), "p", escape="ab")  # pyright: ignore

    def test_to_sql(self):
        sim = SimilarToExpr(Ident("name"), "%a%")
        assert sim.to_sql(PostgreSQL) == "\"name\" SIMILAR TO E'%a%'"

        assert (
            SimilarToExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSQL)
            == "\"name\" NOT SIMILAR TO E'%a%'"
        )
        assert (
            SimilarToExpr(Ident("name"), "%a%", escape="!").to_sql(PostgreSQL)
            == "\"name\" SIMILAR TO E'%a%' ESCAPE E'!'"
        )

        sql, params = SimilarToExpr(Ident("name"), "%a%").prepare(PostgreSQL)
        assert sql == '"name" SIMILAR TO :param_0'
        assert params == {"param_0": "%a%"}

        assert (
            repr(SimilarToExpr(Ident("name"), "%a%"))
            == "SimilarToExpr(Ident(\"name\"), Value('%a%', StringSerializer()))"
        )
        assert (
            repr(SimilarToExpr(Ident("name"), "%a%", negated=True, escape="!"))
            == "SimilarToExpr(Ident(\"name\"), Value('%a%', StringSerializer()), "
            "negated=True, escape='!')"
        )


class TestRLikeExpr:
    def test_new(self):
        rlike = RLikeExpr(Ident("name"), "%a%")
        assert rlike.get_expr() == Ident("name")
        assert rlike.get_pattern() == Value("%a%")
        assert rlike.get_negated() is False
        assert rlike.get_escape() is None

        rlike = RLikeExpr(
            Ident("name"),
            Ident("pattern"),
            negated=True,
            escape="!",
        )
        assert rlike.get_expr() == Ident("name")
        assert rlike.get_pattern() == Ident("pattern")
        assert rlike.get_negated() is True
        assert rlike.get_escape() == "!"

        with pytest.raises((TypeError, ValueError)):
            RLikeExpr(Ident("n"), "p", escape="ab")  # pyright: ignore

    def test_to_sql(self):
        rlike = RLikeExpr(Ident("name"), "%a%")
        assert rlike.to_sql(PostgreSQL) == "\"name\" RLIKE E'%a%'"

        assert (
            RLikeExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSQL)
            == "\"name\" NOT RLIKE E'%a%'"
        )
        assert (
            RLikeExpr(Ident("name"), "%a%", escape="!").to_sql(PostgreSQL)
            == "\"name\" RLIKE E'%a%' ESCAPE E'!'"
        )

        sql, params = RLikeExpr(Ident("name"), "%a%").prepare(PostgreSQL)
        assert sql == '"name" RLIKE :param_0'
        assert params == {"param_0": "%a%"}

        assert (
            repr(RLikeExpr(Ident("name"), "%a%"))
            == "RLikeExpr(Ident(\"name\"), Value('%a%', StringSerializer()))"
        )
        assert (
            repr(RLikeExpr(Ident("name"), "%a%", negated=True, escape="!"))
            == "RLikeExpr(Ident(\"name\"), Value('%a%', StringSerializer()), "
            "negated=True, escape='!')"
        )


class TestCastExpr:
    def test_new(self):
        cast = CastExpr(Ident("id"), "int")
        assert cast.get_expr() == Ident("id")
        assert cast.get_data_type() == "INT"

        cast = CastExpr(1, "varchar(255)")
        assert cast.get_expr() == Value(1)
        assert cast.get_data_type() == "varchar(255)"

        cast = CastExpr(Ident("x"), "some_totally_unknown_type")
        assert cast.get_data_type() == "some_totally_unknown_type"

    def test_to_sql(self):
        assert CastExpr(Ident("id"), "int").to_sql(PostgreSQL) == 'CAST("id" AS INT)'
        assert CastExpr(1, "varchar(255)").to_sql(PostgreSQL) == (
            "CAST(1 AS varchar(255))"
        )

        sql, params = CastExpr(1, "INT").prepare(PostgreSQL)
        assert sql == "CAST(:param_0 AS INT)"
        assert params == {"param_0": 1}


class TestCaseExpr:
    def test_new(self):
        empty = CaseExpr()
        assert empty.get_operand() is None
        assert empty.get_when_list() == ()
        assert empty.get_else() is None

        cond = BinaryExpr(Ident("id"), ">", 0)
        searched = empty.when(cond, "yes")
        assert searched is not empty
        assert empty.get_when_list() == ()
        when_list = searched.get_when_list()
        assert len(when_list) == 1
        assert isinstance(when_list[0][0], BinaryExpr)
        assert when_list[0][1] == Value("yes")
        assert searched.get_else() is None

        with_else = searched.else_("no")
        assert with_else is not searched
        assert searched.get_else() is None
        assert with_else.get_else() == Value("no")

        simple = CaseExpr(Ident("status"))
        assert simple.get_operand() == Ident("status")

        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(Ident("x"), 1)  # pyright: ignore

        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(Value(1), 1)  # pyright: ignore

        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(1, 2)  # pyright: ignore

    def test_to_sql(self):
        assert CaseExpr().to_sql(PostgreSQL) == "CASE END"

        searched = CaseExpr().when(BinaryExpr(Ident("id"), ">", 0), "yes").else_("no")
        assert (
            searched.to_sql(PostgreSQL)
            == "CASE WHEN \"id\" > 0 THEN E'yes' ELSE E'no' END"
        )

        simple = (
            CaseExpr(Ident("status"))
            .when(BinaryExpr(Ident("status"), "=", "open"), 1)
            .else_(0)
        )
        assert (
            simple.to_sql(PostgreSQL)
            == 'CASE "status" WHEN "status" = E\'open\' THEN 1 ELSE 0 END'
        )

        sql, params = searched.prepare(PostgreSQL)
        assert sql == 'CASE WHEN "id" > :param_0 THEN :param_1 ELSE :param_2 END'
        assert params == {"param_0": 0, "param_1": "yes", "param_2": "no"}


class TestCustomExpr:
    def test_new(self):
        custom = CustomExpr("1 = 1")
        assert custom.get_sql() == "1 = 1"
        assert custom.get_args() == ()

        custom = CustomExpr("{} = {}", Ident("a"), 1)
        assert custom.get_sql() == "{} = {}"
        assert custom.get_args() == (Ident("a"), Value(1))

    def test_to_sql(self):
        assert CustomExpr("1 = 1").to_sql(PostgreSQL) == "1 = 1"
        assert CustomExpr("{} = {}", Ident("a"), 1).to_sql(PostgreSQL) == '"a" = 1'
        assert CustomExpr("{{x}}").to_sql(PostgreSQL) == "{x}"
        assert CustomExpr("a {{}} b").to_sql(PostgreSQL) == "a {} b"
        assert CustomExpr("fn({})", Ident("id")).to_sql(PostgreSQL) == 'fn("id")'

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("a {} {}", 1).to_sql(PostgreSQL)

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("a {}", 1, 2).to_sql(PostgreSQL)

        sql, params = CustomExpr("x = {}", 1).prepare(PostgreSQL)
        assert sql == "x = :param_0"
        assert params == {"param_0": 1}

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("{} {}", 1).prepare(PostgreSQL)
