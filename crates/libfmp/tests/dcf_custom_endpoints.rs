mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        dcf::{
            CustomDcfQuery, DcfAssumptions, custom_discounted_cash_flow,
            custom_levered_discounted_cash_flow,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::dcf::{CustomDcfValuation, CustomLeveredDcfValuation},
    transport::HttpMethod,
    types::{FiniteDecimal, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const CUSTOM: &[u8] = include_bytes!("fixtures/custom_discounted_cash_flow.json");
const LEVERED: &[u8] = include_bytes!("fixtures/custom_levered_discounted_cash_flow.json");

fn decimal(value: f64) -> FiniteDecimal {
    FiniteDecimal::new(value).unwrap()
}

#[test]
fn descriptors_use_exact_get_paths_distinct_rows_and_worldwide_unbounded_metadata() {
    let query = CustomDcfQuery::new(
        Ticker::new("BRK.B / Class A").unwrap(),
        DcfAssumptions::new(),
    );
    assert_eq!(query.symbol().as_str(), "BRK.B / Class A");
    assert_eq!(query.assumptions(), &DcfAssumptions::default());

    let custom = custom_discounted_cash_flow(query.clone());
    let levered = custom_levered_discounted_cash_flow(query);
    assert_custom_facts(&custom, "custom-discounted-cash-flow");
    assert_levered_facts(&levered, "custom-levered-discounted-cash-flow");
}

fn assert_custom_facts(
    endpoint: &EndpointSpec<CustomDcfQuery, Vec<CustomDcfValuation>>,
    path: &'static str,
) {
    assert_facts(endpoint, path);
}

fn assert_levered_facts(
    endpoint: &EndpointSpec<CustomDcfQuery, Vec<CustomLeveredDcfValuation>>,
    path: &'static str,
) {
    assert_facts(endpoint, path);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[test]
fn all_eighteen_assumptions_are_independent_and_exposed_without_rescaling() {
    let assumptions = documented_assumptions();

    assert_eq!(
        assumptions.revenue_growth_pct(),
        Some(decimal(0.1094119804597946))
    );
    assert_eq!(assumptions.ebitda_pct(), Some(decimal(0.31273548388)));
    assert_eq!(
        assumptions.depreciation_and_amortization_pct(),
        Some(decimal(0.0345531631720999))
    );
    assert_eq!(
        assumptions.cash_and_short_term_investments_pct(),
        Some(decimal(0.2344222126801843))
    );
    assert_eq!(
        assumptions.receivables_pct(),
        Some(decimal(0.1533770531229388))
    );
    assert_eq!(
        assumptions.inventories_pct(),
        Some(decimal(0.0155245674227653))
    );
    assert_eq!(assumptions.payable_pct(), Some(decimal(0.1614868903169657)));
    assert_eq!(assumptions.ebit_pct(), Some(decimal(0.2781823207138459)));
    assert_eq!(
        assumptions.capital_expenditure_pct(),
        Some(decimal(0.0306025847141713))
    );
    assert_eq!(
        assumptions.operating_cash_flow_pct(),
        Some(decimal(0.2886333485760204))
    );
    assert_eq!(
        assumptions.selling_general_and_administrative_expenses_pct(),
        Some(decimal(0.0662854095187211))
    );
    assert_eq!(assumptions.tax_rate(), Some(decimal(0.14919579658453103)));
    assert_eq!(assumptions.long_term_growth_rate(), Some(decimal(4.0)));
    assert_eq!(assumptions.cost_of_debt(), Some(decimal(3.64)));
    assert_eq!(assumptions.cost_of_equity(), Some(decimal(9.51168)));
    assert_eq!(assumptions.market_risk_premium(), Some(decimal(4.72)));
    assert_eq!(assumptions.beta(), Some(decimal(1.244)));
    assert_eq!(assumptions.risk_free_rate(), Some(decimal(3.64)));
}

#[test]
fn finite_decimal_rejects_every_nonfinite_input_before_an_assumption_can_be_built() {
    assert!(FiniteDecimal::new(f64::NAN).is_err());
    assert!(FiniteDecimal::new(f64::INFINITY).is_err());
    assert!(FiniteDecimal::new(f64::NEG_INFINITY).is_err());
}

#[tokio::test]
async fn proxy_encodes_symbol_then_all_present_assumptions_in_documented_order() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CUSTOM),
        json_fixture(LEVERED),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "custom dcf")
        .executor(executor.clone())
        .build()
        .unwrap();

    let custom = client
        .custom_discounted_cash_flow(CustomDcfQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            documented_assumptions(),
        ))
        .await
        .unwrap();
    let levered = client
        .custom_levered_discounted_cash_flow(CustomDcfQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            documented_assumptions(),
        ))
        .await
        .unwrap();

    assert_eq!(custom[0].equity_value_per_share, 147.18);
    assert_eq!(levered[0].equity_value_per_share, 140.71);

    let suffix = "?symbol=BRK.B+%2F+Class+A&revenueGrowthPct=0.1094119804597946&ebitdaPct=0.31273548388&depreciationAndAmortizationPct=0.0345531631720999&cashAndShortTermInvestmentsPct=0.2344222126801843&receivablesPct=0.1533770531229388&inventoriesPct=0.0155245674227653&payablePct=0.1614868903169657&ebitPct=0.2781823207138459&capitalExpenditurePct=0.0306025847141713&operatingCashFlowPct=0.2886333485760204&sellingGeneralAndAdministrativeExpensesPct=0.0662854095187211&taxRate=0.14919579658453103&longTermGrowthRate=4&costOfDebt=3.64&costOfEquity=9.51168&marketRiskPremium=4.72&beta=1.244&riskFreeRate=3.64";
    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0].expose_url().as_str(),
        format!("https://proxy.example/router/gateway/stable/custom-discounted-cash-flow{suffix}")
    );
    assert_eq!(
        requests[1].expose_url().as_str(),
        format!(
            "https://proxy.example/router/gateway/stable/custom-levered-discounted-cash-flow{suffix}"
        )
    );
    for request in requests.iter() {
        assert_eq!(request.method(), HttpMethod::Get);
        assert_eq!(
            request.expose_headers()["x-router-token"],
            "Token proxy-secret"
        );
        assert_eq!(request.expose_headers()["x-data-scope"], "custom dcf");
    }
}

#[tokio::test]
async fn omitted_assumptions_stay_omitted_while_zero_and_negative_are_encoded_exactly() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CUSTOM),
        json_fixture(LEVERED),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();

    client
        .custom_discounted_cash_flow(CustomDcfQuery::new(
            Ticker::new("AAPL").unwrap(),
            DcfAssumptions::new(),
        ))
        .await
        .unwrap();
    client
        .custom_levered_discounted_cash_flow(CustomDcfQuery::new(
            Ticker::new("AAPL").unwrap(),
            DcfAssumptions::new()
                .with_revenue_growth_pct(decimal(0.0))
                .with_tax_rate(decimal(-1.25))
                .with_long_term_growth_rate(decimal(4.0)),
        ))
        .await
        .unwrap();

    let requests = executor.requests();
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/stable/custom-discounted-cash-flow?symbol=AAPL"
    );
    assert_eq!(
        requests[1].expose_url().as_str(),
        "https://proxy.example/stable/custom-levered-discounted-cash-flow?symbol=AAPL&revenueGrowthPct=0&taxRate=-1.25&longTermGrowthRate=4"
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_both_custom_paths() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(CUSTOM),
            json_fixture(LEVERED),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let query = || CustomDcfQuery::new(Ticker::new("AAPL").unwrap(), DcfAssumptions::new());

        client.custom_discounted_cash_flow(query()).await.unwrap();
        client
            .custom_levered_discounted_cash_flow(query())
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/custom-discounted-cash-flow?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/custom-levered-discounted-cash-flow?symbol=AAPL{suffix}"
                ),
            ]
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_distinct_custom_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let query = || CustomDcfQuery::new(Ticker::new("AAPL").unwrap(), DcfAssumptions::new());

    let errors = [
        client
            .custom_discounted_cash_flow(query())
            .await
            .unwrap_err(),
        client
            .custom_levered_discounted_cash_flow(query())
            .await
            .unwrap_err(),
    ];
    for (error, id) in errors.iter().zip([
        "custom-discounted-cash-flow",
        "custom-levered-discounted-cash-flow",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn documented_assumptions() -> DcfAssumptions {
    DcfAssumptions::new()
        .with_revenue_growth_pct(decimal(0.1094119804597946))
        .with_ebitda_pct(decimal(0.31273548388))
        .with_depreciation_and_amortization_pct(decimal(0.0345531631720999))
        .with_cash_and_short_term_investments_pct(decimal(0.2344222126801843))
        .with_receivables_pct(decimal(0.1533770531229388))
        .with_inventories_pct(decimal(0.0155245674227653))
        .with_payable_pct(decimal(0.1614868903169657))
        .with_ebit_pct(decimal(0.2781823207138459))
        .with_capital_expenditure_pct(decimal(0.0306025847141713))
        .with_operating_cash_flow_pct(decimal(0.2886333485760204))
        .with_selling_general_and_administrative_expenses_pct(decimal(0.0662854095187211))
        .with_tax_rate(decimal(0.14919579658453103))
        .with_long_term_growth_rate(decimal(4.0))
        .with_cost_of_debt(decimal(3.64))
        .with_cost_of_equity(decimal(9.51168))
        .with_market_risk_premium(decimal(4.72))
        .with_beta(decimal(1.244))
        .with_risk_free_rate(decimal(3.64))
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
