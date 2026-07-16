import pytest

from rapidquery._native.data_type import CustomSerializer, Serializer


class TestCustomSerializerConstruction:
    def test_is_serializer_subclass(self):
        assert issubclass(CustomSerializer, Serializer)

    def test_constructs_with_callable(self):
        CustomSerializer(lambda dialect, obj: str(obj))

    def test_function_getter_returns_the_same_callable(self):
        def my_func(dialect, obj):
            return str(obj)

        serializer = CustomSerializer(my_func)
        assert serializer.function is my_func

    @pytest.mark.parametrize("value", [None, 123, "not callable", [1, 2, 3]])
    def test_rejects_non_callable(self, value):
        with pytest.raises(TypeError):
            CustomSerializer(value)

    def test_accepts_any_callable_object_not_just_functions(self):
        class Callable:
            def __call__(self, dialect, obj):
                return "called"

        # Should not raise.
        CustomSerializer(Callable())


class TestCustomSerializerInvocation:
    def test_calls_function_with_object_and_dialect_name(self, dialect_cls):
        captured = {}

        def capture(dialect_name, obj):
            captured["obj"] = obj
            captured["dialect_name"] = dialect_name
            return "ok"

        CustomSerializer(capture).serialize(dialect_cls, 42)

        assert captured["obj"] == 42
        assert captured["dialect_name"] == dialect_cls().name

    def test_returns_the_function_result(self, dialect_cls):
        serializer = CustomSerializer(lambda dialect, obj: f"CUSTOM({obj})")
        result = serializer.serialize(dialect_cls, "value")
        assert result == "CUSTOM(value)"

    def test_dialect_name_differs_per_dialect(self):
        from rapidquery._native.dialect import MySql, PostgreSql

        seen_names = []

        def capture(dialect_name, obj):
            seen_names.append(dialect_name)
            return "ok"

        serializer = CustomSerializer(capture)
        serializer.serialize(PostgreSql, "x")
        serializer.serialize(MySql, "x")

        assert seen_names[0] != seen_names[1]

    def test_exception_inside_function_propagates(self, dialect_cls):
        def failing(dialect, obj):
            raise ValueError("boom")

        with pytest.raises(ValueError, match="boom"):
            CustomSerializer(failing).serialize(dialect_cls, "x")

    def test_non_string_return_value_is_rejected(self, dialect_cls):
        serializer = CustomSerializer(lambda dialect, obj: 12345)  # type: ignore
        with pytest.raises(TypeError):
            serializer.serialize(dialect_cls, "x")

    def test_none_return_value_is_rejected(self, dialect_cls):
        serializer = CustomSerializer(lambda dialect, obj: None)  # type: ignore
        with pytest.raises(TypeError):
            serializer.serialize(dialect_cls, "x")

    def test_can_be_used_to_implement_arbitrary_formatting(self, dialect_cls):
        serializer = CustomSerializer(lambda dialect, obj: f"'{str(obj).upper()}'")
        result = serializer.serialize(dialect_cls, "hello")
        assert result == "'HELLO'"
