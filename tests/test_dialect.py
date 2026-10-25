import pytest

import rapidquery as rq


def test_default_constructors():
    for cls in (
        rq.dialect.AggregateConfig,
        rq.dialect.BitwiseConfig,
        rq.dialect.CastConfig,
        rq.dialect.Dialect,
        rq.dialect.LikeConfig,
        rq.dialect.LiteralConfig,
        rq.dialect.OperatorConfig,
        rq.dialect.SchemaConfig,
        rq.dialect.StringConfig,
        rq.dialect.TemporalConfig,
        rq.dialect.TupleConfig,
    ):
        _ = cls()


def test_enum_constructor():
    cfg = rq.dialect.AggregateConfig(
        null_treatment=rq.dialect.OPT_NULL_TREATMENT_UNSUPPORTED,
        function_filter=rq.dialect.OPT_FUNCTION_FILTER_UNSUPPORTED,
    )
    assert cfg.null_treatment == rq.dialect.OPT_NULL_TREATMENT_UNSUPPORTED
    assert cfg.function_filter == rq.dialect.OPT_FUNCTION_FILTER_UNSUPPORTED

    cfg = rq.dialect.AggregateConfig(
        null_treatment=rq.dialect.OPT_NULL_TREATMENT_INSIDE_PARENS,
        function_filter=rq.dialect.OPT_FUNCTION_FILTER_FILTER_WHERE,
    )
    assert cfg.null_treatment == rq.dialect.OPT_NULL_TREATMENT_INSIDE_PARENS
    assert cfg.function_filter == rq.dialect.OPT_FUNCTION_FILTER_FILTER_WHERE

    # invalid inputs
    with pytest.raises(ValueError):
        _ = rq.dialect.AggregateConfig(null_treatment=9)


def test_bitflags_constructor():
    cfg = rq.dialect.SchemaConfig(
        features=rq.dialect.OPT_SCHEMA_FEATURES_SUPPORTED | rq.dialect.OPT_SCHEMA_FEATURES_IF_NOT_EXISTS
    )

    assert cfg.features == rq.dialect.OPT_SCHEMA_FEATURES_SUPPORTED | rq.dialect.OPT_SCHEMA_FEATURES_IF_NOT_EXISTS
    assert cfg.features & rq.dialect.OPT_SCHEMA_FEATURES_SUPPORTED > 0
    assert cfg.features & rq.dialect.OPT_SCHEMA_FEATURES_DROP_CASCADE == 0

    # invalid inputs
    with pytest.raises(ValueError):
        _ = rq.dialect.SchemaConfig(features=1 << 7)


def test_template_constructor():
    # We don't need to test them here
    # because they are tested while creating named dialects
    pass


# Edge cases are tested in Rust
def test_helper_methods():
    d = rq.dialect.StringConfig(template="E'{hex}'")
    assert d.serialize_string("love") == "E'love'"

    d = rq.dialect.LiteralConfig(
        boolean=rq.dialect.OPT_BOOL_FORMAT_INTEGER,
        bit="'{}'HELLO WORLD",
        bytes="WELCOME '{hex}'",
    )
    assert d.serialize_boolean(True) == "1"
    assert d.serialize_boolean(False) == "0"
    assert d.serialize_bits("10101") == "'10101'HELLO WORLD"
    assert d.serialize_bytes(b"love") == "WELCOME '6C6F7665'"
