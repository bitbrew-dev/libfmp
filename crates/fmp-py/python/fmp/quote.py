"""Public quote-domain models, re-exported from the native extension.

Import these from here (``from fmp.quote import Quote``); ``fmp._native`` is an
internal implementation detail and is not part of the public API.
"""

# fmp library
from fmp._native.quote import Quote as Quote
from fmp._native.quote import QuoteShort as QuoteShort

__all__ = ["Quote", "QuoteShort"]
