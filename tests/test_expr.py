import pytest

from rapidquery import Asterisk, Expr, Function, Ident, Unquote, Value
from rapidquery.expr import (
    BetweenExpr,
    BinaryExpr,
    CaseExpr,
    CastExpr,
    CustomExpr,
    InListExpr,
    IsDistinctFromExpr,
    IsExpr,
    LikeExpr,
    TupleExpr,
    UnaryExpr,
)


class TestExprHierarchy:
    def test_ident_and_value_are_expr(self):
        assert isinstance(Ident("id"), Expr)
        assert isinstance(Value(1), Expr)
        assert isinstance(Function("SUM", "amount"), Expr)

    def test_ident_value_live_in_expr_module(self):
        assert Ident.__module__ == "rapidquery._core.expr"
        assert Value.__module__ == "rapidquery._core.expr"
        assert Unquote.__module__ == "rapidquery._core.expr"
        assert Asterisk.__module__ == "rapidquery._core.expr"
        assert Function.__module__ == "rapidquery._core.expr"

    def test_base_expr_cannot_be_instantiated(self):
        with pytest.raises(TypeError):
            Expr()

    def test_ident_repr_unchanged(self):
        assert repr(Ident("schema", "table")) == 'Ident("schema", "table")'

    def test_value_repr_unchanged(self):
        assert repr(Value(42)).startswith("Value(42,")


class TestBinaryUnary:
    def test_str_is_ident(self):
        expr = BinaryExpr("a", "+", "b")
        assert isinstance(expr.get_left(), Ident)
        assert expr.get_left().to_list()[0] == "a"
        assert expr.get_operator() == "+"
        assert isinstance(expr.get_right(), Ident)
        assert repr(expr) == 'BinaryExpr(Ident("a"), "+", Ident("b"))'

    def test_value_for_string_literal(self):
        expr = BinaryExpr("id", "=", Value("x"))
        assert isinstance(expr.get_left(), Ident)
        assert isinstance(expr.get_right(), Value)
        assert expr.get_right().get_object() == "x"

    def test_custom_operator(self):
        expr = BinaryExpr("data", "->", Unquote("key"))
        assert expr.get_operator() == "->"

    def test_unary_not(self):
        expr = UnaryExpr("NOT", "x")
        assert expr.get_operator() == "NOT"
        assert isinstance(expr.get_expr(), Ident)

    def test_unknown_unary_raises(self):
        with pytest.raises(ValueError, match="unknown unary"):
            UnaryExpr("@", "x")


class TestFunction:
    def test_args_are_idents(self):
        fn = Function("SUM", "amount")
        assert isinstance(fn.get_name(), Ident)
        assert fn.get_args() is not None
        assert isinstance(fn.get_args()[0], Ident)
        assert fn.get_filter() is None

    def test_bare_function(self):
        fn = Function("CURRENT_TIMESTAMP")
        assert fn.get_args() is None

    def test_distinct(self):
        fn = Function("COUNT", "id", distinct=True)
        assert "distinct=True" in repr(fn)


class TestNodes:
    def test_tuple_expr(self):
        expr = TupleExpr("a", "b")
        items = expr.get_items()
        assert len(items) == 2
        assert isinstance(items[0], Ident)

    def test_is_null(self):
        expr = IsExpr("x", "NULL", negated=True)
        assert expr.get_kind() == "NULL"
        assert expr.get_negated() is True

    def test_in_list(self):
        expr = InListExpr("x", ["a", "b"])
        assert len(expr.get_list()) == 2
        assert isinstance(expr.get_list()[0], Ident)

    def test_between(self):
        expr = BetweenExpr("x", "a", "b")
        assert isinstance(expr.get_low(), Ident)
        assert isinstance(expr.get_high(), Ident)

    def test_like_pattern_is_literal(self):
        expr = LikeExpr("name", "%a%")
        assert isinstance(expr.get_expr(), Ident)
        assert isinstance(expr.get_pattern(), Value)
        assert expr.get_pattern().get_object() == "%a%"
        assert expr.get_kind() == "LIKE"

    def test_cast(self):
        expr = CastExpr("x", "int")
        assert isinstance(expr.get_expr(), Ident)
        assert expr.get_data_type() == "INT"

    def test_custom(self):
        expr = CustomExpr("json_agg(DISTINCT {})", "col")
        assert expr.get_sql() == "json_agg(DISTINCT {})"
        assert isinstance(expr.get_args()[0], Ident)

    def test_is_distinct_from(self):
        expr = IsDistinctFromExpr("a", "b", negated=True)
        assert expr.get_negated() is True


class TestConditionExtractor:
    def test_case_when_accepts_binary_condition(self):
        cond = BinaryExpr("x", ">", "y")
        expr = CaseExpr().when(cond, "yes").else_("no")
        when_list = expr.get_when_list()
        assert len(when_list) == 1
        assert isinstance(when_list[0][0], BinaryExpr)
        assert isinstance(when_list[0][1], Ident)
        assert isinstance(expr.get_else(), Ident)

    def test_case_when_accepts_is_expr(self):
        expr = CaseExpr().when(IsExpr("x", "NULL"), "missing")
        assert isinstance(expr.get_when_list()[0][0], IsExpr)

    def test_case_when_rejects_ident(self):
        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(Ident("x"), "yes")

    def test_case_when_rejects_value(self):
        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(Value(1), "yes")

    def test_case_when_rejects_bare_str_and_int(self):
        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when("x", "yes")
        with pytest.raises(TypeError, match="condition"):
            CaseExpr().when(1, "yes")
