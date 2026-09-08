"""Static-only contract exercised by pyright and mypy against the shipped stubs."""

import fmp
from fmp import BinaryPayload, FmpClient, FmpError
from fmp.errors import FmpStatusError, FmpValidationError
from fmp.quote import Quote, QuoteShort


def check_public_contract(client: FmpClient) -> None:
    """Type-check the namespaced call shapes and the model fields."""
    short_rows: list[QuoteShort] = client.quote.short("AAPL")
    full_rows: list[Quote] = client.quote.full("AAPL")
    fund_rows: list[QuoteShort] = client.quote.mutual_funds()
    symbol: str = short_rows[0].symbol
    price: float = full_rows[0].price
    volume: int = fund_rows[0].volume
    market_cap: int | None = full_rows[0].market_cap
    exports: list[str] = fmp.__all__
    version: str = fmp.__version__
    _ = (symbol, price, volume, market_cap, exports, version)


def check_binary_contract() -> None:
    """Type-check the binary payload fields."""
    payload: BinaryPayload = BinaryPayload(b"", "application/octet-stream")
    data: bytes = payload.data
    content_type: str = payload.content_type
    disposition: str | None = payload.content_disposition
    byte_len: int = payload.byte_len
    _ = (data, content_type, disposition, byte_len)


def check_error_contract(client: FmpClient) -> None:
    """Type-check the exception hierarchy and its structured attributes."""
    try:
        client.quote.short("AAPL")
    except FmpStatusError as error:
        status: int | None = error.status
        endpoint: str | None = error.endpoint
        body: str | None = error.body
        body_truncated: bool | None = error.body_truncated
        _ = (status, endpoint, body, body_truncated)
    except FmpValidationError as error:
        category: str | None = error.category
        _ = category
    except FmpError as error:
        base: FmpError = error
        top_level: fmp.FmpError = error
        _ = (base, top_level)


FmpClient(
    token="proxy-token",
    base_url="https://proxy.example",
    auth_mode="custom_header",
    auth_name="X-Proxy-Token",
    headers={"X-Tenant": "blue"},
    timeout=5.0,
    connect_timeout=2.0,
    max_response_body_bytes=67_108_864,
    danger_allow_insecure_authentication=False,
    follow_redirects=False,
)
