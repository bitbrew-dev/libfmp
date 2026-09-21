#!/usr/bin/env python3
"""Generate docs/endpoint-coverage.md from the fmp-py-gen endpoint registry.

The registry under crates/fmp-py-gen/registry/ names every Python method, the
libfmp Client method behind it, and its response shape. registry_check proves
each entry against the real Rust signatures, so the registry is the shared
source of truth for both the Rust and the Python coverage table.

Usage (from the repository root, Python 3.11 or newer for tomllib):

    python3 scripts/gen-endpoint-coverage.py --oracle artifacts/documentations/outer.md

The oracle is the captured provider documentation. It is gitignored, so pass
its path explicitly; without it the documented-entry column reads "n/a".
The script rewrites docs/endpoint-coverage.md unless --output names another
file, and a second run on an unchanged tree leaves git status clean.
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_go_coverage as go_coverage

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "crates" / "fmp-py-gen" / "registry"
DEFAULT_OUTPUT = ROOT / "docs" / "endpoint-coverage.md"

DOMAIN_ORDER = [
    "search", "directory", "company", "screener", "quote", "statements", "chart",
    "economics", "calendar", "transcripts", "news", "institutional_ownership",
    "analyst", "market", "technical_indicators", "funds", "sec_filings",
    "insider_trading", "indexes", "market_hours", "commodities", "dcf", "forex",
    "crypto", "congressional", "esg", "commitment_of_traders", "fundraising",
    "bulk", "tipranks",
]

ORACLE_SECTION = {
    "search": "Search", "directory": "Directory", "company": "Company",
    "screener": "Search", "quote": "Quote", "statements": "Statements",
    "chart": "Chart", "economics": "Economics", "calendar": "Calendar",
    "transcripts": "Earnings Transcript", "news": "News",
    "institutional_ownership": "Form 13F", "analyst": "Analyst",
    "market": "Market Performance", "technical_indicators": "Technical Indicators",
    "funds": "ETF & Mutual Funds", "sec_filings": "SEC Filings",
    "insider_trading": "Insider Trades", "indexes": "Indexes",
    "market_hours": "Market Hours", "commodities": "Commodity",
    "dcf": "Discounted Cash Flow", "forex": "Forex", "crypto": "Crypto",
    "congressional": "Senate", "esg": "ESG",
    "commitment_of_traders": "Commitment Of Traders",
    "fundraising": "Fundraisers", "bulk": "Bulk", "tipranks": "TipRanks",
}

CROSS_LISTED = [
    ("Crypto / All Cryptocurrencies Quotes", "quote.cryptocurrencies"),
    ("Forex / Batch Forex Quotes", "quote.forex"),
    ("Commodity / All Commodities Quotes", "quote.commodities"),
    ("Indexes / All Index Quotes", "quote.indexes"),
    ("Earnings Transcript / Available Transcript Symbols", "directory.earnings_transcript_list"),
]

SHORT_ONLY = "fixed `short=true`; the undocumented `short=false` universe is not exposed"
TIPRANKS = "requires the TipRanks add-on (`AccessRequirement::NamedAddOn`)"
ANNOTATIONS: dict[str, tuple[str, str]] = {
    "sec_filings.search_industry_classifications": (
        "raw", "documented as `[{}]`; Rust `Vec<DynamicObject>`, Python `list[dict]`"),
    "quote.mutual_funds": ("gated", SHORT_ONLY),
    "quote.etfs": ("gated", SHORT_ONLY),
    "quote.commodities": ("gated", SHORT_ONLY),
    "quote.cryptocurrencies": ("gated", SHORT_ONLY),
    "quote.forex": ("gated", SHORT_ONLY),
    "quote.indexes": ("gated", SHORT_ONLY),
    "tipranks.ratings_search": ("gated", TIPRANKS + "; Enterprise plan for history older than 3 years"),
    "tipranks.point_in_time_ratings_by_symbol": ("gated", TIPRANKS),
    "tipranks.point_in_time_ratings_by_analyst": ("gated", TIPRANKS),
    "tipranks.symbol_summary": ("gated", TIPRANKS),
    "tipranks.analyst_summary": ("gated", TIPRANKS),
    "tipranks.firm_summary": ("gated", TIPRANKS),
    "tipranks.analysts": ("gated", TIPRANKS),
    "economics.calendar": (
        "deferred", "`previous`/`estimate`/`actual` are documented non-null `f64`; live nulls tracked in #40"),
    "bulk.etf_holdings": (
        "deferred", "documented key `lastUpdated\"` is mirrored verbatim; upstream fix tracked in #40"),
}
REALTIME = "Nasdaq data is 15-minute delayed unless the real-time user declaration is on file"
NOTES: dict[str, str] = {
    "statements.reports.json": "typed row whose `sections` are dynamic JSON",
    "statements.reports.xlsx": "binary payload (`BinaryResponse` / `fmp.BinaryPayload`)",
    "statements.reports.dates": "rows carry secret download links; redacted in Python `repr`",
    "quote.full": REALTIME,
    "quote.short": REALTIME,
    "quote.aftermarket_trade": REALTIME,
    "quote.aftermarket_quote": REALTIME,
    "quote.batch_quote": REALTIME,
    "quote.batch_quote_short": REALTIME,
    "quote.batch_aftermarket_trade": REALTIME,
    "quote.batch_aftermarket_quote": REALTIME,
    "quote.exchange": REALTIME,
}


@dataclass(frozen=True)
class Entry:
    domain: str
    path: str
    name: str
    method: str
    dynamic: bool
    binary: bool

    @property
    def key(self) -> str:
        return f"{self.path}.{self.name}"

    @property
    def state(self) -> str:
        if self.key in ANNOTATIONS:
            return ANNOTATIONS[self.key][0]
        return "raw" if self.dynamic else "supported"

    @property
    def note(self) -> str:
        if self.key in ANNOTATIONS:
            return ANNOTATIONS[self.key][1]
        return NOTES.get(self.key, "")

    @property
    def go_key(self) -> tuple[str, str]:
        """`(namespace struct, method)` the gen_go emitter produces for this entry."""
        return (go_coverage.struct_name(self.path.split(".")), go_coverage.exported(self.name))


def go_methods() -> set[tuple[str, str]]:
    """The namespace methods declared in sdk/go, keyed like `Entry.go_key`."""
    return {m.go_key for m in go_coverage.load_actual()}


def load_entries(domain: str) -> list[Entry]:
    data = tomllib.loads((REGISTRY / f"{domain}.toml").read_text())
    groups = [(domain, data.get("endpoint", []))]
    groups += [(ns["path"], ns.get("endpoint", [])) for ns in data.get("namespace", [])]
    return [
        Entry(domain, path, e["name"], e["method"],
              e.get("response") == "dynamic", bool(e.get("binary", False)))
        for path, endpoints in groups
        for e in endpoints
    ]


def oracle_counts(path: Path | None) -> Counter[str] | None:
    if path is None:
        return None
    counts: Counter[str] = Counter()
    section = None
    for line in path.read_text().splitlines():
        if line.startswith("## "):
            section = line[3:].strip()
        elif line.startswith("### ") and section:
            counts[section] += 1
    return counts


def render(entries: list[Entry], oracle: Counter[str] | None, go: set[tuple[str, str]]) -> str:
    by_domain = {d: [e for e in entries if e.domain == d] for d in DOMAIN_ORDER}
    states = Counter(e.state for e in entries)
    total_doc = sum(oracle.values()) if oracle else None
    go_total = sum(e.go_key in go for e in entries)
    go_namespaces = len(go_coverage.generated_domains(DOMAIN_ORDER))
    out: list[str] = []
    out += [
        "# Endpoint coverage",
        "",
        "Generated by `scripts/gen-endpoint-coverage.py` from `crates/fmp-py-gen/registry/`. Do not edit by hand.",
        "",
        "This table records the coverage the `libfmp` crate and the `fmp-py-sdk` wheel ship for every endpoint",
        "entry in the captured FMP documentation oracle (`artifacts/documentations/outer.md`). It is the",
        "release deliverable planned for v0.2.0 (#41); that version was superseded before the table landed, and",
        "the coverage below has shipped through the 0.2.0 .. 0.7.0 releases (crates.io `libfmp` and PyPI",
        "`fmp-py-sdk` share the Cargo workspace version).",
        "",
        f"- Documented oracle entries: {total_doc if total_doc is not None else 'n/a'}",
        f"- Rust `Client` methods: {len(entries)} (`cargo run -p fmp-py-gen --bin registry_check` verifies each one)",
        f"- Python methods: {len(entries)} across {len(DOMAIN_ORDER)} namespaces",
        f"- Go methods: {go_total} across {go_namespaces} namespaces (`python3 scripts/check_go_coverage.py` audits each one)",
        "- States: " + ", ".join(f"{s} {states[s]}" for s in ("supported", "raw", "gated", "deferred")),
        "",
        "| State | Meaning |",
        "| --- | --- |",
        "| supported | typed request and typed response rows in Rust and Python |",
        "| raw | the documented response is open-ended, so rows stay dynamic JSON |",
        "| gated | reachable only within the documented entitlement or a fixed query variant |",
        "| deferred | a documented ambiguity is mirrored as written; the upstream contract gap is tracked in #40 |",
        "",
        "## Domains",
        "",
        "| Domain | Documented section | Documented entries | Rust methods | Python methods | Go methods | States |",
        "| --- | --- | --- | --- | --- | --- | --- |",
    ]
    for domain in DOMAIN_ORDER:
        rows = by_domain[domain]
        section = ORACLE_SECTION[domain]
        if oracle is None:
            documented = "n/a"
        elif domain == "search":
            documented = f"{oracle[section]} (includes screener)"
        elif domain == "screener":
            documented = "counted under Search"
        else:
            documented = str(oracle[section])
        domain_states = Counter(e.state for e in rows)
        state_text = ", ".join(f"{s} {n}" for s, n in sorted(domain_states.items()))
        go_rows = sum(e.go_key in go for e in rows)
        out.append(f"| `{domain}` | {section} | {documented} | {len(rows)} | {len(rows)} | {go_rows} | {state_text} |")
    out += [
        "",
        f"The oracle lists {total_doc or 276} entries while the registry names {len(entries)} methods because the Search section holds",
        "the screener entry (`screener.companies`) and five entries are documented twice. Each duplicate is",
        "served by the single method that owns the path:",
        "",
    ]
    out += [f"- {heading}: `{method}`" for heading, method in CROSS_LISTED]
    out += [
        "",
        "## Endpoints",
        "",
        "| Domain | Python method | Rust `Client` method | State | Go | Notes |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for domain in DOMAIN_ORDER:
        for e in by_domain[domain]:
            go_state = "supported" if e.go_key in go else "pending"
            out.append(f"| `{domain}` | `{e.key}` | `{e.method}` | {e.state} | {go_state} | {e.note} |")
    return "\n".join(out).rstrip() + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--oracle", type=Path, help="path to the captured documentation oracle")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args(argv)
    if args.oracle is not None and not args.oracle.is_file():
        parser.error(f"oracle not found: {args.oracle}")
    entries = [e for d in DOMAIN_ORDER for e in load_entries(d)]
    unknown = sorted(set(ANNOTATIONS) | set(NOTES))
    unknown = [k for k in unknown if k not in {e.key for e in entries}]
    if unknown:
        parser.error("annotation names unknown registry entries: " + ", ".join(unknown))
    args.output.write_text(render(entries, oracle_counts(args.oracle), go_methods()))
    print(f"wrote {args.output}: {len(entries)} methods", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
