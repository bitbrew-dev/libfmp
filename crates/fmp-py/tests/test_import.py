def test_import_exposes_workspace_version() -> None:
    import fmp_py

    assert fmp_py.__version__ == "0.1.0"
