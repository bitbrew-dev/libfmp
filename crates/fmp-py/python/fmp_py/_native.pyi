from typing import Literal, Optional

AuthMode = Literal[
    "none",
    "fmp_header",
    "fmp_query",
    "bearer",
    "custom_header",
    "custom_query",
]

class QuoteShort:
    """A compact quote returned by the quote-short endpoint."""

    def __init__(self, symbol: str, price: float, change: float, volume: int) -> None: ...
    @property
    def symbol(self) -> str: ...
    @property
    def price(self) -> float: ...
    @property
    def change(self) -> float: ...
    @property
    def volume(self) -> int: ...

class FmpClient:
    """Synchronous FMP client backed by the async Rust transport.

    Supplying ``token`` without ``auth_mode`` selects FMP's exact ``apikey``
    header. Omitting both selects no auth, which requires a custom base URL.
    ``timeout`` and ``connect_timeout`` are positive finite numbers of seconds.
    Redirects are either disabled or restricted to the same origin.
    """

    def __init__(
        self,
        *,
        token: Optional[str] = ...,
        base_url: Optional[str] = ...,
        path_prefix: Optional[str] = ...,
        auth_mode: Optional[AuthMode] = ...,
        auth_name: Optional[str] = ...,
        auth_prefix: Optional[str] = ...,
        headers: Optional[dict[str, str]] = ...,
        timeout: Optional[float] = ...,
        connect_timeout: Optional[float] = ...,
        follow_redirects: Optional[bool] = ...,
    ) -> None: ...

    def quote_short(self, symbol: str) -> list[QuoteShort]: ...

class FmpError(Exception):
    """Base exception for all fmp_py failures."""

    category: Optional[str]
    endpoint: Optional[str]
    status: Optional[int]
    body: Optional[str]
    body_truncated: Optional[bool]

class FmpValidationError(FmpError):
    """An input failed local validation."""

class FmpConfigError(FmpError):
    """Client or transport configuration is invalid."""

class FmpTransportError(FmpError):
    """A request could not be completed by the transport."""

class FmpStatusError(FmpError):
    """The provider returned a non-success HTTP status."""

class FmpDecodeError(FmpError):
    """A successful response could not be decoded."""

def _test_error(category: str) -> None: ...

__version__: str
