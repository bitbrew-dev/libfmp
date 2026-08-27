def test_import_exposes_workspace_version() -> None:
    from importlib.metadata import distribution
    from importlib.resources import files

    import fmp

    installed_distribution = distribution("fmp-py-sdk")

    assert installed_distribution.metadata["Name"] == "fmp-py-sdk"
    assert fmp.__name__ == "fmp"
    assert fmp.__version__ == installed_distribution.version

    package_files = files("fmp")
    assert package_files.joinpath("py.typed").is_file()
    assert package_files.joinpath("__init__.pyi").is_file()
    assert package_files.joinpath("_native.pyi").is_file()
    assert package_files.joinpath("client.py").is_file()
    assert package_files.joinpath("client.pyi").is_file()
    assert package_files.joinpath("errors.py").is_file()
    assert package_files.joinpath("errors.pyi").is_file()
    assert package_files.joinpath("quote.py").is_file()
    assert package_files.joinpath("quote.pyi").is_file()
    assert fmp.__all__ == [
        "FmpClient",
        "FmpConfigError",
        "FmpDecodeError",
        "FmpError",
        "FmpStatusError",
        "FmpTransportError",
        "FmpValidationError",
        "QuoteShort",
        "__version__",
    ]


def test_public_imports_resolve_to_registered_native_modules() -> None:
    import sys
    from types import ModuleType

    import fmp
    import fmp.client
    import fmp.errors
    import fmp.quote
    from fmp import FmpClient
    from fmp import QuoteShort
    from fmp.client import FmpClient as DomainFmpClient
    from fmp.errors import FmpError as DomainFmpError
    from fmp.quote import QuoteShort as DomainQuoteShort

    modules = {
        "client": fmp.client,
        "errors": fmp.errors,
        "quote": fmp.quote,
    }
    for name, module in modules.items():
        assert isinstance(module, ModuleType)
        assert module is sys.modules[f"fmp.{name}"]
        assert module is getattr(fmp._native, name)
        assert module.__name__ == name
        assert module.__package__ is None
        assert not hasattr(module, "__file__")

    assert fmp.client.__all__ == ["FmpClient"]
    assert fmp.errors.__all__ == [
        "FmpError",
        "FmpValidationError",
        "FmpConfigError",
        "FmpTransportError",
        "FmpStatusError",
        "FmpDecodeError",
    ]
    assert fmp.quote.__all__ == ["QuoteShort"]
    assert FmpClient is DomainFmpClient
    assert FmpClient.__module__ == "fmp.client"
    assert fmp.FmpError is DomainFmpError
    assert QuoteShort is DomainQuoteShort
    assert QuoteShort.__module__ == "fmp.quote"

    assert not hasattr(fmp._native, "FmpClient")
    assert not hasattr(fmp._native, "FmpError")
    assert not hasattr(fmp._native, "QuoteShort")


def test_public_exception_hierarchy_is_stable() -> None:
    import pickle

    import fmp
    from fmp import (
        FmpConfigError,
        FmpDecodeError,
        FmpError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )
    from fmp import _native
    from fmp.errors import (
        FmpConfigError as DomainFmpConfigError,
        FmpDecodeError as DomainFmpDecodeError,
        FmpError as DomainFmpError,
        FmpStatusError as DomainFmpStatusError,
        FmpTransportError as DomainFmpTransportError,
        FmpValidationError as DomainFmpValidationError,
    )

    subclasses = (
        FmpValidationError,
        FmpConfigError,
        FmpTransportError,
        FmpStatusError,
        FmpDecodeError,
    )
    assert all(issubclass(exception, FmpError) for exception in subclasses)
    assert all(issubclass(exception, Exception) for exception in subclasses)
    assert FmpError is DomainFmpError
    assert subclasses == (
        DomainFmpValidationError,
        DomainFmpConfigError,
        DomainFmpTransportError,
        DomainFmpStatusError,
        DomainFmpDecodeError,
    )
    assert fmp.errors is _native.errors
    assert {exception.__name__ for exception in subclasses} == {
        "FmpValidationError",
        "FmpConfigError",
        "FmpTransportError",
        "FmpStatusError",
        "FmpDecodeError",
    }
    assert all(exception.__module__ == "fmp.errors" for exception in subclasses)
    assert FmpError.__module__ == "fmp.errors"
    assert not hasattr(_native, "FmpError")

    expected_categories = {
        FmpError: None,
        FmpValidationError: "validation",
        FmpConfigError: "configuration",
        FmpTransportError: "transport",
        FmpStatusError: "status",
        FmpDecodeError: "decode",
    }
    for exception_type, category in expected_categories.items():
        exception = exception_type("message")
        assert exception.category == category
        assert exception.endpoint is None
        assert exception.status is None
        assert exception.body is None
        assert exception.body_truncated is None
        restored = pickle.loads(pickle.dumps(exception))
        assert type(restored) is exception_type
        assert restored.args == ("message",)
        assert restored.category == category


def test_native_error_conversion_has_exact_safe_attributes() -> None:
    from fmp import (
        FmpConfigError,
        FmpDecodeError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )
    from fmp import _native

    safe_body = "denied?apikey=[REDACTED] [REDACTED]"
    cases = {
        "validation": (
            FmpValidationError,
            ("validation", None, None, None, None),
            "invalid ticker",
        ),
        "configuration": (
            FmpConfigError,
            ("configuration", None, None, None, None),
            "invalid client configuration",
        ),
        "transport": (
            FmpTransportError,
            ("transport", "quote-short", None, None, None),
            "request failed (endpoint: quote-short)",
        ),
        "status": (
            FmpStatusError,
            ("status", "quote-short", 401, safe_body, False),
            f"provider returned HTTP status 401 (endpoint: quote-short): {safe_body}",
        ),
        "decode": (
            FmpDecodeError,
            ("decode", "quote-short", 200, safe_body, False),
            f"response decode failed (endpoint: quote-short): {safe_body}",
        ),
    }

    assert "_test_error" not in __import__("fmp").__all__
    for category, (exception_type, attributes, message) in cases.items():
        try:
            _native._test_error(category)
        except exception_type as error:
            assert (
                error.category,
                error.endpoint,
                error.status,
                error.body,
                error.body_truncated,
            ) == attributes
            assert str(error) == message
            diagnostic = f"{error!s} {error!r} {error.body}"
            assert "query-secret" not in diagnostic
            assert "body-secret" not in diagnostic
        else:
            raise AssertionError(f"{category} did not raise {exception_type.__name__}")
