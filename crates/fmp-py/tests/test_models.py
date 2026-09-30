"""Protocol contract every generated response model honours.

The sweep covers every model class the shipped stubs declare: each one is
built from sample values derived from its stub ``__new__`` annotations, so a
model added by ``gen_models`` is checked without a hand-kept list.

Models are keyword-only, compare by value, are unhashable (``__hash__`` is
``None``, the same as a Python class that defines ``__eq__`` alone), pickle
through ``__getnewargs_ex__``, list their attributes in ``__match_args__`` and
``repr()``, and convert to plain dicts with ``to_dict()``. Secret fields (no
attribute) stay out of all three.
"""

import ast
import copy
import datetime
import importlib
import inspect
import pickle
from dataclasses import dataclass
from importlib.resources import files
from pathlib import Path
from typing import Any

import pytest

NATIVE_ROOT = Path(str(files("fmp"))) / "_native"
NUMBER = "builtins.int | builtins.float"


@dataclass(frozen=True)
class ModelStub:
    """One model class and its ``__new__`` keyword annotations from the stub."""

    module: str
    name: str
    params: tuple[tuple[str, str], ...]
    json_params: frozenset[str]
    attributes: tuple[str, ...]

    @property
    def cls(self) -> type:
        """The runtime class."""
        return getattr(importlib.import_module(self.module), self.name)


def _model_stubs() -> dict[str, ModelStub]:
    """Every stub class that pickles by keyword, keyed by dotted path."""
    found: dict[str, ModelStub] = {}
    for stub in sorted(NATIVE_ROOT.rglob("__init__.pyi")):
        module = ".".join(("fmp", "_native", *stub.parent.relative_to(NATIVE_ROOT).parts))
        tree = ast.parse(stub.read_text(encoding="utf-8"))
        for node in tree.body:
            if not isinstance(node, ast.ClassDef):
                continue
            methods = {item.name: item for item in node.body if isinstance(item, ast.FunctionDef)}
            new = methods.get("__new__")
            if new is None or "__getnewargs_ex__" not in methods:
                continue
            params = tuple((arg.arg, ast.unparse(arg.annotation)) for arg in new.args.kwonlyargs if arg.annotation)
            json_params = frozenset(
                name
                for name, method in methods.items()
                if method.returns is not None and ast.unparse(method.returns) == "typing.Any"
            )
            attributes = tuple(name for name, _ in params if name in methods)
            found[f"{module}.{node.name}"] = ModelStub(module, node.name, params, json_params, attributes)
    return found


MODELS = _model_stubs()


def _sample(annotation: str, module: str) -> Any:
    """A valid value for one stub annotation."""
    for wrapper in ("typing.Optional[", "builtins.list[", "typing.Sequence["):
        if annotation.startswith(wrapper):
            inner = _sample(annotation[len(wrapper) : -1], module)
            return inner if wrapper == "typing.Optional[" else [inner]
    if annotation.startswith("typing.Literal["):
        return ast.literal_eval(annotation.removeprefix("typing.Literal[").split(",")[0].rstrip("]"))
    simple: dict[str, Any] = {
        "builtins.str": "x",
        "builtins.int": 7,
        "builtins.float": 1.5,
        "builtins.bool": True,
        NUMBER: 2.5,
        f"{NUMBER} | None": 3,
        "datetime.date": datetime.date(2024, 1, 2),
        "datetime.date | builtins.int": 2020,
        "datetime.datetime": datetime.datetime(2024, 1, 2, 3, 4, 5),
    }
    if annotation in simple:
        return simple[annotation]
    target = annotation if "." in annotation else f"{module}.{annotation}"
    return build(MODELS[target])


def kwargs_for(model: ModelStub) -> dict[str, Any]:
    """Sample keyword arguments that construct ``model``."""
    kwargs = {name: _sample(annotation, model.module) for name, annotation in model.params}
    for name in model.json_params & kwargs.keys():
        if kwargs[name] == "x":
            kwargs[name] = "{}"
    return kwargs


def build(model: ModelStub) -> Any:
    """An instance of ``model`` built from sample values."""
    kwargs = kwargs_for(model)
    try:
        return model.cls(**kwargs)
    except ValueError:
        for name in model.json_params & kwargs.keys():
            if kwargs[name] == "{}":
                kwargs[name] = "1"
        return model.cls(**kwargs)


def test_the_sweep_finds_the_generated_models() -> None:
    """The stub scan sees every generated model, including nested helpers."""
    assert len(MODELS) > 150
    assert "fmp._native.quote.Quote" in MODELS


@pytest.fixture(params=sorted(MODELS), ids=lambda path: path.removeprefix("fmp._native."))
def model(request: pytest.FixtureRequest) -> ModelStub:
    """One generated model."""
    return MODELS[request.param]


def test_constructor_is_keyword_only(model: ModelStub) -> None:
    """Every ``__new__`` parameter is keyword-only; a positional call fails."""
    parameters = inspect.signature(model.cls).parameters.values()
    assert all(parameter.kind is inspect.Parameter.KEYWORD_ONLY for parameter in parameters)
    with pytest.raises(TypeError):
        model.cls(*kwargs_for(model).values())


def test_equality_compares_values(model: ModelStub) -> None:
    """Two rows built from the same values are equal; other types are not."""
    row = build(model)
    assert row == build(model)
    assert not row != build(model)
    assert row != object()


def test_models_are_unhashable(model: ModelStub) -> None:
    """``__hash__`` is ``None``, so a row cannot be a set member or dict key."""
    assert model.cls.__hash__ is None
    with pytest.raises(TypeError, match="unhashable"):
        hash(build(model))


@pytest.mark.parametrize("protocol", range(2, pickle.HIGHEST_PROTOCOL + 1))
def test_pickle_round_trip_is_equal(model: ModelStub, protocol: int) -> None:
    """Pickling rebuilds the row by keyword and it compares equal."""
    row = build(model)
    assert pickle.loads(pickle.dumps(row, protocol=protocol)) == row


def test_copy_and_deepcopy_are_equal(model: ModelStub) -> None:
    """``copy`` goes through the same keyword reconstruction."""
    row = build(model)
    assert copy.copy(row) == row
    assert copy.deepcopy(row) == row


def test_match_args_list_the_attributes(model: ModelStub) -> None:
    """``__match_args__`` is every attribute, in constructor order."""
    assert model.cls.__match_args__ == model.attributes


def test_positional_match_pattern_binds_attributes(model: ModelStub) -> None:
    """A positional ``case`` pattern binds the first attribute."""
    row = build(model)
    if not model.attributes:
        pytest.skip("model has no attributes")
    first = model.attributes[0]
    match row:
        case model.cls(value):  # type: ignore[misc]
            assert value == getattr(row, first)
        case _:
            pytest.fail("positional pattern did not match")


def test_repr_lists_every_attribute(model: ModelStub) -> None:
    """``repr()`` is ``Name(field=repr(value), ...)`` over the attributes."""
    if len(model.attributes) != len(model.params):
        pytest.skip("secret-bearing models keep their redacting repr (test_secrets.py)")
    row = build(model)
    fields = ", ".join(f"{name}={getattr(row, name)!r}" for name in model.attributes)
    assert repr(row) == f"{model.name}({fields})"


def _plain(value: Any) -> Any:
    """``value`` with every model replaced by its ``to_dict()``."""
    if isinstance(value, list):
        return [_plain(item) for item in value]
    if hasattr(value, "to_dict"):
        return value.to_dict()
    return value


def test_to_dict_maps_attributes_to_plain_values(model: ModelStub) -> None:
    """``to_dict()`` keys are the attributes; nested models become dicts."""
    row = build(model)
    as_dict = row.to_dict()
    assert type(as_dict) is dict
    assert list(as_dict) == list(model.attributes)
    for name in model.attributes:
        assert as_dict[name] == _plain(getattr(row, name))


NUMBER_FIELDS = sorted(
    (path, name) for path, model in MODELS.items() for name, annotation in model.params if annotation == NUMBER
)


def test_number_fields_are_typed_int_or_float() -> None:
    """The former ``Any`` JSON-number fields are stubbed ``int | float``."""
    assert len(NUMBER_FIELDS) > 40


@pytest.mark.parametrize(
    "value", [0, -7, 10**30, -(10**30), 0.25, 2.0, 1e300], ids=["zero", "neg", "big", "big-neg", "frac", "whole", "huge"]
)
def test_number_field_round_trips_ints_and_floats(value: float) -> None:
    """An ``int`` keeps its exact digits and type; a ``float`` stays a float."""
    path, name = NUMBER_FIELDS[0]
    model = MODELS[path]
    row = model.cls(**{**kwargs_for(model), name: value})
    assert getattr(row, name) == value
    assert type(getattr(row, name)) is type(value)
    assert pickle.loads(pickle.dumps(row)) == row
    assert row.to_dict()[name] == value


@pytest.mark.parametrize(
    ("value", "error"),
    [(True, TypeError), ("1", TypeError), (None, TypeError), (float("nan"), ValueError), (float("inf"), ValueError)],
    ids=["bool", "str", "none", "nan", "inf"],
)
def test_number_field_rejects_other_values(value: object, error: type[Exception]) -> None:
    """``bool``, text, ``None`` for a required field, and non-finite floats fail."""
    path, name = NUMBER_FIELDS[0]
    model = MODELS[path]
    with pytest.raises(error):
        model.cls(**{**kwargs_for(model), name: value})
