"""Static-only contract exercised by mypy and pyright."""

from typing import Optional

import fmp
from fmp import _native
from fmp import FmpClient, FmpError, QuoteShort
from fmp.client import FmpClient as DomainFmpClient
from fmp.errors import FmpStatusError
from fmp.quote import QuoteShort as DomainQuoteShort


def check_public_contract(client: FmpClient, error: FmpError) -> None:
    domain_client: DomainFmpClient = client
    rows: list[QuoteShort] = domain_client.quote_short("AAPL")
    domain_row: DomainQuoteShort = rows[0]
    symbol: str = domain_row.symbol
    status: Optional[int] = error.status
    exports: list[str] = fmp.__all__
    version: str = fmp.__version__
    _ = (symbol, status, exports, version)


FmpClient(
    token="proxy-token",
    base_url="https://proxy.example",
    auth_mode="custom_header",
    auth_name="X-Proxy-Token",
    headers={"X-Tenant": "blue"},
    max_response_body_bytes=67_108_864,
)

status_error: FmpError = FmpStatusError("denied")

native_client_type: type[FmpClient] = _native._FmpClient
native_error_type: type[FmpError] = _native._FmpError
native_quote_type: type[QuoteShort] = _native._QuoteShort
