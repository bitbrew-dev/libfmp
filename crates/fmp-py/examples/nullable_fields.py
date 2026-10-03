"""Nullable fields: members the provider may omit or send as null are ``Optional``.

Since 1.2.0 a member that FMP really leaves out for some rows (for example
``CompanyProfile.isin`` or ``country``) is typed ``X | None`` instead of failing
the whole decode, so guard it before use. Open-ended extra members land in a
plain ``dict``. Run with ``uv run nullable_fields.py``; needs ``FMP_API_KEY``.
"""

# fmp library
from fmp import FmpClient


def company_profile(client: FmpClient) -> None:
    """Reads optional profile members with an explicit ``None`` check."""
    for symbol in ["AAPL", "SPY"]:
        profile = client.company.profile(symbol)[0]
        isin = profile.isin if profile.isin is not None else "n/a"
        country = profile.country or "unknown"
        employees = profile.full_time_employees
        print(f"{profile.symbol}: isin={isin} country={country} employees={employees} ipo={profile.ipo_date}")


def congressional_aggregate(client: FmpClient) -> None:
    """Sums optional net-worth columns, then lists the open-ended extra columns."""
    member = client.congressional.profiles(active=True, limit=1)[0]
    print(f"member {member.first_name} {member.last_name} ({member.latest_position})")
    for row in client.congressional.net_worth_aggregated(member.member_id)[:3]:
        # cash_and_cash_equivalents and mutual_funds_and_etfs are Optional since 1.3.0:
        # a None means "not disclosed", not zero, so do not coerce it silently.
        liquid = [value for value in (row.cash_and_cash_equivalents, row.mutual_funds_and_etfs) if value is not None]
        cash = row.cash_and_cash_equivalents
        print(f"  {row.year}: total={row.total:,.0f} cash={cash} liquid_known={sum(liquid):,.0f}")
        # additional_columns keeps any member the model does not name, keyed by
        # the raw provider name; it is always a dict (possibly empty).
        if row.additional_columns:
            print(f"    extra columns: {sorted(row.additional_columns)}")


def main() -> None:
    """Runs both nullable-field walkthroughs."""
    client = FmpClient()
    company_profile(client)
    congressional_aggregate(client)


if __name__ == "__main__":
    main()
