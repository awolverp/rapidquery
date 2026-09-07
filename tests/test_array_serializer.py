import pytest

from rapidquery.data_type import (
    ArraySerializer,
    IntegerSerializer,
    Serializer,
    StringSerializer,
)


class TestArraySerializerConstruction:
    def test_is_serializer_subclass(self):
        assert issubclass(ArraySerializer, Serializer)

    def test_constructs_with_element_serializer(self):
        # Should not raise.
        ArraySerializer(IntegerSerializer())

    def test_element_property_returns_the_given_serializer_type(self):
        arr = ArraySerializer(IntegerSerializer())
        assert isinstance(arr.element, IntegerSerializer)

    def test_element_property_reflects_string_serializer(self):
        arr = ArraySerializer(StringSerializer())
        assert isinstance(arr.element, StringSerializer)

    def test_rejects_non_serializer_element(self):
        with pytest.raises(TypeError):
            ArraySerializer("not-a-serializer")  # type: ignore

    def test_nested_array_serializer(self):
        """ArraySerializer should be composable (array of arrays)."""
        inner = ArraySerializer(IntegerSerializer())
        outer = ArraySerializer(inner)
        assert isinstance(outer.element, ArraySerializer)
        assert isinstance(outer.element.element, IntegerSerializer)


class TestArraySerializerIntendedBehaviour:
    def test_serializes_each_int_element_independently(self, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        result = arr.serialize(dialect_cls, [1, 2, 3])
        assert result == "ARRAY[1, 2, 3]"

    def test_serializes_tuple_same_as_list(self, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        list_result = arr.serialize(dialect_cls, [1, 2, 3])
        tuple_result = arr.serialize(dialect_cls, (1, 2, 3))
        assert list_result == tuple_result == "ARRAY[1, 2, 3]"

    def test_serializes_string_elements_with_correct_individual_quoting(
        self, dialect_cls
    ):
        arr = ArraySerializer(StringSerializer())
        result = arr.serialize(dialect_cls, ["a", "b"])
        # Each element must be independently quoted, not the whole list.
        assert result.count("a") == 1
        assert result.count("b") == 1

    def test_empty_array(self, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        result = arr.serialize(dialect_cls, [])
        assert result == "ARRAY[]"

    def test_single_element_array(self, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        result = arr.serialize(dialect_cls, [42])
        assert result == "ARRAY[42]"

    def test_element_type_error_propagates_with_correct_element_identified(
        self, dialect_cls
    ):
        """If a single element is the wrong type for the element serializer,
        the error should reflect that one bad element, not the container.
        """
        arr = ArraySerializer(IntegerSerializer())
        with pytest.raises(TypeError):
            arr.serialize(dialect_cls, [1, "not an int", 3])

    def test_wraps_result_in_array_literal_prefix(self, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        result = arr.serialize(dialect_cls, [1])
        assert result.startswith("ARRAY[") and result.endswith("]")


class TestArraySerializerContainerValidation:
    """These parts of the contract are independent of the per-element bug:
    input-type validation happens before the (buggy) per-element loop, so
    rejecting a bad *container* type still works correctly today.
    """

    @pytest.mark.parametrize("value", ["not a list", 123, None, {"a": 1}])
    def test_rejects_non_list_tuple_container(self, value, dialect_cls):
        arr = ArraySerializer(IntegerSerializer())
        with pytest.raises(TypeError):
            arr.serialize(dialect_cls, value)

    def test_rejects_set_as_container(self, dialect_cls):
        """Only list/tuple are documented as accepted containers, not set."""
        arr = ArraySerializer(IntegerSerializer())
        with pytest.raises(TypeError):
            arr.serialize(dialect_cls, {1, 2, 3})
