def test_import_exposes_workspace_version() -> None:
    from importlib.metadata import distribution
    from importlib.resources import files

    import fmp_py

    installed_distribution = distribution("fmp-py-sdk")

    assert installed_distribution.metadata["Name"] == "fmp-py-sdk"
    assert installed_distribution.version == "0.1.0"
    assert fmp_py.__name__ == "fmp_py"
    assert fmp_py.__version__ == installed_distribution.version

    package_files = files("fmp_py")
    assert package_files.joinpath("py.typed").is_file()
    assert package_files.joinpath("__init__.pyi").is_file()
    assert package_files.joinpath("_native.pyi").is_file()
    assert package_files.joinpath("quote.pyi").is_file()


def test_public_exception_hierarchy_is_stable() -> None:
    import pickle

    import fmp_py
    from fmp_py import (
        FmpConfigError,
        FmpDecodeError,
        FmpError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )
    from fmp_py import _native

    subclasses = (
        FmpValidationError,
        FmpConfigError,
        FmpTransportError,
        FmpStatusError,
        FmpDecodeError,
    )
    assert all(issubclass(exception, FmpError) for exception in subclasses)
    assert all(issubclass(exception, Exception) for exception in subclasses)
    assert fmp_py.FmpError is _native.FmpError
    assert {exception.__name__ for exception in subclasses} == {
        "FmpValidationError",
        "FmpConfigError",
        "FmpTransportError",
        "FmpStatusError",
        "FmpDecodeError",
    }
    assert all(exception.__module__ == "fmp_py._native" for exception in subclasses)
    assert FmpError.__module__ == "fmp_py._native"

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
    from fmp_py import (
        FmpConfigError,
        FmpDecodeError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )
    from fmp_py import _native

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

    assert "_test_error" not in __import__("fmp_py").__all__
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
