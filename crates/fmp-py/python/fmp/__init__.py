"""Python bindings for libfmp."""

# Importing the private extension registers the public native submodules before
# the imports below are resolved. Public classes are re-exported only from their
# stable domain modules.
# plugins
# itofin library
# fmp library
from fmp import _native as _native
from fmp.client import FmpClient
from fmp.errors import FmpConfigError, FmpDecodeError, FmpError, FmpStatusError, FmpTransportError, FmpValidationError
from fmp.quote import QuoteShort

__version__ = _native.__version__

__all__ = [
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
