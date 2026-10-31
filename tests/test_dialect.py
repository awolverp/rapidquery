from collections.abc import Callable, Sequence

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


class TemplateCase:
    """Valid and invalid strings for one ``template(keys; min..=max)`` field.

    ``keys`` uses the Rust macro spelling. ``"left"`` is required, ``"hex?"`` is
    optional, ``"_"`` is the positional ``{}``, and ``"_?"`` is an optional ``{}``.
    ``maximum`` defaults to ``minimum``.

    Unknown ``{...}`` text is not a placeholder. A required key is reported only
    when the placeholder count is already inside ``[minimum, maximum]``.
    """

    def __init__(self, keys: Sequence[str], minimum: int, maximum: int | None = None) -> None:
        if minimum < 0:
            raise ValueError("minimum must be >= 0")

        self.minimum = minimum
        self.maximum = minimum if maximum is None else maximum

        if self.minimum > self.maximum:
            raise ValueError("minimum is greater than maximum")

        # (key name, `{name}` text, required). `_` is the empty positional name.
        parsed: list[tuple[str, str, bool]] = []
        for key in keys:
            optional = key.endswith("?")
            name = key[:-1] if optional else key
            if name == "_":
                name = ""

            parsed.append((name, "{" + name + "}", not optional))

        if not parsed:
            raise ValueError("expected at least one template key")

        self._keys = parsed

    def valid(self) -> list[str]:
        """Templates ``Template::parse`` accepts for this field."""

        required = [placeholder for _, placeholder, is_required in self._keys if is_required]
        optional = [placeholder for _, placeholder, is_required in self._keys if not is_required]
        out: list[str] = []

        def add(text: str) -> None:
            if text not in out:
                out.append(text)

        if not required and self.minimum == 0:
            add("")
            add("?")

        if not required and self.minimum <= 1 <= self.maximum:
            for placeholder in optional:
                add(placeholder)

        for count in (self.minimum, self.maximum):
            built = self._build(count, required, optional)
            if built is not None:
                add(built)

        skeleton = next((text for text in out if text not in ("", "?")), "")
        add(f"pre{skeleton}post{{not-a-key}}")
        return out

    def invalid(self) -> list[tuple[str, str]]:
        """``(template, ValueError match)`` pairs ``Template::parse`` rejects."""

        cases = [("a" * 257, "template is too long")]

        if self.minimum > 0:
            short = self.minimum - 1
            text = "plain" if short == 0 else self._keys[0][1] * short
            cases.append((text, "placeholders are missing"))

        cases.append((self._keys[0][1] * (self.maximum + 1), "too many placeholders"))

        missing = self._missing_required()
        if missing is not None:
            cases.append(missing)
        return cases

    def _build(self, count: int, required: list[str], optional: list[str]) -> str | None:
        if count < len(required) or not self.minimum <= count <= self.maximum:
            return None
        parts = list(required)
        if len(parts) == count:
            return "".join(parts)
        filler = required[0] if required else (optional[0] if optional else None)
        if filler is None:
            return None
        parts.extend([filler] * (count - len(parts)))
        return "".join(parts)

    def _missing_required(self) -> tuple[str, str] | None:
        """A count-legal template that omits one required key, when that can happen."""

        required = [(name, placeholder) for name, placeholder, is_required in self._keys if is_required]
        if not required:
            return None

        omit_name, omit_placeholder = required[-1]
        keep = [
            placeholder for _, placeholder, is_required in self._keys if is_required and placeholder != omit_placeholder
        ]
        fillers = keep + [placeholder for _, placeholder, is_required in self._keys if not is_required]
        target = max(self.minimum, len(keep))

        if not fillers:
            if self.minimum == 0 and not keep:
                return ("plain", f"one of required keys not found: {omit_name}")
            return None
        if target > self.maximum:
            return None

        parts = list(keep)
        while len(parts) < target:
            parts.append(fillers[0])

        present = set(parts)
        missing_name = next(
            name for name, placeholder, is_required in self._keys if is_required and placeholder not in present
        )
        return ("".join(parts), f"one of required keys not found: {missing_name}")


def _template_fields() -> list[tuple[str, Callable[[str], str], TemplateCase]]:
    positional = TemplateCase(("_",), 1)

    def string_template(text: str) -> str:
        return rq.dialect.StringConfig(template=text).template

    def string_concat(text: str) -> str:
        return rq.dialect.StringConfig(concat=text).concat

    def try_cast(text: str) -> str:
        rendered = rq.dialect.CastConfig(try_cast=text).try_cast
        assert rendered is not None
        return rendered

    def temporal(field: str) -> Callable[[str], str]:
        def accept(text: str) -> str:
            cfg = rq.dialect.TemporalConfig(**{field: text})
            rendered: str = getattr(cfg, field)
            return rendered

        return accept

    def literal(field: str) -> Callable[[str], str]:
        def accept(text: str) -> str:
            cfg = rq.dialect.LiteralConfig(**{field: text})
            rendered = getattr(cfg, field)
            assert rendered is not None
            return str(rendered)

        return accept

    def location(text: str) -> str:
        rendered = rq.dialect.SchemaConfig(location=text).location
        assert rendered is not None
        return rendered

    def placeholder(text: str) -> str:
        return rq.dialect.Dialect(placeholder=text).placeholder

    def identifier(text: str) -> str:
        return rq.dialect.Dialect(identifier=text).identifier

    string_keys = ("single?", "double?", "hex?", "hex_quote?", "mysql?", "unicode?", "unicode_quote?")
    identifier_keys = (
        "double?",
        "single?",
        "bracket?",
        "backtick?",
        "slashdouble?",
        "slashsingle?",
        "slashbracket?",
        "slashbacktick?",
    )
    return [
        ("string.template", string_template, TemplateCase(string_keys, 1)),
        ("string.concat", string_concat, TemplateCase(("left", "right"), 2)),
        ("cast.try_cast", try_cast, TemplateCase(("expr", "type"), 2)),
        ("temporal.date", temporal("date"), positional),
        ("temporal.time", temporal("time"), positional),
        ("temporal.datetime_naive", temporal("datetime_naive"), positional),
        ("temporal.datetime_aware", temporal("datetime_aware"), positional),
        ("literal.bit", literal("bit"), positional),
        ("literal.inet", literal("inet"), positional),
        ("literal.macaddr", literal("macaddr"), positional),
        ("literal.uuid", literal("uuid"), positional),
        ("literal.json", literal("json"), positional),
        ("literal.array", literal("array"), positional),
        ("literal.bytes", literal("bytes"), TemplateCase(("hex?", "slashx?"), 1)),
        ("schema.location", location, positional),
        ("dialect.placeholder", placeholder, TemplateCase(("_?",), 0, 1)),
        ("dialect.identifier", identifier, TemplateCase(identifier_keys, 1)),
    ]


@pytest.mark.parametrize(
    ("accept", "case"),
    [pytest.param(accept, case, id=name) for name, accept, case in _template_fields()],
)
def test_template_parameters(accept: Callable[[str], str], case: TemplateCase) -> None:
    for text in case.valid():
        assert accept(text) == text
    for text, match in case.invalid():
        with pytest.raises(ValueError, match=match):
            accept(text)
