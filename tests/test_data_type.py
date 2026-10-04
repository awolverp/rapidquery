import pytest

from rapidquery.data_type import DataType


class TestDataTypeNameNormalization:
    @pytest.mark.parametrize(
        "input_name,canonical",
        [
            ("int", "INT"),
            ("integer", "INTEGER"),
            ("varchar", "VARCHAR"),
            ("text", "TEXT"),
            ("uuid", "UUID"),
        ],
    )
    def test_known_names_are_uppercased_to_canonical_form(self, input_name, canonical):
        dt = DataType(input_name)
        assert dt.name == canonical

    def test_name_matching_is_case_insensitive(self):
        assert DataType("INTEGER") == DataType("integer")
        assert DataType("Integer") == DataType("integer")

    def test_unrecognized_name_is_preserved_as_is(self):
        dt = DataType("some_totally_unknown_type")
        assert dt.name == "some_totally_unknown_type"

    def test_parameterized_name_is_preserved_as_is(self):
        dt = DataType("VARCHAR(255)")
        assert dt.name == "VARCHAR(255)"

    def test_parameterized_name_is_not_alias_matched(self):
        dt = DataType("varchar(255)")
        assert dt.name == "varchar(255)"


class TestDataTypeEquality:
    def test_interned_spellings_of_the_same_name_are_equal(self):
        assert DataType("int") == DataType("INT")
        assert hash(DataType("int")) == hash(DataType("INT"))

    def test_distinct_interned_names_are_not_equal(self):
        assert DataType("int") != DataType("integer")

    def test_not_equal_to_unrelated_types(self):
        assert DataType("int") != "INT"
        assert DataType("int") != 1
