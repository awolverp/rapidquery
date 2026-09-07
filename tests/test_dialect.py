import pytest

from rapidquery.dialect import SqlDialect

from .conftest import ALL_DIALECT_NAMES, ALL_DIALECTS


class TestDialectIdentity:
    def test_name_matches_class_name(self, dialect_cls: type[SqlDialect]):
        instance = dialect_cls()
        assert instance.name == dialect_cls.__name__

    def test_name_is_a_string(self, dialect_cls: type[SqlDialect]):
        assert isinstance(dialect_cls().name, str)

    def test_name_is_stable_across_instances(self, dialect_cls: type[SqlDialect]):
        assert dialect_cls().name == dialect_cls().name

    def test_all_expected_dialects_are_exposed(self):
        import rapidquery.dialect as dialect_module

        for expected_name in ALL_DIALECT_NAMES:
            assert hasattr(dialect_module, expected_name), (
                f"expected dialect module to expose {expected_name!r}"
            )

    def test_different_dialects_have_different_names(self):
        names = {cls().name for cls in ALL_DIALECTS}
        assert len(names) == len(ALL_DIALECTS)


class TestDialectSubclassing:
    def test_is_subclass_of_sql_dialect(self, dialect_cls: type[SqlDialect]):
        assert issubclass(dialect_cls, SqlDialect)

    def test_instance_is_instance_of_sql_dialect(self, dialect_instance: SqlDialect):
        assert isinstance(dialect_instance, SqlDialect)

    def test_dialect_classes_are_all_distinct(self):
        assert len(set(ALL_DIALECTS)) == len(ALL_DIALECTS)


class TestDialectFinality:
    def test_cannot_be_subclassed_further(self, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            _ = type(f"Sub{dialect_cls.__name__}", (dialect_cls,), {})

    def test_base_sql_dialect_can_be_subclassed(self):
        class _CustomDialect(SqlDialect):
            pass

        assert issubclass(_CustomDialect, SqlDialect)


class TestDialectConstruction:
    def test_constructs_with_no_arguments(self, dialect_cls: type[SqlDialect]):
        _ = dialect_cls()

    def test_rejects_unexpected_arguments(self, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            _ = dialect_cls("unexpected")

    def test_name_is_read_only(self, dialect_instance: SqlDialect):
        with pytest.raises(AttributeError):
            dialect_instance.name = "Something"

    def test_base_cannot_be_instantiated(self):
        with pytest.raises(TypeError):
            _ = SqlDialect()


class TestDialectMarkers:
    def test_instances_of_same_dialect_are_equal(self, dialect_cls: type[SqlDialect]):
        assert dialect_cls() == dialect_cls()

    def test_instances_of_different_dialects_are_not_equal(self):
        from rapidquery.dialect import MySql, PostgreSQL

        assert PostgreSQL() != MySql()

    def test_instance_is_not_equal_to_its_class(self, dialect_cls: type[SqlDialect]):
        assert dialect_cls() != dialect_cls
        assert dialect_cls != dialect_cls()

    def test_not_equal_to_unrelated_values(self, dialect_instance: SqlDialect):
        assert dialect_instance != object()
        assert dialect_instance != dialect_instance.name
        assert dialect_instance != 1

    def test_equal_instances_share_a_hash(self, dialect_cls: type[SqlDialect]):
        assert hash(dialect_cls()) == hash(dialect_cls())

    def test_same_dialect_collapses_in_a_set(self, dialect_cls: type[SqlDialect]):
        assert len({dialect_cls(), dialect_cls()}) == 1

    def test_different_dialects_are_distinct_set_keys(self):
        from rapidquery.dialect import MySql, PostgreSQL

        assert len({PostgreSQL(), MySql()}) == 2

    def test_repr_is_call_form(self, dialect_cls: type[SqlDialect]):
        assert repr(dialect_cls()) == f"{dialect_cls.__name__}()"

    def test_repr_matches_name(self, dialect_instance: SqlDialect):
        assert repr(dialect_instance) == f"{dialect_instance.name}()"
