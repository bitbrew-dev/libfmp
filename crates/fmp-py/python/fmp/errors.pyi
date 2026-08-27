from typing import Optional

class FmpError(Exception):
    """Base exception for all fmp failures."""

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

__all__: list[str]
