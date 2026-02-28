"""
A custom stub generator which imports a compiled library and uses docstrings to
understand and generate type stubs.

Supported docstring tags:
  @signature (params) -> ReturnType
      Override the detected signature and/or return type.
      May span multiple lines if the closing `)` or `->` is absent.
  @overload (params) -> ReturnType
      Declare one overload variant. Multiple tags produce multiple overloads.
      Each value must be a complete `(params) -> ReturnType` expression.
  @extends BaseClass, ...
      Explicitly set base classes (skips auto-detection from __bases__).
  @setter TypeName
      Declare that a getset-descriptor property has a setter accepting TypeName.
  @alias Name: TypeAlias = ...
      Emit a TypeAlias assignment in the types section.

Additional conventions:
  - TypeVars are detected automatically from single uppercase letters in
    signatures and base-class expressions.
  - @typing.final is emitted when subclassing a class raises TypeError.
  - __stub_imports__ on a module is appended verbatim to the import block.
"""

from __future__ import annotations

import dataclasses
import enum
import importlib
import inspect
import re
import sys
import types
import typing

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_TAG_RE = re.compile(r"^\s*@(\w+)(?:\s+(.+?))?\s*$")
_TYPEVAR_RE = re.compile(r"\b([A-Z])\b")


def _csv(value: str | None) -> list[str]:
    """Split a comma-separated tag value, dropping blank parts."""
    if not value:
        return []
    return [part for part in (p.strip() for p in value.split(",")) if part]


def _signature_is_complete(sig: str) -> bool:
    """Return True when *sig* ends a signature (has `)` or `->`)."""
    return "->" in sig or sig.rstrip().endswith(")")


# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class Signature:
    """A parsed signature fragment: optional parameter list and return type."""

    parameters: str | None = None
    return_type: str | None = None


@dataclasses.dataclass
class Docstring:
    """All structured information extracted from a docstring."""

    document: str | None = None
    extends: list[str] = dataclasses.field(default_factory=list)
    overloads: list[Signature] = dataclasses.field(default_factory=list)
    signature: Signature = dataclasses.field(default_factory=Signature)
    setter_type: str | None = None
    aliases: list[str] = dataclasses.field(default_factory=list)

    @classmethod
    def parse(
        cls,
        obj: object,
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
    ) -> typing.Self:
        raw = obj if isinstance(obj, str) else inspect.getdoc(obj)
        if raw is None:
            return cls()

        result = cls()
        prose: list[str] = []

        # "sig"     → currently extending result.signature.parameters
        # "overload" → currently extending result.overloads[-1].parameters
        _pending: typing.Literal["sig", "overload"] | None = None

        for line in raw.splitlines():
            # --- Continue a multi-line tag ---
            if _pending is not None:
                target = result.signature if _pending == "sig" else result.overloads[-1]
                assert target.parameters is not None
                target.parameters += " " + line.strip()
                if _signature_is_complete(target.parameters):
                    updated = _split_sig_return(target)
                    if _pending == "sig":
                        result.signature = updated
                    else:
                        result.overloads[-1] = updated
                    _pending = None
                continue

            match = _TAG_RE.match(line)
            if match is None:
                prose.append(line)
                continue

            tag, value = match.group(1), (match.group(2) or "").strip() or None

            if tag == "extends":
                result.extends.extend(_csv(value))

            elif tag == "signature":
                if not value:
                    continue
                result.signature.parameters = value
                if _signature_is_complete(value):
                    result.signature = _split_sig_return(result.signature)
                else:
                    _pending = "sig"

            elif tag == "overload":
                if not value:
                    continue
                result.overloads.append(Signature(parameters=value))
                if _signature_is_complete(value):
                    result.overloads[-1] = _split_sig_return(result.overloads[-1])
                else:
                    _pending = "overload"

            elif tag == "setter":
                result.setter_type = value or "Incomplete"

            elif tag == "alias":
                if not value:
                    continue
                eq = value.find("=")
                if eq == -1:
                    raise ValueError(f"Invalid @alias tag (no '=' found): @alias {value}")
                result.aliases.append(value[:eq] + ": typing.TypeAlias" + value[eq:])

        result.document = "\n".join(prose).strip() or None

        if custom_return_type is not None:
            result.signature.return_type = custom_return_type
        if custom_signature is not None:
            result.signature.parameters = custom_signature

        return result


def _split_sig_return(sig: Signature) -> Signature:
    """Extract an inline `-> ReturnType` from *sig.parameters* into *sig.return_type*."""
    if sig.parameters and "->" in sig.parameters:
        params, ret = sig.parameters.split("->", 1)
        return Signature(parameters=params.strip(), return_type=ret.strip())
    return sig


# ---------------------------------------------------------------------------
# Implementation-type detection
# ---------------------------------------------------------------------------


class ImplementationType(enum.IntEnum):
    MODULE = enum.auto()
    CLASS = enum.auto()
    FUNCTION = enum.auto()
    BUILTIN_METHOD = enum.auto()
    METHOD_DESCRIPTOR = enum.auto()
    GETSET_DESCRIPTOR = enum.auto()
    UNKNOWN = enum.auto()

    def is_function(self) -> bool:
        return self in (
            ImplementationType.FUNCTION,
            ImplementationType.BUILTIN_METHOD,
            ImplementationType.METHOD_DESCRIPTOR,
        )

    def is_class(self) -> bool:
        return self == ImplementationType.CLASS

    def is_module(self) -> bool:
        return self == ImplementationType.MODULE

    def is_getset(self) -> bool:
        return self == ImplementationType.GETSET_DESCRIPTOR

    @classmethod
    def guess(cls, obj: object) -> ImplementationType:
        if inspect.ismodule(obj):
            return cls.MODULE
        if inspect.isclass(obj):
            return cls.CLASS
        if inspect.ismethoddescriptor(obj):
            return cls.METHOD_DESCRIPTOR
        if inspect.isgetsetdescriptor(obj):
            return cls.GETSET_DESCRIPTOR
        if isinstance(obj, types.BuiltinMethodType):
            return cls.BUILTIN_METHOD
        if callable(obj):
            return cls.FUNCTION
        return cls.UNKNOWN


# ---------------------------------------------------------------------------
# Signature utilities
# ---------------------------------------------------------------------------


def _is_final(cls: type) -> bool:
    """Return True when *cls* cannot be subclassed."""
    try:

        class _Sub(cls):  # type: ignore[misc]
            pass
    except TypeError:
        return True
    return False


def _normalize_signature(sig: str, obj: object) -> str:
    """Strip the C-level `$` marker and inject `cls` for class constructors."""
    if sig.startswith(("($cls", "($self")):
        sig = "(" + sig[2:]

    if inspect.isclass(obj):
        if sig == "()":
            return "(cls)"
        if not sig.startswith("(cls"):
            return "(cls, " + sig[1:]

    return sig


def _infer_signature(obj: object) -> str | None:
    """Best-effort signature string for *obj*, normalized for use in stubs."""
    raw = getattr(obj, "__text_signature__", None)
    if raw is not None:
        return _normalize_signature(raw, obj)

    try:
        spec = inspect.getfullargspec(obj)
    except TypeError:
        try:
            return _normalize_signature(
                str(inspect.signature(obj)),  # type: ignore
                obj,
            )
        except (TypeError, ValueError):
            return None

    args: list[str] = spec.args or []
    defaults: tuple[object, ...] = spec.defaults or ()
    annotations: dict[str, object] = spec.annotations or {}
    num_positional = len(args) - len(defaults)
    padded_defaults: list[object | None] = [None] * num_positional + list(defaults)

    def ann(name: str) -> str:
        a = annotations.get(name)
        return f": {a.__name__}" if isinstance(a, type) else ""  # type: ignore[union-attr]

    def default_str(val: object | None) -> str:
        return f" = {val!r}" if val is not None else ""

    parts: list[str] = [f"{a}{ann(a)}{default_str(d)}" for a, d in zip(args, padded_defaults)]

    if spec.varargs:
        parts.append(f"*{spec.varargs}{ann(spec.varargs)}")
    elif spec.kwonlyargs:
        parts.append("*")

    kw_defaults: dict[str, object] = spec.kwonlydefaults or {}
    for kw in spec.kwonlyargs:
        kd = f" = {kw_defaults[kw]!r}" if kw in kw_defaults else ""
        parts.append(f"{kw}{ann(kw)}{kd}")

    if spec.varkw:
        parts.append(f"**{spec.varkw}{ann(spec.varkw)}")

    return _normalize_signature(f"({', '.join(parts)})", obj)


# ---------------------------------------------------------------------------
# Known return types for special methods
# ---------------------------------------------------------------------------

_SPECIAL_RETURN: dict[str, str] = {
    # Representation
    "__repr__": "str",
    "__str__": "str",
    "__format__": "str",
    "__bytes__": "bytes",
    "__fspath__": "str",
    # Numeric coercions
    "__bool__": "bool",
    "__int__": "int",
    "__float__": "float",
    "__complex__": "complex",
    "__index__": "int",
    "__trunc__": "int",
    "__floor__": "int",
    "__ceil__": "int",
    "__round__": "int",
    "__abs__": "int",
    "__sizeof__": "int",
    "__hash__": "int",
    "__len__": "int",
    "__length_hint__": "int",
    # Comparisons
    "__eq__": "bool",
    "__ne__": "bool",
    "__lt__": "bool",
    "__le__": "bool",
    "__gt__": "bool",
    "__ge__": "bool",
    "__contains__": "bool",
    # Lifecycle
    "__new__": "typing.Self",
    "__init__": "None",
    "__copy__": "typing.Self",
    "__deepcopy__": "typing.Self",
    "__pos__": "typing.Self",
    "__neg__": "typing.Self",
    "__invert__": "typing.Self",
    # In-place operators
    "__iadd__": "typing.Self",
    "__isub__": "typing.Self",
    "__imul__": "typing.Self",
    "__itruediv__": "typing.Self",
    "__ifloordiv__": "typing.Self",
    "__imod__": "typing.Self",
    "__ipow__": "typing.Self",
    "__imatmul__": "typing.Self",
    "__ilshift__": "typing.Self",
    "__irshift__": "typing.Self",
    "__iand__": "typing.Self",
    "__ior__": "typing.Self",
    "__ixor__": "typing.Self",
    # Context managers
    "__enter__": "typing.Self",
    "__exit__": "bool",
    "__aenter__": "typing.Self",
    "__aexit__": "bool",
    # Iteration
    "__iter__": "typing.Iterator[Incomplete]",
    "__aiter__": "typing.AsyncIterator[Incomplete]",
    "__reversed__": "typing.Iterator[Incomplete]",
    # Async
    "__await__": "typing.Generator[typing.Any, None, typing.Self]",
    "__anext__": "Incomplete",
    "__next__": "Incomplete",
    # Deletion / setters
    "__del__": "None",
    "__delattr__": "None",
    "__delitem__": "None",
    "__delslice__": "None",
    "__init_subclass__": "None",
    "__set_name__": "None",
    "__setattr__": "None",
    "__setitem__": "None",
    "__set__": "None",
}

# ---------------------------------------------------------------------------
# Expr operator overrides
# ---------------------------------------------------------------------------

# These methods return Self instead of the standard bool/NotImplemented so we
# keep explicit overrides rather than relying on _SPECIAL_RETURN.
_EXPR_OVERRIDES: dict[str, Docstring] = {
    f"Expr.{name}": Docstring(
        document=doc,
        signature=Signature(parameters=params, return_type="typing.Self"),
    )
    for name, doc, params in [
        (
            "__eq__",
            "Create an equality comparison expression.",
            "(self, other: object)",
        ),
        (
            "__ne__",
            "Create an inequality comparison expression.",
            "(self, other: object)",
        ),
        (
            "__gt__",
            "Create a greater-than comparison expression.",
            "(self, other: object)",
        ),
        (
            "__ge__",
            "Create a greater-than-or-equal comparison expression.",
            "(self, other: object)",
        ),
        (
            "__lt__",
            "Create a less-than comparison expression.",
            "(self, other: object)",
        ),
        (
            "__le__",
            "Create a less-than-or-equal comparison expression.",
            "(self, other: object)",
        ),
        ("__add__", "Create an addition expression.", "(self, other: object)"),
        ("__sub__", "Create a subtraction expression.", "(self, other: object)"),
        ("__and__", "Create a logical AND expression.", "(self, other: object)"),
        ("__neg__", "Create a negation expression.", "(self)"),
        ("__or__", "Create a logical OR expression.", "(self, other: object)"),
        ("__truediv__", "Create a division expression.", "(self, other: object)"),
        (
            "__lshift__",
            "Create a bitwise left-shift expression.",
            "(self, other: object)",
        ),
        (
            "__rshift__",
            "Create a bitwise right-shift expression.",
            "(self, other: object)",
        ),
        ("__mod__", "Create a modulo expression.", "(self, other: object)"),
        ("__mul__", "Create a multiplication expression.", "(self, other: object)"),
    ]
}


def _lookup_expr_override(obj: object) -> Docstring | None:
    """Return an Expr operator override docstring, or None."""
    objclass = getattr(obj, "__objclass__", None)
    if objclass is None:
        return None

    name: str = obj.__name__  # type: ignore[union-attr]
    key = f"{objclass.__name__}.{name}"
    if key in _EXPR_OVERRIDES:
        return _EXPR_OVERRIDES[key]

    # __radd__, __rsub__, … → try the non-reflected name
    if name.startswith("__r") and name.endswith("__"):
        key = f"{objclass.__name__}.__{name[3:]}"
        return _EXPR_OVERRIDES.get(key)

    return None


# ---------------------------------------------------------------------------
# Implementation
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class Implementation:
    """Fully resolved metadata for a single stub entry."""

    type: ImplementationType
    docstring: Docstring
    typevars: list[str] = dataclasses.field(default_factory=list)
    final: bool = False
    type_ignore: str = ""

    @classmethod
    def parse(
        cls,
        obj: object,
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
    ) -> typing.Self:
        impl_type = ImplementationType.guess(obj)
        if impl_type == ImplementationType.UNKNOWN:
            raise TypeError(f"Cannot determine implementation type for: {obj!r}")

        docstring = Docstring.parse(obj, custom_signature, custom_return_type)

        if impl_type.is_function():
            if override := _lookup_expr_override(obj):
                docstring = override

        result = cls(type=impl_type, docstring=docstring)
        result.final = _is_final(obj) if impl_type.is_class() else False  # type: ignore[arg-type]

        # --- Base classes ---
        if not result.docstring.extends and impl_type.is_class():
            result.docstring.extends = [
                b.__name__
                for b in obj.__bases__  # type: ignore[union-attr]
                if b is not object
            ]

        # --- Signature parameters ---
        sig = result.docstring.signature
        if sig.parameters:
            sig.parameters = _normalize_signature(sig.parameters, obj)
        else:
            sig.parameters = _infer_signature(obj) or (
                "(self)" if impl_type.is_function() else "(cls)"
            )

        # --- Return type ---
        if not sig.return_type:
            name: str = getattr(obj, "__name__", "")
            if impl_type.is_function():
                sig.return_type = _SPECIAL_RETURN.get(name, "Incomplete")
            elif impl_type.is_class():
                sig.return_type = _SPECIAL_RETURN["__new__"]
            else:
                sig.return_type = "Incomplete"

        # --- TypeVar detection (order-preserving dedup) ---
        tv_sources = [
            *result.docstring.extends,
            sig.return_type or "",
            sig.parameters or "",
            *(f"{ov.parameters} {ov.return_type}" for ov in result.docstring.overloads),
        ]
        seen: dict[str, None] = {}
        for src in tv_sources:
            for tv in _TYPEVAR_RE.findall(src):
                seen[tv] = None
        result.typevars = list(seen)

        # --- type: ignore[override] when return type diverges from the dunder table ---
        if sig.return_type and sig.return_type != "Incomplete":
            name = getattr(obj, "__name__", "")
            expected = (
                _SPECIAL_RETURN.get(name, sig.return_type)
                if impl_type.is_function()
                else _SPECIAL_RETURN.get("__new__", sig.return_type)
            )
            if sig.return_type != expected:
                result.type_ignore = "# type: ignore[override]"

        return result


# ---------------------------------------------------------------------------
# classmethod detection
# ---------------------------------------------------------------------------


def _is_classmethod(class_: type, member: object) -> bool:
    sig: str = getattr(member, "__text_signature__", "") or ""
    return getattr(member, "__self__", None) is class_ or "(cls" in sig or "$cls" in sig


# ---------------------------------------------------------------------------
# Stub generator
# ---------------------------------------------------------------------------


class StubGenerator:
    def __init__(self, root: types.ModuleType) -> None:
        self.root = root
        self._import_lines: list[str] = [
            "import typing",
            "from _typeshed import Incomplete",
        ]
        self._types_lines: list[str] = []
        self._body_lines: list[str] = []
        self.__all__: list[str] = []
        self.undefined_members: list[tuple[str, object]] = []
        self._indent = 0

    # ------------------------------------------------------------------
    # Low-level line helpers
    # ------------------------------------------------------------------

    def _emit(self, *lines: str) -> None:
        pad = " " * self._indent
        self._body_lines.extend(pad + line for line in lines)

    def _emit_docstring(self, text: str | None) -> None:
        if not text:
            return
        if len(text) > 70 or "\n" in text:
            self._emit('"""')
            for line in text.splitlines():
                self._emit(line)
            self._emit('"""')
        else:
            self._emit(f'"""{text.strip()}"""')

    def _emit_typevars(self, names: typing.Iterable[str]) -> None:
        for name in names:
            self._types_lines.append(f"{name} = typing.TypeVar({name!r})")

    def _emit_typealiases(self, aliases: typing.Iterable[str]) -> None:
        self._types_lines.extend(aliases)

    # ------------------------------------------------------------------
    # Visitors
    # ------------------------------------------------------------------

    def _visit_module(self, module: types.ModuleType) -> None:
        for key, val in inspect.getmembers(module):
            if key == "__stub_imports__":
                self._import_lines.extend(val)
                continue
            if key.startswith("__"):
                continue

            kind = ImplementationType.guess(val)
            if kind.is_class():
                self._visit_class(val)
                self.__all__.append(val.__name__)
            elif kind.is_function():
                self._visit_function(val)
                self.__all__.append(val.__name__)
            elif key.isupper():
                self._visit_constant(key, val)
                self.__all__.append(key)
            elif kind.is_module():
                raise NotImplementedError("Nested module re-export is not supported.")
            else:
                self.undefined_members.append((key, val))

    def _visit_function(
        self,
        fn: object,
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
        parent: type | None = None,
    ) -> None:
        impl = Implementation.parse(fn, custom_signature, custom_return_type)
        self._emit_typevars(impl.typevars)
        self._emit_typealiases(impl.docstring.aliases)
        self._emit("")

        name: str = fn.__name__  # type: ignore[union-attr]
        is_cls = parent is not None and _is_classmethod(parent, fn)

        if name == "__new__":
            impl.docstring.document = None

        # --- @typing.overload variants ---
        # In .pyi stubs, only the @overload variants are emitted — there is no
        # trailing implementation signature (that would be a mypy [misc] error).
        if impl.docstring.overloads:
            for ov in impl.docstring.overloads:
                if is_cls:
                    self._emit("@classmethod")
                self._emit("@typing.overload")
                params = ov.parameters or impl.docstring.signature.parameters
                ret = ov.return_type or "Incomplete"

                self._emit(f"def {name}{params} -> {ret}:")
                self._indent += 4
                self._emit_docstring(impl.docstring.document)
                self._emit("...")
                self._indent -= 4

                self._emit("")
            return

        # --- Non-overloaded definition ---
        if is_cls:
            self._emit("@classmethod")
        self._emit(
            "def {}{} -> {}: {}".format(
                name,
                impl.docstring.signature.parameters,
                impl.docstring.signature.return_type,
                impl.type_ignore,
            ).rstrip()
        )
        self._indent += 4
        self._emit_docstring(impl.docstring.document)
        self._emit("...")
        self._indent -= 4

    def _visit_getset_descriptor(self, descriptor: types.GetSetDescriptorType) -> None:
        impl = Implementation.parse(descriptor)
        self._emit("")
        self._emit(
            "@property",
            "def {}(self) -> {}:".format(descriptor.__name__, impl.docstring.signature.return_type),
        )
        self._indent += 4
        self._emit_docstring(impl.docstring.document)
        self._emit("...")
        self._indent -= 4

        if impl.docstring.setter_type is not None:
            self._emit(
                f"@{descriptor.__name__}.setter",
                f"def {descriptor.__name__}(self, value: {impl.docstring.setter_type}) -> None:",
            )
            self._indent += 4
            self._emit("...")
            self._indent -= 4

    def _visit_class(self, cls: type) -> None:
        impl = Implementation.parse(cls)
        self._emit_typevars(impl.typevars)
        self._emit_typealiases(impl.docstring.aliases)
        self._emit("")

        if impl.final:
            self._emit("@typing.final")

        bases = ", ".join(impl.docstring.extends)
        self._emit(f"class {cls.__name__}({bases}):" if bases else f"class {cls.__name__}:")
        self._indent += 4
        self._emit_docstring(impl.docstring.document)

        # Emit __new__ when the class has a meaningful signature
        sig = impl.docstring.signature
        if sig.parameters and sig.parameters != "(cls)":
            self._visit_function(
                cls.__new__,
                custom_signature=sig.parameters,
                custom_return_type=sig.return_type,
            )

        hash_is_none = False
        for attr_key, attr_val in inspect.getmembers(cls):
            attr_kind = ImplementationType.guess(attr_val)

            if attr_key in ("__new__", "__doc__", "__class__", "__module__"):
                continue

            if attr_key == "__hash__" and attr_val is None:
                hash_is_none = True
                continue

            if attr_kind == ImplementationType.METHOD_DESCRIPTOR:
                if attr_val.__objclass__ is cls:
                    self._visit_function(attr_val, parent=cls)

            elif attr_kind == ImplementationType.BUILTIN_METHOD:
                if not attr_key.startswith("__") and attr_val.__self__ is cls:
                    self._visit_function(attr_val, parent=cls)

            elif attr_kind == ImplementationType.GETSET_DESCRIPTOR:
                if attr_val.__objclass__ is cls:
                    self._visit_getset_descriptor(attr_val)

            elif attr_key.isupper():
                self._visit_constant(attr_key, attr_val, classvar=True)

            else:
                self.undefined_members.append((f"{cls.__name__}.{attr_key}", attr_val))

        if hash_is_none:
            self._emit("", "__hash__ = None  # type: ignore")

        self._indent -= 4

    def _visit_constant(
        self,
        name: str,
        value: object,
        classvar: bool = False,
    ) -> None:
        type_name = type(value).__name__
        if classvar:
            self._emit(f"{name}: typing.ClassVar[{type_name}] = ...")
        else:
            self._emit(f"{name}: typing.Final[{type_name}] = ...")

    # ------------------------------------------------------------------
    # Entry point
    # ------------------------------------------------------------------

    def generate(self) -> typing.Self:
        self._visit_module(self.root)
        return self

    def result(self) -> str:
        parts: list[str] = []

        root_doc = Docstring.parse(self.root)
        if root_doc.document:
            parts.append(f'"""\n{root_doc.document}\n"""\n')

        parts.append("from __future__ import annotations\n")
        parts.append("\n".join(dict.fromkeys(self._import_lines)))
        parts.append("\n")
        parts.append(
            "__all__ = [\n{}\n]".format(",\n".join(f"    {name!r}" for name in self.__all__))
        )
        parts.append("\n")
        if self._types_lines:
            parts.append("\n".join(dict.fromkeys(self._types_lines)))
            parts.append("\n")
        parts.append("\n".join(self._body_lines))

        return "\n".join(parts)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] in ("help", "-h", "--help"):
        print(f"Usage:\n\t{sys.argv[0]} IMPORT_NAME")
        return

    module = importlib.import_module(sys.argv[1])
    stub = StubGenerator(module).generate()

    for key, val in stub.undefined_members:
        print(f"Warning: undefined member: {key}\n\t{val!r}", file=sys.stderr)

    print(stub.result())


if __name__ == "__main__":
    main()
