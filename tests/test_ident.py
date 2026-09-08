import pytest

from rapidquery import Asterisk, Ident, Unquote
from rapidquery.dialect import MSSQL, MySql, PostgreSQL, SqlDialect, Teradata

from .conftest import ALL_DIALECTS


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
        assert ident.to_sql(PostgreSQL) == '"schema.table"'

    def test_rejects_no_arguments(self):
        with pytest.raises(ValueError, match="empty"):
            _ = Ident()

    def test_rejects_empty_string_part(self):
        with pytest.raises(ValueError, match="empty"):
            _ = Ident("")

    def test_rejects_non_ident_argument(self):
        with pytest.raises(TypeError):
            _ = Ident(1)

    def test_flattens_nested_ident(self):
        ident = Ident(Ident("t"), "id")
        assert list(ident) == ["t", "id"]
        assert ident.to_sql(PostgreSQL) == '"t"."id"'

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
        assert ident.to_sql(PostgreSQL) == '"*"'
        assert ident.to_sql(MySql) == "`*`"

    def test_asterisk_class_is_wildcard(self):
        ident = Ident(Asterisk)
        assert ident[0] == Asterisk()
        assert ident.to_sql(PostgreSQL) == "*"

    def test_asterisk_instance_is_wildcard(self):
        ident = Ident(Asterisk())
        assert ident.to_sql(MySql) == "*"

    def test_qualified_wildcard(self):
        ident = Ident("t", Asterisk)
        assert ident.to_sql(PostgreSQL) == '"t".*'
        assert ident.to_sql(MySql) == "`t`.*"

    def test_asterisk_not_last_is_rejected(self):
        with pytest.raises(ValueError, match="asterisk"):
            _ = Ident(Asterisk, "id")


class TestUnquote:
    def test_simple_name(self):
        part = Unquote("Users")
        assert part.name == "Users"
        assert Ident(part).to_sql(PostgreSQL) == "Users"

    def test_mixed_with_quoted_parts(self):
        ident = Ident("public", Unquote("Users"), "id")
        assert ident.to_sql(PostgreSQL) == '"public".Users."id"'

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


class TestIndexingAndIteration:
    def test_len_getitem_iter_to_list(self):
        ident = Ident("a", "b", "c")
        assert len(ident) == 3
        assert ident[0] == "a"
        assert ident[-1] == "c"
        assert list(ident) == ["a", "b", "c"]
        assert ident.to_list() == ["a", "b", "c"]

    def test_index_error(self):
        ident = Ident("a")
        with pytest.raises(IndexError):
            _ = ident[1]
        with pytest.raises(IndexError):
            _ = ident[-2]

    def test_slice_returns_ident(self):
        ident = Ident("a", "b", "c")
        sliced = ident[1:]
        assert isinstance(sliced, Ident)
        assert list(sliced) == ["b", "c"]
        assert sliced.to_sql(PostgreSQL) == '"b"."c"'

    def test_empty_slice_is_value_error(self):
        ident = Ident("a", "b")
        with pytest.raises(ValueError, match="empty"):
            _ = ident[1:1]

    def test_unquote_and_asterisk_parts_round_trip(self):
        ident = Ident("t", Unquote("Users"), Asterisk)
        assert ident[0] == "t"
        assert ident[1] == Unquote("Users")
        assert ident[2] == Asterisk()
        assert list(ident)[1] == Unquote("Users")


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
    def test_class_and_instance_dialect_agree(self):
        ident = Ident("id")
        assert ident.to_sql(PostgreSQL) == ident.to_sql(PostgreSQL())
        assert ident.to_sql(MySql) == ident.to_sql(MySql())

    @pytest.mark.parametrize("dialect_cls", ALL_DIALECTS)
    def test_render_succeeds_on_every_dialect(self, dialect_cls: type[SqlDialect]):
        ident = Ident("users", "id")
        sql = ident.to_sql(dialect_cls)
        assert "users" in sql
        assert "id" in sql
        assert "." in sql

    def test_delimiter_escaping(self):
        assert Ident('a"b').to_sql(PostgreSQL) == '"a""b"'
        assert Ident("a`b").to_sql(MySql) == "`a``b`"
        assert Ident("a]b").to_sql(MSSQL) == "[a]]b]"
        assert Ident("a'b").to_sql(Teradata) == "'a''b'"

    def test_quoted_injection_stays_inside_delimiters(self):
        payload = "a; DROP TABLE t--"
        sql = Ident(payload).to_sql(PostgreSQL)
        assert sql == f'"{payload}"'
        assert sql.startswith('"') and sql.endswith('"')
