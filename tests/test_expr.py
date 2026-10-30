import pytest

from rapidquery.dialect import (
    BigQuery,
    Dialect,
    DuckDb,
    MsSql,
    MySql,
    Oracle,
    PostgreSql,
    Redshift,
    Snowflake,
    Spark,
    Sqlite,
    Teradata,
)
from rapidquery.expr import (
    Asterisk,
    BetweenExpr,
    BinaryExpr,
    CaseExpr,
    CastExpr,
    CollateExpr,
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
    TrimExpr,
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
            TrimExpr(Ident("s")),
            CollateExpr(Ident("s"), "en_US"),
            CaseExpr(),
            CustomExpr("1"),
        ]
        for node in nodes:
            assert isinstance(node, Expr)

    def test_render_succeeds_on_every_dialect(self, dialect: Dialect):
        expr = BinaryExpr(Ident("id"), ">", 10)
        sql = expr.to_sql(dialect)
        assert "id" in sql
        assert "10" in sql


def same(actual: object, expected: object) -> bool:
    """Structural check.

    ``==`` on an expression builds a SQL comparison and returns that node, so
    it is not a boolean. Compare ``repr`` instead. A tuple of nodes is compared
    element by element.
    """

    if isinstance(actual, tuple) and isinstance(expected, tuple):
        return len(actual) == len(expected) and all(
            same(left, right) for left, right in zip(actual, expected, strict=True)
        )
    if isinstance(actual, Expr) or isinstance(expected, Expr):
        return repr(actual) == repr(expected)
    return actual == expected


def _assert_binary(expr: Expr, operator: str, left: Expr, right: Expr) -> None:
    assert isinstance(expr, BinaryExpr)
    assert expr.get_operator() == operator
    assert same(expr.get_left(), left)
    assert same(expr.get_right(), right)


class TestExprMethods:
    """Fluent methods build the same nodes as the constructors.

    ``==`` and ``!=`` are SQL on every node, including ``Ident`` and ``Value``.
    Python ``&`` / ``|`` are ``AND`` / ``OR``.
    """

    def test_operators(self):
        name = Ident("name")
        one = Value(1)
        two = Value(2)
        text = Value("x")

        # A bare string is a literal. ``&`` / ``|`` are logical; bitwise forms
        # are named so ``~expr`` is not silently ``bit_not``.
        built = [
            (name + "x", "+", name, text),
            (name - 1, "-", name, one),
            (name * 2, "*", name, two),
            (name / 2, "/", name, two),
            (name % 2, "%", name, two),
            (name < 1, "<", name, one),
            (name <= 1, "<=", name, one),
            (name > 1, ">", name, one),
            (name >= 1, ">=", name, one),
            (name == 1, "=", name, one),
            (name != 1, "<>", name, one),
            (name << 1, "<<", name, one),
            (name >> 1, ">>", name, one),
            (name & 1, "AND", name, one),
            (name | 1, "OR", name, one),
            (name.bit_and(1), "&", name, one),
            (name.bit_or(1), "|", name, one),
            (name.bit_xor(1), "^", name, one),
            (name.concat("x"), "||", name, text),
            # Reflected: the other operand is on the left.
            (1 + name, "+", one, name),
            (1 - name, "-", one, name),
            (2 * name, "*", two, name),
            (2 / name, "/", two, name),
            (2 % name, "%", two, name),
            (1 << name, "<<", one, name),
            (1 >> name, ">>", one, name),
            (1 & name, "AND", one, name),
            (1 | name, "OR", one, name),
            (one + 2, "+", one, two),
            (one == 2, "=", one, two),
            (one != 2, "<>", one, two),
            # Python swaps comparisons. There is no reflected ``<``.
            (1 < name, ">", name, one),
        ]
        for expr, operator, left, right in built:
            _assert_binary(expr, operator, left, right)

        assert (+name).get_operator() == "+"
        assert same((+name).get_expr(), name)
        assert (-name).get_operator() == "-"
        assert name.not_().get_operator() == "NOT"
        assert name.bit_not().get_operator() == "~"
        with pytest.raises(TypeError):
            ~name  # pyright: ignore

        # ``*`` binds tighter than ``+``, so the sum is parenthesized.
        assert ((name + 1) * 2).to_sql(PostgreSql) == '("name" + 1) * 2'

    def test_is_accepts_none_bool_and_string(self):
        name = Ident("name")

        # ``None`` / ``True`` / ``False`` are ``IS`` tests, not ``= NULL``.
        null = name.is_(None)
        assert isinstance(null, IsExpr)
        assert same(null.get_expr(), name)
        assert null.get_kind() == "NULL"
        assert null.get_negated() is False
        assert name.is_(True).get_kind() == "TRUE"
        assert name.is_(False).get_kind() == "FALSE"
        assert name.is_("true").get_kind() == "TRUE"
        assert name.is_("unknown").get_kind() == "UNKNOWN"

        negated = name.is_not(False)
        assert negated.get_kind() == "FALSE"
        assert negated.get_negated() is True
        assert name.is_not(None).to_sql(PostgreSql) == '"name" IS NOT NULL'
        assert name.is_(True).to_sql(PostgreSql) == '"name" IS TRUE'

        with pytest.raises(TypeError):
            name.is_(1)

        with pytest.raises(ValueError, match="unknown IS kind"):
            name.is_("MAYBE")

    def test_predicates(self):
        name = Ident("name")

        between = name.between(1, 3)
        assert isinstance(between, BetweenExpr)
        assert same(between.get_low(), Value(1))
        assert same(between.get_high(), Value(3))
        assert between.get_negated() is False
        assert name.not_between(1, 3).get_negated() is True

        inside = name.in_([1, name])
        assert isinstance(inside, InListExpr)
        assert same(inside.get_list(), (Value(1), name))
        assert name.not_in([1]).get_negated() is True

        distinct = name.is_distinct_from(1)
        assert isinstance(distinct, IsDistinctFromExpr)
        assert same(distinct.get_right(), Value(1))
        assert distinct.get_negated() is False
        assert name.is_not_distinct_from(1).get_negated() is True

        like = name.like("a%", escape="!")
        assert isinstance(like, LikeExpr)
        assert same(like.get_pattern(), Value("a%"))
        assert like.get_escape() == "!"
        assert like.get_negated() is False
        assert like.get_case_sensitive() is True
        assert name.not_like("a%").get_negated() is True
        assert name.ilike("a%").get_case_sensitive() is False
        assert same(name.not_ilike(["a%", "b%"]).get_pattern(), (Value("a%"), Value("b%")))
        # PyO3 extracts ``escape`` as one character before the node is built.
        with pytest.raises(ValueError, match="length 1"):
            name.like("a", escape="ab")  # pyright: ignore

        similar = name.similar_to("a%")
        assert isinstance(similar, SimilarToExpr)
        assert same(similar.get_pattern(), Value("a%"))
        assert name.not_similar_to("a%").get_negated() is True

        regex = name.rlike("a.*")
        assert isinstance(regex, RLikeExpr)
        assert same(regex.get_pattern(), Value("a.*"))
        assert name.not_rlike("a.*").get_negated() is True

        # One render per family. Dialect spellings stay on the constructor tests.
        assert between.to_sql(PostgreSql) == '("name" BETWEEN 1 AND 3)'
        assert like.to_sql(PostgreSql) == "\"name\" LIKE E'a%' ESCAPE E'!'"
        assert similar.to_sql(PostgreSql) == "\"name\" SIMILAR TO E'a%'"
        assert regex.to_sql(PostgreSql) == "\"name\" ~ E'a.*'"

    def test_cast_collate_and_trim(self):
        name = Ident("name")

        cast = name.cast_as("int")
        assert isinstance(cast, CastExpr)
        assert cast.get_kind() == "CAST"
        assert cast.get_data_type_name() == "INT"
        assert cast.to_sql(PostgreSql) == 'CAST("name" AS INT)'

        attempt = name.try_cast_as("int")
        assert attempt.get_kind() == "TRY_CAST"
        assert attempt.to_sql(BigQuery) == "SAFE_CAST(`name` AS INT)"

        collated = name.collate(Unquote("C"))
        assert isinstance(collated, CollateExpr)
        assert same(collated.get_collation(), Ident(Unquote("C")))
        assert name.collate("en_US").to_sql(PostgreSql) == '"name" COLLATE "en_US"'

        trimmed = name.trim()
        assert isinstance(trimmed, TrimExpr)
        assert trimmed.get_where() == "BOTH"
        assert trimmed.get_characters() is None
        assert trimmed.to_sql(PostgreSql) == 'TRIM("name")'

        leading = name.trim(where="leading", characters="x")
        assert leading.get_where() == "LEADING"
        assert same(leading.get_characters(), Value("x"))
        with pytest.raises(ValueError, match="unknown TRIM where"):
            name.trim(where="NOPE")


class TestFunction:
    def test_new(self):
        fn = Function("now")
        assert same(fn.get_name(), Ident("now"))
        assert fn.get_distinct() is False
        assert fn.get_args() == ()
        assert fn.get_filter() is None
        assert fn.get_null_treatment() is None

        fn = Function("example").filter(BinaryExpr(1, ">", 2))
        assert same(fn.get_name(), Ident("example"))
        assert fn.get_distinct() is False
        assert fn.get_args() == ()
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = Function("count", Ident("id"), distinct=False).filter(BinaryExpr(1, ">", 2))
        assert same(fn.get_name(), Ident("count"))
        assert fn.get_distinct() is False
        assert same(fn.get_args(), (Ident("id"),))
        assert isinstance(fn.get_filter(), BinaryExpr)

        fn = Function(Unquote("NOW"))
        assert same(fn.get_name(), Ident(Unquote("NOW")))

        fn = Function("schema.fn")
        assert same(fn.get_name(), Ident("schema", "fn"))

        bare = Function("now")
        with_arg = Function("now", 1)
        assert with_arg is not bare
        assert same(with_arg.get_args(), (Value(1),))
        assert bare.get_args() == ()

    def test_to_sql(self):
        assert Function("current_timestamp").to_sql(PostgreSql) == '"current_timestamp"()'
        assert Function(Unquote("CURRENT_TIMESTAMP")).to_sql(PostgreSql) == "CURRENT_TIMESTAMP()"
        assert Function(Unquote("NOW")).to_sql(PostgreSql) == "NOW()"

        fn = Function("count", Asterisk).filter(BinaryExpr(Ident("status"), "=", "completed"))
        assert fn.to_sql(PostgreSql) == '"count"(*) FILTER (WHERE "status" = E\'completed\')'

        fn = Function("count", Ident("id"), distinct=False).filter(BinaryExpr(Ident("status"), "=", "completed"))
        assert fn.to_sql(PostgreSql) == '"count"("id") FILTER (WHERE "status" = E\'completed\')'

        fn = Function("count", Ident("id"), distinct=True).filter(BinaryExpr(Ident("status"), "=", "completed"))
        assert fn.to_sql(PostgreSql) == '"count"(DISTINCT "id") FILTER (WHERE "status" = E\'completed\')'

        assert Function("now", distinct=True).to_sql(PostgreSql) == '"now"()'
        assert Function("custom", 1, 2, Ident("id"), distinct=True).to_sql(PostgreSql) == (
            '"custom"(DISTINCT 1, 2, "id")'
        )

        fn = Function("lag", Ident("x")).ignore_nulls()
        assert fn.to_sql(Oracle) == '"lag"("x") IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(`x` IGNORE NULLS)"
        assert fn.to_sql(DuckDb) == '"lag"("x" IGNORE NULLS)'
        assert fn.get_null_treatment() == "IGNORE NULLS"
        with pytest.raises(RuntimeError, match="null_treatment"):
            fn.to_sql(PostgreSql)

        fn = Function("lag", Ident("x")).respect_nulls()
        assert fn.to_sql(Oracle) == '"lag"("x") RESPECT NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(`x` RESPECT NULLS)"
        assert fn.to_sql(DuckDb) == '"lag"("x" RESPECT NULLS)'
        assert fn.get_null_treatment() == "RESPECT NULLS"

        fn = Function("lag").ignore_nulls()
        assert fn.to_sql(Oracle) == '"lag"() IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`lag`(IGNORE NULLS)"

        completed = BinaryExpr(Ident("status"), "=", "completed")
        fn = Function("count", Ident("id")).ignore_nulls().filter(completed)
        assert fn.to_sql(Oracle) == '"count"("id") FILTER (WHERE "status" = \'completed\') IGNORE NULLS'
        assert fn.to_sql(BigQuery) == "`count`(`id` IGNORE NULLS WHERE `status` = 'completed')"
        assert fn.to_sql(DuckDb) == '"count"("id" IGNORE NULLS) FILTER ("status" = \'completed\')'

    def test_edge_cases(self):
        with pytest.raises(TypeError):
            Function(1)  # pyright: ignore

        with pytest.raises(ValueError, match="empty"):
            Function("")

        Function("now").filter(Ident("x"))
        Function("now").filter(1)

        fn = Function("f", "id")
        assert same(fn.get_args(), (Value("id"),))
        assert fn.to_sql(PostgreSql) == "\"f\"(E'id')"

        fn = Function(Ident("schema.fn"))
        assert same(fn.get_name(), Ident("schema.fn"))
        assert fn.to_sql(PostgreSql) == '"schema.fn"()'

        first = Function("f", 1)
        second = Function("f", 2, 3)
        assert same(first.get_args(), (Value(1),))
        assert same(second.get_args(), (Value(2), Value(3)))

        filtered = Function("f").filter(BinaryExpr(1, ">", 0))
        replaced = filtered.filter(BinaryExpr(2, ">", 0))
        assert filtered.to_sql(PostgreSql) == '"f"() FILTER (WHERE 1 > 0)'
        assert replaced.to_sql(PostgreSql) == '"f"() FILTER (WHERE 2 > 0)'

        ignored = Function("lag", Ident("x")).ignore_nulls()
        respected = ignored.respect_nulls()
        assert ignored.get_null_treatment() == "IGNORE NULLS"
        assert respected.get_null_treatment() == "RESPECT NULLS"
        assert ignored.to_sql(Oracle) == '"lag"("x") IGNORE NULLS'
        assert respected.to_sql(Oracle) == '"lag"("x") RESPECT NULLS'

        fn = Function("count", 1).filter(BinaryExpr(Ident("x"), ">", 2))
        sql, params = fn.prepare(PostgreSql)
        assert sql == '"count"($1) FILTER (WHERE "x" > $2)'
        assert params == (1, 2)

        assert repr(Function("lag", Ident("x")).ignore_nulls()) == ('Function(Ident("lag"), Ident("x")).ignore_nulls()')


class TestTupleExpr:
    def test_new(self):
        tpl = TupleExpr(1, 2, "A")
        assert same(tpl.get_items(), (Value(1), Value(2), Value("A")))

        tpl = TupleExpr(1, 2, Ident("A"))
        assert same(tpl.get_items(), (Value(1), Value(2), Ident("A")))

        empty = TupleExpr()
        assert empty.get_items() == ()

        nested = TupleExpr((1, 2), Ident("A"))
        assert isinstance(nested.get_items()[0], TupleExpr)
        assert same(nested.get_items()[0].get_items(), (Value(1), Value(2)))
        assert same(nested.get_items()[1], Ident("A"))

    def test_to_sql(self):
        tpl = TupleExpr(1, 2, "A")
        assert tpl.to_sql(PostgreSql) == "ROW(1, 2, E'A')"

        sql, params = tpl.prepare(PostgreSql)
        assert sql == "ROW($1, $2, $3)"
        assert params == (1, 2, "A")

        assert TupleExpr().to_sql(PostgreSql) == "ROW()"
        assert TupleExpr(1).to_sql(PostgreSql) == "ROW(1)"
        assert TupleExpr((1, 2), Ident("A")).to_sql(PostgreSql) == 'ROW(ROW(1, 2), "A")'

        assert TupleExpr(1, 2).to_sql(Sqlite) == "(1, 2)"

        with pytest.raises(RuntimeError, match="tuple with one item"):
            TupleExpr(1).to_sql(Sqlite)

        assert TupleExpr(1, 2).to_sql(Spark) == "struct(1, 2)"

        with pytest.raises(RuntimeError, match="tuple"):
            TupleExpr(1, 2).to_sql(BigQuery)


class TestUnaryExpr:
    def test_new(self):
        un = UnaryExpr("NOT", 1)
        assert same(un.get_expr(), Value(1))
        assert un.get_operator() == "NOT"

        un = UnaryExpr("Not", 1)
        assert un.get_operator() == "NOT"

        un = UnaryExpr("-", 1)
        assert un.get_operator() == "-"

        un = UnaryExpr("+", Ident("id"))
        assert un.get_operator() == "+"
        assert same(un.get_expr(), Ident("id"))

        un = UnaryExpr("~", 1)
        assert un.get_operator() == "~"

        for token in (
            "@-@",
            "@@",
            "#",
            "@",
            "||/",
            "!",
            "!!",
            "|/",
            "?-",
            "?|",
            "bang",
        ):
            with pytest.raises(ValueError, match="unknown unary operator"):
                UnaryExpr(token, 1)

    def test_to_sql(self):
        un = UnaryExpr("-", Function("now"))
        assert un.to_sql(PostgreSql) == '-"now"()'

        assert UnaryExpr("NOT", 1).to_sql(PostgreSql) == "NOT 1"
        assert UnaryExpr("NOT", Ident("id")).to_sql(PostgreSql) == 'NOT "id"'
        assert UnaryExpr("+", 1).to_sql(PostgreSql) == "+1"
        assert UnaryExpr("~", Ident("id")).to_sql(PostgreSql) == '~"id"'
        assert UnaryExpr("-", UnaryExpr("-", 1)).to_sql(PostgreSql) == "-(-1)"
        assert UnaryExpr("NOT", UnaryExpr("NOT", 1)).to_sql(PostgreSql) == "NOT (NOT 1)"
        assert UnaryExpr("-", BinaryExpr(1, "+", 2)).to_sql(PostgreSql) == "-(1 + 2)"

        sql, params = UnaryExpr("-", 1).prepare(PostgreSql)
        assert sql == "-$1"
        assert params == (1,)


class TestBinaryExpr:
    def test_new(self):
        bin = BinaryExpr(Ident("id"), ">", 10)
        assert same(bin.get_left(), Ident("id"))
        assert bin.get_operator() == ">"
        assert same(bin.get_right(), Value(10))

        bin = BinaryExpr("str", "-custom-", 10)
        assert same(bin.get_left(), Value("str"))
        assert bin.get_operator() == "-custom-"
        assert same(bin.get_right(), Value(10))

        bin = BinaryExpr(1, "!=", 2)
        assert bin.get_operator() == "<>"

        bin = BinaryExpr(1, "and", 2)
        assert bin.get_operator() == "AND"
        bin = BinaryExpr(1, "Or", 2)
        assert bin.get_operator() == "OR"
        bin = BinaryExpr(1, "xor", 2)
        assert bin.get_operator() == "xor"

    def test_to_sql(self):
        bin = BinaryExpr(Ident("id"), ">", 10)
        assert bin.to_sql(PostgreSql) == '"id" > 10'

        bin = BinaryExpr("str", "-custom-", 10)
        assert bin.to_sql(PostgreSql) == "E'str' -custom- 10"

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "&", 10))
        assert bin.to_sql(PostgreSql) == '"id" > 456 & 10'

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "CusTom", 10))
        assert bin.to_sql(PostgreSql) == '"id" > (456 CusTom 10)'

        bin = BinaryExpr(Ident("id"), ">", BinaryExpr(456, "<", 10))
        assert bin.to_sql(PostgreSql) == '"id" > (456 < 10)'

        assert BinaryExpr(1, "!=", 2).to_sql(PostgreSql) == "1 <> 2"
        assert BinaryExpr(1, "AND", 2).to_sql(PostgreSql) == "1 AND 2"
        assert BinaryExpr(BinaryExpr(1, "+", 2), "AND", 3).to_sql(PostgreSql) == "1 + 2 AND 3"
        assert BinaryExpr(1, "+", BinaryExpr(2, "AND", 3)).to_sql(PostgreSql) == "1 + (2 AND 3)"

        sql, params = BinaryExpr(Ident("id"), ">", 10).prepare(PostgreSql)
        assert sql == '"id" > $1'
        assert params == (10,)

        concat = BinaryExpr(1, "||", BinaryExpr(2, "+", 3))
        assert concat.to_sql(PostgreSql) == "1 || 2 + 3"
        assert concat.to_sql(Sqlite) == "1 || 2 + 3"
        assert concat.to_sql(MsSql) == "1 + 2 + 3"
        assert concat.to_sql(MySql) == "CONCAT(1, 2 + 3)"
        assert BinaryExpr(BinaryExpr(1, "||", 2), "||", 3).to_sql(MySql) == "CONCAT(CONCAT(1, 2), 3)"

        assert BinaryExpr(1, "^", 2).to_sql(PostgreSql) == "1 # 2"
        assert BinaryExpr(1, "^", 2).to_sql(DuckDb) == "xor(1, 2)"
        assert BinaryExpr(1, "^", 2).to_sql(Snowflake) == "BITXOR(1, 2)"
        assert BinaryExpr(1, "^", 2).to_sql(MySql) == "1 ^ 2"

        with pytest.raises(RuntimeError, match=r"BinaryExpr '\^'"):
            BinaryExpr(1, "^", 2).to_sql(Sqlite)

        assert BinaryExpr(5, "%", 2).to_sql(Oracle) == "MOD(5, 2)"
        assert BinaryExpr(5, "%", 2).to_sql(Teradata) == "5 MOD 2"
        assert BinaryExpr(5, "%", 2).to_sql(PostgreSql) == "5 % 2"

    def test_repr(self):
        assert repr(BinaryExpr(Ident("id"), ">", 10)) == 'BinaryExpr(Ident("id"), ">", Value(10, IntegerSerializer()))'


class TestIsExpr:
    def test_new(self):
        is_ = IsExpr(Ident("id"), "null")
        assert same(is_.get_expr(), Ident("id"))
        assert is_.get_kind() == "NULL"
        assert is_.get_negated() is False

        is_ = IsExpr(None, "UNKNOWN", negated=True)
        assert same(is_.get_expr(), Value(None))
        assert is_.get_kind() == "UNKNOWN"
        assert is_.get_negated() is True

        with pytest.raises(ValueError, match="unknown IS kind"):
            IsExpr(Ident("id"), "MAYBE")

    def test_to_sql(self):
        is_ = IsExpr(Ident("id"), "NUll")
        assert is_.to_sql(PostgreSql) == '"id" IS NULL'

        is_ = IsExpr(Ident("id"), "FalSE")
        assert is_.to_sql(PostgreSql) == '"id" IS FALSE'

        is_ = IsExpr(Ident("id"), "tRuE")
        assert is_.to_sql(PostgreSql) == '"id" IS TRUE'

        is_ = IsExpr(Ident("id"), "unknown", negated=True)
        assert is_.to_sql(PostgreSql) == '"id" IS NOT UNKNOWN'

        sql, params = IsExpr(Ident("id"), "NULL").prepare(PostgreSql)
        assert sql == '"id" IS NULL'
        assert params == ()

        sql, params = IsExpr(1, "TRUE").prepare(PostgreSql)
        assert sql == "$1 IS TRUE"
        assert params == (1,)

    def test_repr(self):
        assert repr(IsExpr(Ident("id"), "NULL")) == 'IsExpr(Ident("id"), "NULL")'
        assert repr(IsExpr(Ident("id"), "NULL", negated=True)) == 'IsExpr(Ident("id"), "NULL", negated=True)'


class TestIsDistinctFromExpr:
    def test_new(self):
        idf = IsDistinctFromExpr(2, 3)
        assert same(idf.get_left(), Value(2))
        assert same(idf.get_right(), Value(3))
        assert idf.get_negated() is False

        idf = IsDistinctFromExpr("A", "B")
        assert same(idf.get_left(), Value("A"))
        assert same(idf.get_right(), Value("B"))
        assert idf.get_negated() is False

        idf = IsDistinctFromExpr(Ident("id"), 10, negated=True)
        assert same(idf.get_left(), Ident("id"))
        assert same(idf.get_right(), Value(10))
        assert idf.get_negated() is True

        idf = IsDistinctFromExpr(Ident("a"), Ident("b"))
        assert same(idf.get_left(), Ident("a"))
        assert same(idf.get_right(), Ident("b"))

    def test_to_sql(self):
        idf = IsDistinctFromExpr(2, 3)
        assert idf.to_sql(PostgreSql) == "2 IS DISTINCT FROM 3"

        idf = IsDistinctFromExpr(Ident("id"), 10, negated=True)
        assert idf.to_sql(PostgreSql) == '"id" IS NOT DISTINCT FROM 10'

        sql, params = idf.prepare(PostgreSql)
        assert sql == '"id" IS NOT DISTINCT FROM $1'
        assert params == (10,)

        assert IsDistinctFromExpr(Ident("a"), Ident("b")).to_sql(PostgreSql) == '"a" IS DISTINCT FROM "b"'

        assert IsDistinctFromExpr(1, 2).to_sql(MySql) == "NOT (1 <=> 2)"
        assert IsDistinctFromExpr(1, 2, negated=True).to_sql(MySql) == "1 <=> 2"
        with pytest.raises(RuntimeError, match="IsDistinctFromExpr"):
            IsDistinctFromExpr(1, 2).to_sql(Oracle)


class TestInListExpr:
    def test_new(self):
        lst = InListExpr(Ident("id"), [1, 2, 3])
        assert same(lst.get_expr(), Ident("id"))
        assert same(lst.get_list(), (Value(1), Value(2), Value(3)))
        assert lst.get_negated() is False

        lst = InListExpr("A", [1, Ident("id"), 3], negated=True)
        assert same(lst.get_expr(), Value("A"))
        assert same(lst.get_list(), (Value(1), Ident("id"), Value(3)))
        assert lst.get_negated() is True

        lst = InListExpr(Ident("id"), [])
        assert lst.get_list() == ()

        with pytest.raises(TypeError):
            InListExpr(Ident("id"), 1)  # pyright: ignore

    def test_to_sql(self):
        lst = InListExpr(Ident("id"), [1, 2, 3])
        assert lst.to_sql(PostgreSql) == '("id" IN (1, 2, 3))'

        sql, params = lst.prepare(PostgreSql)
        assert sql == '("id" IN ($1, $2, $3))'
        assert params == (1, 2, 3)

        assert InListExpr(Ident("id"), []).to_sql(PostgreSql) == "FALSE"
        assert InListExpr(Ident("id"), [], negated=True).to_sql(PostgreSql) == "TRUE"
        assert InListExpr(Ident("id"), [], negated=True).to_sql(MySql) == "1"
        assert InListExpr(Ident("id"), [1], negated=True).to_sql(PostgreSql) == ('("id" NOT IN (1))')
        assert InListExpr(Ident("id"), [1, 2]).to_sql(MsSql) == "([id] IN (1, 2))"
        assert (
            BinaryExpr(
                InListExpr(Ident("id"), [1]),
                "OR",
                InListExpr(Ident("id"), [2]),
            ).to_sql(MsSql)
            == "([id] IN (1)) OR ([id] IN (2))"
        )


class TestBetweenExpr:
    def test_new(self):
        bw = BetweenExpr(Ident("id"), 0, 10)
        assert same(bw.get_expr(), Ident("id"))
        assert same(bw.get_low(), Value(0))
        assert same(bw.get_high(), Value(10))
        assert bw.get_negated() is False

        bw = BetweenExpr(Ident("id"), BinaryExpr(Ident("old"), "|", 0), 10, negated=True)
        assert same(bw.get_expr(), Ident("id"))
        assert isinstance(bw.get_low(), BinaryExpr)
        assert same(bw.get_high(), Value(10))
        assert bw.get_negated() is True

    def test_to_sql(self):
        bw = BetweenExpr(Ident("id"), BinaryExpr(Ident("old"), "|", 0), 10, negated=True)
        assert bw.to_sql(PostgreSql) == '("id" NOT BETWEEN "old" | 0 AND 10)'

        bw = BetweenExpr(Ident("id"), 1, 10)
        assert BetweenExpr(Ident("id"), 1, 10, negated=True).to_sql(MsSql) == ("([id] NOT BETWEEN 1 AND 10)")
        assert bw.to_sql(PostgreSql) == '("id" BETWEEN 1 AND 10)'

        bw = BetweenExpr(Ident("id"), BinaryExpr(1, "AND", 2), 10)
        assert bw.to_sql(PostgreSql) == '("id" BETWEEN (1 AND 2) AND 10)'

        sql, params = BetweenExpr(Ident("id"), 1, 10).prepare(PostgreSql)
        assert sql == '("id" BETWEEN $1 AND $2)'
        assert params == (1, 10)


class TestLikeExpr:
    def test_new(self):
        like = LikeExpr(Ident("name"), "%a%")
        assert same(like.get_expr(), Ident("name"))
        assert same(like.get_pattern(), Value("%a%"))
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
        assert same(like.get_expr(), Ident("name"))
        assert same(like.get_pattern(), Ident("pattern"))
        assert like.get_case_sensitive() is False
        assert like.get_negated() is True
        assert like.get_escape() == "!"

        any_like = LikeExpr(Ident("name"), ["%a%", "%b%"])
        assert same(any_like.get_pattern(), (Value("%a%"), Value("%b%")))

        one_any = LikeExpr(Ident("name"), ["%a%"])
        assert same(one_any.get_pattern(), (Value("%a%"),))

        tup = LikeExpr(Ident("name"), ("%a%", Ident("p")))
        assert same(tup.get_pattern(), (Value("%a%"), Ident("p")))

        with pytest.raises(ValueError, match="empty"):
            LikeExpr(Ident("name"), [])

        with pytest.raises((TypeError, ValueError)):
            LikeExpr(Ident("n"), "p", escape="ab")  # pyright: ignore

    def test_to_sql(self):
        like = LikeExpr(Ident("name"), "%a%")
        assert like.to_sql(PostgreSql) == "\"name\" LIKE E'%a%'"

        assert LikeExpr(Ident("name"), "%a%", case_sensitive=False).to_sql(PostgreSql) == "\"name\" ILIKE E'%a%'"
        assert LikeExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSql) == "\"name\" NOT LIKE E'%a%'"
        assert LikeExpr(Ident("name"), "%a%", escape="!").to_sql(PostgreSql) == "\"name\" LIKE E'%a%' ESCAPE E'!'"

        sql, params = LikeExpr(Ident("name"), "%a%").prepare(PostgreSql)
        assert sql == '"name" LIKE $1'
        assert params == ("%a%",)

        any_like = LikeExpr(Ident("name"), ["%a%", "%b%"])
        assert any_like.to_sql(PostgreSql) == "\"name\" LIKE ANY (ARRAY[E'%a%', E'%b%'])"
        assert any_like.to_sql(Snowflake) == "\"name\" LIKE ANY ('%a%', '%b%')"
        assert any_like.to_sql(Sqlite) == "(\"name\" LIKE '%a%' OR \"name\" LIKE '%b%')"

        sql, params = any_like.prepare(PostgreSql)
        assert sql == '"name" LIKE ANY (ARRAY[$1, $2])'
        assert params == ("%a%", "%b%")

        negated = LikeExpr(Ident("name"), ["%a%", "%b%"], negated=True)
        assert negated.to_sql(PostgreSql) == "NOT (\"name\" LIKE ANY (ARRAY[E'%a%', E'%b%']))"
        assert negated.to_sql(Snowflake) == "NOT (\"name\" LIKE ANY ('%a%', '%b%'))"
        assert negated.to_sql(Sqlite) == "(\"name\" NOT LIKE '%a%' AND \"name\" NOT LIKE '%b%')"

        ilike_any = LikeExpr(Ident("name"), ["%a%", "%b%"], case_sensitive=False)
        assert ilike_any.to_sql(PostgreSql) == "\"name\" ILIKE ANY (ARRAY[E'%a%', E'%b%'])"

        with pytest.raises(RuntimeError, match="LIKE ANY ESCAPE"):
            LikeExpr(Ident("name"), ["%a%", "%b%"], escape="!").to_sql(PostgreSql)
        assert LikeExpr(Ident("name"), ["%a%", "%b%"], escape="!").to_sql(Snowflake) == (
            "\"name\" LIKE ANY ('%a%', '%b%') ESCAPE '!'"
        )
        assert LikeExpr(Ident("name"), ["%a%", "%b%"], escape="!").to_sql(Sqlite) == (
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
        assert same(sim.get_expr(), Ident("name"))
        assert same(sim.get_pattern(), Value("%a%"))
        assert sim.get_negated() is False

        sim = SimilarToExpr(Ident("name"), Ident("pattern"), negated=True)
        assert same(sim.get_expr(), Ident("name"))
        assert same(sim.get_pattern(), Ident("pattern"))
        assert sim.get_negated() is True

    def test_to_sql(self):
        sim = SimilarToExpr(Ident("name"), "%a%")
        assert sim.to_sql(PostgreSql) == "\"name\" SIMILAR TO E'%a%'"
        assert sim.to_sql(Redshift) == "\"name\" SIMILAR TO '%a%'"
        assert sim.to_sql(DuckDb) == "\"name\" SIMILAR TO '%a%'"

        assert SimilarToExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSql) == (
            "\"name\" NOT SIMILAR TO E'%a%'"
        )

        sql, params = sim.prepare(PostgreSql)
        assert sql == '"name" SIMILAR TO $1'
        assert params == ("%a%",)

        with pytest.raises(RuntimeError, match="SimilarToExpr"):
            sim.to_sql(MySql)

        with pytest.raises(RuntimeError, match="SimilarToExpr"):
            sim.to_sql(Sqlite)

        assert repr(sim) == "SimilarToExpr(Ident(\"name\"), Value('%a%', StringSerializer()))"
        assert (
            repr(SimilarToExpr(Ident("name"), "%a%", negated=True))
            == "SimilarToExpr(Ident(\"name\"), Value('%a%', StringSerializer()), "
            "negated=True)"
        )


class TestRLikeExpr:
    def test_new(self):
        rlike = RLikeExpr(Ident("name"), "%a%")
        assert same(rlike.get_expr(), Ident("name"))
        assert same(rlike.get_pattern(), Value("%a%"))
        assert rlike.get_negated() is False

        rlike = RLikeExpr(Ident("name"), Ident("pattern"), negated=True)
        assert same(rlike.get_expr(), Ident("name"))
        assert same(rlike.get_pattern(), Ident("pattern"))
        assert rlike.get_negated() is True

    def test_to_sql(self):
        rlike = RLikeExpr(Ident("name"), "%a%")
        assert rlike.to_sql(PostgreSql) == "\"name\" ~ E'%a%'"
        assert rlike.to_sql(MySql) == "`name` RLIKE '%a%'"
        assert rlike.to_sql(Spark) == "`name` RLIKE '%a%'"
        assert rlike.to_sql(Snowflake) == "\"name\" RLIKE '%a%'"
        assert rlike.to_sql(Oracle) == "REGEXP_LIKE(\"name\", '%a%')"
        assert rlike.to_sql(BigQuery) == "REGEXP_CONTAINS(`name`, '%a%')"
        assert rlike.to_sql(DuckDb) == "regexp_matches(\"name\", '%a%')"

        assert RLikeExpr(Ident("name"), "%a%", negated=True).to_sql(PostgreSql) == "\"name\" !~ E'%a%'"
        assert RLikeExpr(Ident("name"), "%a%", negated=True).to_sql(Oracle) == "NOT REGEXP_LIKE(\"name\", '%a%')"

        sql, params = rlike.prepare(PostgreSql)
        assert sql == '"name" ~ $1'
        assert params == ("%a%",)

        with pytest.raises(RuntimeError, match="RLikeExpr"):
            rlike.to_sql(Sqlite)

        with pytest.raises(RuntimeError, match="RLikeExpr"):
            rlike.to_sql(MsSql)

        assert repr(rlike) == "RLikeExpr(Ident(\"name\"), Value('%a%', StringSerializer()))"
        assert (
            repr(RLikeExpr(Ident("name"), "%a%", negated=True))
            == "RLikeExpr(Ident(\"name\"), Value('%a%', StringSerializer()), "
            "negated=True)"
        )


class TestCastExpr:
    def test_new(self):
        cast = CastExpr(Ident("id"), "int")
        assert same(cast.get_expr(), Ident("id"))
        assert cast.get_data_type_name() == "INT"
        assert cast.get_kind() == "CAST"

        cast = CastExpr(1, "varchar(255)", kind="TRY_CAST")
        assert same(cast.get_expr(), Value(1))
        assert cast.get_data_type_name() == "varchar(255)"
        assert cast.get_kind() == "TRY_CAST"

        cast = CastExpr(Ident("x"), "some_totally_unknown_type", kind="cAsT")
        assert cast.get_data_type_name() == "some_totally_unknown_type"
        assert cast.get_kind() == "CAST"

        cast = CastExpr(Ident("x"), "integer", kind="TRY_CAST")
        assert cast.get_data_type_name() == "INTEGER"
        assert cast.get_kind() == "TRY_CAST"

        cast = CastExpr(Ident("x"), "integer", kind="::")
        assert cast.get_kind() == "::"

        with pytest.raises(ValueError, match="unknown cast kind"):
            CastExpr(Ident("x"), "integer", kind="SAFE_CAST")

    def test_to_sql(self):
        assert CastExpr(Ident("id"), "int").to_sql(PostgreSql) == 'CAST("id" AS INT)'
        assert CastExpr(1, "varchar(255)").to_sql(PostgreSql) == "CAST(1 AS varchar(255))"
        assert CastExpr(Ident("id"), "INT ARRAY").to_sql(PostgreSql) == 'CAST("id" AS INT ARRAY)'

        with pytest.raises(RuntimeError, match="'TRY_CAST'"):
            CastExpr(Ident("id"), "int", kind="TRY_CAST").to_sql(PostgreSql)

        assert CastExpr(Ident("id"), "int", kind="TRY_CAST").to_sql(BigQuery) == "SAFE_CAST(`id` AS INT)"
        assert CastExpr(Ident("id"), "int", kind="TRY_CAST").to_sql(Teradata) == 'TRYCAST("id" AS INT)'
        assert CastExpr(Ident("id"), "int", kind="::").to_sql(DuckDb) == '("id")::INT'

        sql, params = CastExpr(1, "INT").prepare(PostgreSql)
        assert sql == "CAST($1 AS INT)"
        assert params == (1,)

        assert repr(CastExpr(Ident("id"), "int")) == 'CastExpr(Ident("id"), "INT")'
        assert repr(CastExpr(Ident("id"), "int", kind="TRY_CAST")) == 'CastExpr(Ident("id"), "INT", kind="TRY_CAST")'


class TestTrimExpr:
    def test_new(self):
        trim = TrimExpr(Ident("s"))
        assert same(trim.get_expr(), Ident("s"))
        assert trim.get_where() == "BOTH"
        assert trim.get_characters() is None

        trim = TrimExpr(Ident("s"), where="leading", characters="x")
        assert trim.get_where() == "LEADING"
        assert same(trim.get_characters(), Value("x"))

        trim = TrimExpr(Ident("s"), characters=Ident("b"))
        assert same(trim.get_characters(), Ident("b"))

        with pytest.raises(ValueError, match="unknown TRIM where"):
            TrimExpr(Ident("s"), where="MIDDLE")
        with pytest.raises(TypeError):
            TrimExpr(Ident("s"), what="x")  # pyright: ignore

    def test_to_sql(self):
        assert TrimExpr(Ident("s")).to_sql(PostgreSql) == 'TRIM("s")'
        assert TrimExpr(Ident("s"), where="LEADING").to_sql(PostgreSql) == 'TRIM(LEADING FROM "s")'
        assert TrimExpr(Ident("s"), characters="x").to_sql(PostgreSql) == "TRIM(E'x' FROM \"s\")"
        assert TrimExpr(Ident("s"), where="BOTH", characters="x").to_sql(PostgreSql) == "TRIM(E'x' FROM \"s\")"
        assert TrimExpr(Ident("s"), where="LEADING", characters="x").to_sql(Sqlite) == "LTRIM(\"s\", 'x')"

        sql, params = TrimExpr(Ident("s"), characters="x").prepare(PostgreSql)
        assert sql == 'TRIM($1 FROM "s")'
        assert params == ("x",)

        assert repr(TrimExpr(Ident("s"))) == 'TrimExpr(Ident("s"))'
        assert repr(TrimExpr(Ident("s"), where="LEADING", characters="x")) == (
            'TrimExpr(Ident("s"), where="LEADING", characters=Value(\'x\', StringSerializer()))'
        )


class TestCollateExpr:
    def test_new(self):
        col = CollateExpr(Ident("name"), "en_US")
        assert same(col.get_expr(), Ident("name"))
        assert same(col.get_collation(), Ident("en_US"))

        col = CollateExpr("café", Ident("pg_catalog", "default"))
        assert same(col.get_expr(), Value("café"))
        assert same(col.get_collation(), Ident("pg_catalog", "default"))

        col = CollateExpr(Ident("name"), Unquote("C"))
        assert same(col.get_collation(), Ident(Unquote("C")))

        col = CollateExpr(Ident("name"), "pg_catalog.default")
        assert same(col.get_collation(), Ident("pg_catalog", "default"))

    def test_to_sql(self):
        assert CollateExpr(Ident("name"), "en_US").to_sql(PostgreSql) == ('"name" COLLATE "en_US"')
        assert CollateExpr(Ident("name"), Unquote("C")).to_sql(PostgreSql) == ('"name" COLLATE C')
        assert (
            CollateExpr(Ident("name"), Ident("pg_catalog", "default")).to_sql(PostgreSql)
            == '"name" COLLATE "pg_catalog"."default"'
        )
        assert CollateExpr(BinaryExpr(1, "+", 2), "en_US").to_sql(PostgreSql) == ('(1 + 2) COLLATE "en_US"')
        assert BinaryExpr(CollateExpr(Ident("a"), "c"), "+", 1).to_sql(PostgreSql) == '"a" COLLATE "c" + 1'

        assert repr(CollateExpr(Ident("name"), "en_US")) == 'CollateExpr(Ident("name"), Ident("en_US"))'


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
        assert same(when_list[0][1], Value("yes"))
        assert searched.get_else() is None

        with_else = searched.else_("no")
        assert with_else is not searched
        assert searched.get_else() is None
        assert same(with_else.get_else(), Value("no"))

        simple = CaseExpr(Ident("status"))
        assert same(simple.get_operand(), Ident("status"))

        CaseExpr().when(Ident("x"), 1)
        CaseExpr().when(Value(1), 1)
        CaseExpr().when(1, 2)

    def test_to_sql(self):
        assert CaseExpr().to_sql(PostgreSql) == "CASE END"

        searched = CaseExpr().when(BinaryExpr(Ident("id"), ">", 0), "yes").else_("no")
        assert searched.to_sql(PostgreSql) == "CASE WHEN \"id\" > 0 THEN E'yes' ELSE E'no' END"

        simple = CaseExpr(Ident("status")).when(BinaryExpr(Ident("status"), "=", "open"), 1).else_(0)
        assert simple.to_sql(PostgreSql) == 'CASE "status" WHEN "status" = E\'open\' THEN 1 ELSE 0 END'

        sql, params = searched.prepare(PostgreSql)
        assert sql == 'CASE WHEN "id" > $1 THEN $2 ELSE $3 END'
        assert params == (0, "yes", "no")


class TestCustomExpr:
    def test_new(self):
        custom = CustomExpr("1 = 1")
        assert custom.get_sql() == "1 = 1"
        assert custom.get_args() == ()

        custom = CustomExpr("{} = {}", Ident("a"), 1)
        assert custom.get_sql() == "{} = {}"
        assert same(custom.get_args(), (Ident("a"), Value(1)))

    def test_to_sql(self):
        assert CustomExpr("1 = 1").to_sql(PostgreSql) == "1 = 1"
        assert CustomExpr("{} = {}", Ident("a"), 1).to_sql(PostgreSql) == '"a" = 1'
        assert CustomExpr("{{x}}").to_sql(PostgreSql) == "{x}"
        assert CustomExpr("a {{}} b").to_sql(PostgreSql) == "a {} b"
        assert CustomExpr("fn({})", Ident("id")).to_sql(PostgreSql) == 'fn("id")'

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("a {} {}", 1).to_sql(PostgreSql)

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("a {}", 1, 2).to_sql(PostgreSql)

        sql, params = CustomExpr("x = {}", 1).prepare(PostgreSql)
        assert sql == "x = $1"
        assert params == (1,)

        with pytest.raises(ValueError, match="placeholders"):
            CustomExpr("{} {}", 1).prepare(PostgreSql)


class TestExprProtocol:
    def test_attribute(self):

        class Eq:
            __expr__ = BinaryExpr(1, "=", 2)

        cust = CustomExpr("{}", Eq)
        assert isinstance(cust.get_args()[0], BinaryExpr)
        assert cust.to_sql(PostgreSql) == "1 = 2"

    def test_property(self):

        class Eq:
            def __init__(self, a: object, b: object):
                self.a = a
                self.b = b

            @property
            def __expr__(self):
                return BinaryExpr(1, "=", 2)

        cust = CustomExpr("{}", Eq(1, 2))
        assert isinstance(cust.get_args()[0], BinaryExpr)
        assert cust.to_sql(PostgreSql) == "1 = 2"
