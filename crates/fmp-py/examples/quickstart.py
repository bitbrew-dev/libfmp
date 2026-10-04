"""Quickstart: a quote, an income statement, and a DCF valuation for one symbol.

Run with ``uv run quickstart.py``; ``FMP_API_KEY`` must be set in the environment.
"""

# fmp library
from fmp import FmpClient

SYMBOL = "AAPL"


def main() -> None:
    """Fetches and prints a few typed rows."""
    # No token= argument: the client reads FMP_API_KEY from the environment.
    client = FmpClient()

    quote = client.quote.short(SYMBOL)[0]
    print(f"{quote.symbol}: price={quote.price} change={quote.change} volume={quote.volume}")

    # Optional arguments are keyword-only; period is a typing.Literal in the stubs.
    for row in client.statements.income.statement(SYMBOL, period="annual", limit=3):
        print(f"{row.fiscal_year} {row.period}: revenue={row.revenue:,.0f} net_income={row.net_income:,.0f}")

    valuation = client.dcf.standard(SYMBOL)[0]
    print(f"DCF {valuation.date}: dcf={valuation.dcf:.2f} vs stock_price={valuation.stock_price:.2f}")


if __name__ == "__main__":
    main()
