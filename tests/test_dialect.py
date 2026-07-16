import pytest

from rapidquery._native.dialect import SqlDialect

from .conftest import ALL_DIALECTS, ALL_DIALECT_NAMES


class TestDialectIdentity:
    def test_name_matches_class_name(self, dialect_cls):
        """`SomeDialect().name` should equal the dialect's own name, e.g. 'PostgreSql'."""
        instance = dialect_cls()
        assert instance.name == dialect_cls.__name__

    def test_name_is_a_string(self, dialect_cls):
        assert isinstance(dialect_cls().name, str)

    def test_name_is_stable_across_instances(self, dialect_cls):
        """Two independently constructed instances report the same name."""
        assert dialect_cls().name == dialect_cls().name

    def test_all_expected_dialects_are_exposed(self):
        import rapidquery._native.dialect as dialect_module

        for expected_name in ALL_DIALECT_NAMES:
            assert hasattr(dialect_module, expected_name), (
                f"expected dialect module to expose {expected_name!r}"
            )

    def test_different_dialects_have_different_names(self):
        names = {cls().name for cls in ALL_DIALECTS}
        assert len(names) == len(ALL_DIALECTS)


class TestDialectSubclassing:
    def test_is_subclass_of_sql_dialect(self, dialect_cls):
        assert issubclass(dialect_cls, SqlDialect)

    def test_instance_is_instance_of_sql_dialect(self, dialect_instance):
        assert isinstance(dialect_instance, SqlDialect)

    def test_dialect_classes_are_all_distinct(self):
        assert len(set(ALL_DIALECTS)) == len(ALL_DIALECTS)


class TestDialectFinality:
    def test_cannot_be_subclassed_further(self, dialect_cls):
        """Dialect classes are marked @final; subclassing them should fail."""
        with pytest.raises(TypeError):
            type(f"Sub{dialect_cls.__name__}", (dialect_cls,), {})

    def test_base_sql_dialect_can_be_subclassed(self):
        """Only the concrete dialects are @final; the base class itself is not."""

        class _CustomDialect(SqlDialect):
            pass

        assert issubclass(_CustomDialect, SqlDialect)


class TestDialectConstruction:
    def test_constructs_with_no_arguments(self, dialect_cls):
        # Should not raise.
        dialect_cls()

    def test_rejects_unexpected_arguments(self, dialect_cls):
        with pytest.raises(TypeError):
            dialect_cls("unexpected")

    def test_name_is_read_only(self, dialect_instance):
        with pytest.raises(AttributeError):
            dialect_instance.name = "Something"
