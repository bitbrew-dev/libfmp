"""Import-time contract: package metadata, public names, and the error hierarchy."""

import sys
from importlib.metadata import distribution
from importlib.resources import files
from types import ModuleType, SimpleNamespace

import pytest


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
