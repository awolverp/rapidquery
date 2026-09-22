import pytest

from rapidquery import Asterisk, Ident, Unquote
from rapidquery.dialect import MySql, PostgreSql, SqlDialect


class TestIdentConstruction:
    def test_varargs_parts(self):
        ident = Ident("table", "id")
        assert len(ident) == 2
        assert ident[0] == "table"
        assert ident[1] == "id"

    def test_single_str_is_one_quoted_part_even_with_dots(self):
        ident = Ident("schema.table")

        assert len(ident) == 1
        assert ident[0] == "schema.table"
        assert ident.to_sql(PostgreSql()) == '"schema.table"'

    def test_rejects_no_arguments(self):
        with pytest.raises(ValueError, match="empty"):
            _ = Ident()

    def test_rejects_empty_string_part(self):
        with pytest.raises(ValueError, match="empty"):
            _ = Ident("")

    def test_rejects_non_ident_argument(self):
        with pytest.raises(TypeError):
            _ = Ident(1)  # pyright: ignore

    def test_flattens_nested_ident(self):
        ident = Ident(Ident("t"), "id")

        assert ident.to_list() == ["t", "id"]
        assert ident.to_sql(PostgreSql()) == '"t"."id"'

    def test_too_many_parts(self):
        with pytest.raises(ValueError, match="too many"):
            _ = Ident(*[f"p{i}" for i in range(11)])

    def test_ten_parts_are_allowed(self):
        ident = Ident(*[f"p{i}" for i in range(10)])
        assert len(ident) == 10


class TestQuotedStarVersusAsterisk:
    def test_str_star_is_quoted_column(self):
        ident = Ident("*")
        assert len(ident) == 1
        assert ident[0] == "*"
        assert ident.to_sql(PostgreSql()) == '"*"'
        assert ident.to_sql(MySql()) == "`*`"

    def test_asterisk_class_is_wildcard(self):
        ident = Ident(Asterisk)
        assert ident[0] == Asterisk()
        assert ident.to_sql(PostgreSql()) == "*"

    def test_asterisk_instance_is_wildcard(self):
        ident = Ident(Asterisk())
        assert ident.to_sql(MySql()) == "*"

    def test_qualified_wildcard(self):
        ident = Ident("t", Asterisk)
        assert ident.to_sql(PostgreSql()) == '"t".*'
        assert ident.to_sql(MySql()) == "`t`.*"

    def test_asterisk_not_last_is_rejected(self):
        with pytest.raises(ValueError, match="asterisk"):
            _ = Ident(Asterisk, "id")


class TestUnquote:
    def test_simple_name(self):
        part = Unquote("Users")
        assert part.name == "Users"
        assert Ident(part).to_sql(PostgreSql()) == "Users"

    def test_mixed_with_quoted_parts(self):
        ident = Ident("public", Unquote("Users"), "id")
        assert ident.to_sql(PostgreSql()) == '"public".Users."id"'

    def test_equals_by_name(self):
        assert Unquote("id") == Unquote("id")
        assert Unquote("id") != Unquote("ID")
        assert Unquote("id") != "id"

    def test_rejects_injection_and_non_ascii(self):
        for bad in [
            "",
            "*",
            "a;b",
            "a.b",
            "1id",
            "a-b",
            "a b",
            "café",
            "a; DROP TABLE t--",
        ]:
            with pytest.raises(ValueError):
                _ = Unquote(bad)

    def test_accepts_underscore_and_digits(self):
        assert Unquote("_id").name == "_id"
        assert Unquote("created_at").name == "created_at"
        assert Unquote("a1").name == "a1"


class TestEqualityHashReprFrozen:
    def test_equal_when_parts_match(self):
        assert Ident("t", "id") == Ident("t", "id")
        assert Ident("t", "id") != Ident("t", "name")

    def test_quoted_is_not_unquoted(self):
        assert Ident("id") != Ident(Unquote("id"))
        assert hash(Ident("id")) != hash(Ident(Unquote("id")))

    def test_asterisk_instances_are_equal(self):
        assert Asterisk() == Asterisk()
        assert Ident(Asterisk) == Ident(Asterisk())

    def test_usable_as_dict_key(self):
        mapping = {Ident("t", "id"): 1, Unquote("id"): 2, Asterisk(): 3}
        assert mapping[Ident("t", "id")] == 1
        assert mapping[Unquote("id")] == 2
        assert mapping[Asterisk()] == 3

    def test_repr(self):
        assert repr(Ident("schema", "table")) == 'Ident("schema", "table")'
        assert repr(Ident("t", Asterisk)) == 'Ident("t", Asterisk)'
        assert repr(Ident(Unquote("Users"))) == 'Ident(Unquote("Users"))'
        assert repr(Unquote("Users")) == 'Unquote("Users")'
        assert repr(Asterisk()) == "Asterisk()"


class TestRender:
    def test_to_sql_uses_dialect_quote_identifier(self, dialect_instance: SqlDialect):
        ident = Ident("users", "id")
        sql = ident.to_sql(dialect_instance)
        assert sql == (
            f"{dialect_instance.quote_identifier('users')}."
            f"{dialect_instance.quote_identifier('id')}"
        )

    def test_quoted_injection_stays_inside_delimiters(self):
        payload = "a; DROP TABLE t--"
        sql = Ident(payload).to_sql(PostgreSql())
        assert sql == PostgreSql().quote_identifier(payload)
