# standard library
from typing import Literal, Optional

# fmp library
from fmp.quote import QuoteShort

class FmpClient:
    """Synchronous FMP client backed by the async Rust transport.

    Supplying ``token`` without ``auth_mode`` selects FMP's exact ``apikey``
    header. Omitting both selects no auth, which requires a custom base URL.
    ``timeout`` and ``connect_timeout`` are positive finite numbers of seconds.
    ``max_response_body_bytes`` bounds each buffered response.
    Redirects are either disabled or restricted to the same origin.
    """

    def __init__(
        self,
        *,
        token: Optional[str] = ...,
        base_url: Optional[str] = ...,
        path_prefix: Optional[str] = ...,
        auth_mode: Optional[
            Literal[
                "none",
                "fmp_header",
                "fmp_query",
                "bearer",
                "custom_header",
                "custom_query",
            ]
        ] = ...,
        auth_name: Optional[str] = ...,
        auth_prefix: Optional[str] = ...,
        headers: Optional[dict[str, str]] = ...,
        timeout: Optional[float] = ...,
        connect_timeout: Optional[float] = ...,
        max_response_body_bytes: Optional[int] = ...,
        follow_redirects: Optional[bool] = ...,
    ) -> None: ...
    def quote_short(self, symbol: str) -> list[QuoteShort]: ...

__all__: list[str]
