import pytest

from rapidquery import Asterisk, Expr, Function, Ident, Unquote, Value
from rapidquery.data_type import DataType
from rapidquery.expr import (
    Between,
    BinaryExpr,
    Case,
    Cast,
    Custom,
    InList,
    Is,
    IsDistinctFrom,
    Like,
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
        assert isinstance(expr.left, Ident)
        assert expr.left[0] == "a"
        assert expr.op == "+"
        assert isinstance(expr.right, Ident)
        assert repr(expr) == 'Binary(Ident("a"), "+", Ident("b"))'

    def test_value_for_string_literal(self):
        expr = BinaryExpr("id", "=", Value("x"))
        assert isinstance(expr.left, Ident)
        assert isinstance(expr.right, Value)
        assert expr.right.object == "x"

    def test_custom_operator(self):
        expr = BinaryExpr("data", "->", Unquote("key"))
        assert expr.op == "->"

    def test_unary_not(self):
        expr = UnaryExpr("NOT", "x")
        assert expr.op == "NOT"
        assert isinstance(expr.expr, Ident)

    def test_unknown_unary_raises(self):
        with pytest.raises(ValueError, match="unknown unary"):
            UnaryExpr("@", "x")


class TestFunc:
    def test_args_are_idents(self):
        fn = Function("SUM", "amount")
        assert isinstance(fn.name, Ident)
        assert fn.args is not None
        assert isinstance(fn.args[0], Ident)
        assert fn.filter is None

    def test_bare_function(self):
        fn = Function("CURRENT_TIMESTAMP", parentheses=False)
        assert fn.args is None
        assert "parentheses=False" in repr(fn)

    def test_distinct(self):
        fn = Function("COUNT", "id", distinct=True)
        assert "distinct=True" in repr(fn)


class TestNodes:
    def test_tuple_expr(self):
        expr = TupleExpr("a", "b")
        assert len(expr.items) == 2
        assert isinstance(expr.items[0], Ident)

    def test_is_null(self):
        expr = Is("x", "NULL", negated=True)
        assert expr.kind == "NULL"
        assert expr.negated is True

    def test_in_list(self):
        expr = InList("x", ["a", "b"])
        assert len(expr.list) == 2
        assert isinstance(expr.list[0], Ident)

    def test_between(self):
        expr = Between("x", "a", "b")
        assert isinstance(expr.low, Ident)
        assert isinstance(expr.high, Ident)

    def test_like_pattern_is_literal(self):
        expr = Like("name", "%a%")
        assert isinstance(expr.expr, Ident)
        assert isinstance(expr.pattern, Value)
        assert expr.pattern.object == "%a%"
        assert expr.kind == "LIKE"

    def test_cast(self):
        expr = Cast("x", DataType("int"))
        assert isinstance(expr.expr, Ident)
        assert expr.data_type.name == "INT"

    def test_case(self):
        expr = Case([("a", "b")], else_="c")
        assert expr.operand is None
        assert len(expr.when_then) == 1
        assert isinstance(expr.when_then[0][0], Ident)
        assert isinstance(expr.else_, Ident)

    def test_custom(self):
        expr = Custom("json_agg(DISTINCT {})", "col")
        assert expr.sql == "json_agg(DISTINCT {})"
        assert isinstance(expr.args[0], Ident)

    def test_is_distinct_from(self):
        expr = IsDistinctFrom("a", "b", negated=True)
        assert expr.negated is True


class TestEquality:
    def test_ident_equality_still_works(self):
        assert Ident("t", "id") == Ident("t", "id")
        assert hash(Ident("id")) != hash(Ident(Unquote("id")))

    def test_binary_equality(self):
        assert BinaryExpr("a", "+", "b") == BinaryExpr("a", "+", "b")
        assert BinaryExpr("a", "+", "b") != BinaryExpr("a", "-", "b")
