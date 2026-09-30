mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        economics::{
            EconomicIndicatorsQuery, TreasuryRatesQuery, economic_indicators, treasury_rates,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    query::EconomicIndicator,
    transport::HttpMethod,
    types::{Date, DateRange},
};

use support::{FixtureExecutor, json_fixture};

const TREASURY: &[u8] = include_bytes!("fixtures/treasury_rates.json");
const INDICATORS: &[u8] = include_bytes!("fixtures/economic_indicators.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_the_90_day_bound() {
    assert_facts(&treasury_rates(TreasuryRatesQuery::new()), "treasury-rates");
    assert_facts(
        &economic_indicators(EconomicIndicator::Gdp.into()),
        "economic-indicators",
    );

    let bounds = EndpointBounds::new().with_date_range_days(90);
    let range_90_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-26").unwrap(),
    )
    .unwrap();
    let range_91_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-27").unwrap(),
    )
    .unwrap();
    let contradictory_sample = DateRange::new(
        Date::from_str("2025-04-27").unwrap(),
        Date::from_str("2026-04-27").unwrap(),
    )
    .unwrap();
    assert!(bounds.accepts_date_range(&range_90_days));
    assert!(!bounds.accepts_date_range(&range_91_days));
    assert!(!bounds.accepts_date_range(&contradictory_sample));
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_date_range_days(90)
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(TREASURY),
        json_fixture(INDICATORS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "rates-indicators")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-27").unwrap();

    let rates = client
        .treasury_rates(TreasuryRatesQuery::new().with_from(from).with_to(to))
        .await
        .unwrap();
    let indicators = client
        .economic_indicators(
            EconomicIndicatorsQuery::new(EconomicIndicator::Gdp)
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();

    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].year_30, Some(5.2));
    assert_eq!(indicators.len(), 1);
    assert_eq!(indicators[0].value, Some(31_422.526));

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "rates-indicators"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/treasury-rates?from=2026-01-27&to=2026-04-27",
            "https://proxy.example/router/stable/economic-indicators?name=GDP&from=2026-01-27&to=2026-04-27",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_independent_date_omission() {
    for (authentication, query_suffix, expected_header) in [
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
            json_fixture(TREASURY),
            json_fixture(TREASURY),
            json_fixture(INDICATORS),
            json_fixture(INDICATORS),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-27").unwrap();

        client
            .treasury_rates(TreasuryRatesQuery::new().with_from(from))
            .await
            .unwrap();
        client
            .treasury_rates(TreasuryRatesQuery::new().with_to(to))
            .await
            .unwrap();
        client
            .economic_indicators(
                EconomicIndicatorsQuery::new(EconomicIndicator::Gdp).with_from(from),
            )
            .await
            .unwrap();
        client
            .economic_indicators(EconomicIndicatorsQuery::new(EconomicIndicator::Gdp).with_to(to))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/treasury-rates?from=2026-01-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/treasury-rates?to=2026-04-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-indicators?name=GDP&from=2026-01-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-indicators?name=GDP&to=2026-04-27{query_suffix}"
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
async fn every_documented_indicator_and_other_reach_the_exact_wire_query() {
    let documented = [
        "GDP",
        "realGDP",
        "nominalPotentialGDP",
        "realGDPPerCapita",
        "federalFunds",
        "CPI",
        "inflationRate",
        "inflation",
        "retailSales",
        "consumerSentiment",
        "durableGoods",
        "unemploymentRate",
        "totalNonfarmPayroll",
        "initialClaims",
        "industrialProductionTotalIndex",
        "newPrivatelyOwnedHousingUnitsStartedTotalUnits",
        "totalVehicleSales",
        "retailMoneyFunds",
        "smoothedUSRecessionProbabilities",
        "3MonthOr90DayRatesAndYieldsCertificatesOfDeposit",
        "commercialBankInterestRateOnCreditCardPlansAllAccounts",
        "30YearFixedRateMortgageAverage",
        "15YearFixedRateMortgageAverage",
        "tradeBalanceGoodsAndServices",
    ];
    let indicators = EconomicIndicator::DOCUMENTED
        .into_iter()
        .chain([EconomicIndicator::new("futureProviderIndicator").unwrap()])
        .collect::<Vec<_>>();
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(INDICATORS)).take(indicators.len()),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("indicator-secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    for indicator in indicators {
        assert_eq!(
            client.economic_indicators(indicator).await.unwrap().len(),
            1
        );
    }

    let expected = documented
        .into_iter()
        .chain(["futureProviderIndicator"])
        .map(|name| {
            format!("https://financialmodelingprep.com/stable/economic-indicators?name={name}")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        executor
            .requests()
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        expected
    );
}
