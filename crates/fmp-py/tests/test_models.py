"""Protocol contract every generated response model honours.

The sweep covers every model class the shipped stubs declare: each one is
built from sample values derived from its stub ``__new__`` annotations, so a
model added by ``gen_models`` is checked without a hand-kept list.

Models are keyword-only, compare by value, are unhashable (``__hash__`` is
``None``, the same as a Python class that defines ``__eq__`` alone), and
pickle through ``__getnewargs_ex__``.
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


@dataclass(frozen=True)
class ModelStub:
    """One model class and its ``__new__`` keyword annotations from the stub."""

    module: str
    name: str
    params: tuple[tuple[str, str], ...]
    json_params: frozenset[str]

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
            found[f"{module}.{node.name}"] = ModelStub(module, node.name, params, json_params)
    return found


MODELS = _model_stubs()


def _sample(annotation: str, module: str) -> Any:
    """A valid value for one stub annotation."""
    if annotation.startswith("typing.Optional["):
        return None
    if annotation.startswith(("builtins.list[", "typing.Sequence[")):
        return []
    simple: dict[str, Any] = {
        "builtins.str": "x",
        "builtins.int": 7,
        "builtins.float": 1.5,
        "builtins.bool": True,
        "datetime.date": datetime.date(2024, 1, 2),
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
