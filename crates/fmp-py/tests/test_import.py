"""Import-time contract: package metadata, public names, and the error hierarchy.

The generated surface is checked against the shipped stubs rather than a
hand-kept list: every ``fmp._native`` package with an ``__init__.pyi`` must
import, and every name its ``__all__`` declares must exist at runtime, so a
model that ``gen_models`` emits but ``register_namespaces`` does not add (or the
reverse) fails here instead of at a user's import.
"""

import ast
import importlib
import pickle
import sys
from importlib.metadata import distribution
from importlib.resources import files
from pathlib import Path
from types import ModuleType, SimpleNamespace

import pytest

PACKAGE_ROOT = Path(str(files("fmp")))


def _packages_under(root: Path, prefix: tuple[str, ...], marker: str) -> list[str]:
    """Dotted names of every package below ``root`` that ships ``marker``."""
    return sorted(
        ".".join((*prefix, *stub.parent.relative_to(root).parts))
        for stub in root.rglob(marker)
        if stub.parent != root
    )


NATIVE_PACKAGES = _packages_under(PACKAGE_ROOT / "_native", ("fmp", "_native"), "__init__.pyi")
PUBLIC_PACKAGES = _packages_under(PACKAGE_ROOT, ("fmp",), "__init__.py")


def _stub_all(package: str) -> list[str]:
    """The ``__all__`` list declared in ``package``'s shipped ``__init__.pyi``."""
    stub = PACKAGE_ROOT.joinpath(*package.split(".")[1:], "__init__.pyi")
    tree = ast.parse(stub.read_text(encoding="utf-8"), filename=str(stub))
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__" for target in node.targets
        ):
            return list(ast.literal_eval(node.value))
    return []


def test_version_matches_the_installed_distribution() -> None:
    """``fmp.__version__`` is the version of the ``fmp-py-sdk`` distribution."""
    import fmp

    installed = distribution("fmp-py-sdk")
    assert installed.metadata["Name"] == "fmp-py-sdk"
    assert fmp.__name__ == "fmp"
    assert isinstance(fmp.__version__, str)
    assert fmp.__version__ == installed.version
    assert files("fmp").joinpath("py.typed").is_file()


def test_public_names_are_exported() -> None:
    """The client and version are in ``__all__``; the client is the native type."""
    import fmp
    from fmp import FmpClient

    assert "FmpClient" in fmp.__all__
    assert "__version__" in fmp.__all__
    assert {"FmpError", "FmpStatusError"} <= set(fmp.__all__)
    assert "QuoteShort" not in fmp.__all__
    assert not hasattr(fmp, "QuoteShort")
    assert not hasattr(fmp._native, "_FmpError")
    assert FmpClient is fmp._native.FmpClient
    assert type(FmpClient(auth_mode="none", base_url="http://127.0.0.1:0").quote).__name__ == "QuoteNamespace"


def test_quote_models_are_reexported_from_the_native_module() -> None:
    """``fmp.quote`` re-exports the native models under the native module path."""
    import fmp.quote
    from fmp._native import quote as native_quote
    from fmp.quote import Quote, QuoteShort

    assert isinstance(fmp.quote, ModuleType)
    assert fmp.quote is sys.modules["fmp.quote"]
    assert {"Quote", "QuoteShort"} <= set(fmp.quote.__all__)
    assert Quote is native_quote.Quote
    assert QuoteShort is native_quote.QuoteShort
    assert Quote.__module__ == "fmp._native.quote"
    assert QuoteShort.__module__ == "fmp._native.quote"
    assert native_quote is sys.modules["fmp._native.quote"]


def test_chart_models_are_registered() -> None:
    """``fmp._native.chart`` exposes the registered chart models."""
    from fmp._native import chart

    for name in ("StockChartLightBar", "StockChartIntradayBar"):
        model = getattr(chart, name)
        assert isinstance(model, type)
        assert model.__module__ == "fmp._native.chart"


def test_every_domain_ships_a_native_stub_and_a_public_package() -> None:
    """The stub inventory is non-trivial and every native domain has a public twin."""
    assert len(NATIVE_PACKAGES) > 30
    assert {"fmp._native.quote", "fmp._native.statements.growth.income", "fmp._native.bulk.eod"} <= set(NATIVE_PACKAGES)
    public = set(PUBLIC_PACKAGES)
    for package in NATIVE_PACKAGES:
        if package != "fmp._native.errors":
            assert package.replace("fmp._native.", "fmp.", 1) in public


@pytest.mark.parametrize("package", NATIVE_PACKAGES)
def test_native_submodule_is_importable_and_published(package: str) -> None:
    """``import fmp._native.<path>`` resolves and is the module bound on its parent."""
    module = importlib.import_module(package)

    parent_name, _, attribute = package.rpartition(".")
    assert module is sys.modules[package]
    assert getattr(sys.modules[parent_name], attribute) is module


@pytest.mark.parametrize("package", NATIVE_PACKAGES)
def test_native_stub_names_exist_at_runtime(package: str) -> None:
    """Every name the shipped ``__init__.pyi`` exports is registered on the module."""
    module = importlib.import_module(package)
    exported = _stub_all(package)

    assert exported, f"{package} declares no __all__"
    for name in exported:
        assert hasattr(module, name), f"{package}.{name} is in the stub but not registered"
        member = getattr(module, name)
        if isinstance(member, type) and not issubclass(member, BaseException):
            assert member.__module__ == package


@pytest.mark.parametrize("package", PUBLIC_PACKAGES)
def test_public_package_reexports_its_native_module(package: str) -> None:
    """``fmp.<path>`` imports and every ``__all__`` entry is the native object."""
    module = importlib.import_module(package)
    native_name = package.replace("fmp.", "fmp._native.", 1)
    native = sys.modules.get(native_name)

    assert module is sys.modules[package]
    assert module.__all__ == sorted(module.__all__)
    for name in module.__all__:
        member = getattr(module, name)
        if native is not None and not isinstance(member, ModuleType):
            assert member is getattr(native, name)


def test_domain_models_import_from_their_public_packages() -> None:
    """Flat and nested domain packages expose the generated models by name."""
    from fmp.analyst import FinancialEstimate
    from fmp.bulk.eod import BulkEodBar
    from fmp.statements.income import IncomeStatement

    assert FinancialEstimate.__module__ == "fmp._native.analyst"
    assert BulkEodBar.__module__ == "fmp._native.bulk.eod"
    assert IncomeStatement.__module__ == "fmp._native.statements.income"


def test_client_quote_namespace_is_the_generated_getter(errors: SimpleNamespace) -> None:
    """``client.quote`` is the generated namespace and validates its arguments."""
    from fmp import FmpClient
    from fmp.quote import QuoteNamespace

    client = FmpClient(auth_mode="none", base_url="http://127.0.0.1:0")
    assert isinstance(client.quote, QuoteNamespace)
    with pytest.raises(errors.FmpValidationError):
        client.quote.short("")


ERROR_NAMES = (
    "FmpError",
    "FmpValidationError",
    "FmpConfigError",
    "FmpTransportError",
    "FmpStatusError",
    "FmpDecodeError",
)


def test_exception_hierarchy_is_stable(errors: SimpleNamespace) -> None:
    """Every error subclasses ``FmpError`` and carries the structured attributes."""
    subclasses = (
        errors.FmpValidationError,
        errors.FmpConfigError,
        errors.FmpTransportError,
        errors.FmpStatusError,
        errors.FmpDecodeError,
    )
    assert issubclass(errors.FmpError, Exception)
    assert all(issubclass(exception, errors.FmpError) for exception in subclasses)
    assert errors.FmpError.__module__ == "fmp.errors"
    assert all(exception.__module__ == "fmp.errors" for exception in subclasses)


def test_errors_package_reexports_the_native_hierarchy() -> None:
    """``fmp.errors`` and the top-level ``fmp`` expose the same native exception types."""
    import fmp
    import fmp.errors
    from fmp import FmpError, FmpStatusError
    from fmp._native import errors as native_errors

    assert isinstance(fmp.errors, ModuleType)
    assert fmp.errors is sys.modules["fmp.errors"]
    assert native_errors is sys.modules["fmp._native.errors"]
    assert set(ERROR_NAMES) <= set(fmp.errors.__all__)
    assert FmpError is fmp.errors.FmpError is native_errors.FmpError
    assert FmpStatusError is fmp.errors.FmpStatusError is native_errors.FmpStatusError
    for name in ERROR_NAMES:
        assert getattr(fmp, name) is getattr(fmp.errors, name)


@pytest.mark.parametrize("name", ERROR_NAMES)
def test_exceptions_survive_a_pickle_round_trip(errors: SimpleNamespace, name: str) -> None:
    """Pickling resolves the class through ``fmp.errors`` and keeps the attributes."""
    cls = getattr(errors, name)
    exception = cls("message")
    exception.endpoint = "/quote"
    exception.status = 401

    restored = pickle.loads(pickle.dumps(exception))

    assert type(restored) is cls
    assert restored.args == ("message",)
    assert restored.category == cls.category
    assert (restored.endpoint, restored.status) == ("/quote", 401)


@pytest.mark.parametrize(
    ("name", "category"),
    [
        ("FmpError", None),
        ("FmpValidationError", "validation"),
        ("FmpConfigError", "configuration"),
        ("FmpTransportError", "transport"),
        ("FmpStatusError", "status"),
        ("FmpDecodeError", "decode"),
    ],
)
def test_exception_attributes_default_to_none(errors: SimpleNamespace, name: str, category: str | None) -> None:
    """A bare exception instance reports its category and no request context."""
    exception = getattr(errors, name)("message")
    assert exception.args == ("message",)
    assert exception.category == category
    assert (exception.endpoint, exception.status, exception.body, exception.body_truncated) == (None, None, None, None)
    assert (exception.decode_path, exception.decode_kind) == (None, None)
    assert (exception.retry_after, exception.headers, exception.proxy_error) == (None, None, None)
