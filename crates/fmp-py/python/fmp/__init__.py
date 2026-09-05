"""Python bindings for the Financial Modeling Prep API, powered by libfmp."""

# fmp library
from fmp import _native
from fmp._native import FmpClient

__version__: str = _native.__version__

__all__ = ["FmpClient", "__version__"]
