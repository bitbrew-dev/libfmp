mod support;

use std::sync::Arc;

use http::{
    HeaderMap, HeaderValue,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        BinaryResponse, EndpointSpec, ExpectedContentType,
        metadata::EndpointMetadata,
        statements::{
            financial_reports_dates, financial_reports_json, financial_reports_xlsx,
            reports::{
                FINANCIAL_REPORTS_XLSX_CONTENT_TYPES, FinancialReportsDatesQuery,
                FinancialReportsJsonQuery, FinancialReportsXlsxQuery,
            },
        },
    },
    error::{ErrorCategory, MAX_SAFE_BODY_BYTES},
    query::{FiscalPeriod, Year},
    transport::{HttpMethod, TransportResponse},
    types::Ticker,
};

use support::{FixtureExecutor, fixture_response, json_fixture};

const DATES: &[u8] = include_bytes!("fixtures/financial_reports_dates.json");
const REPORT: &[u8] = include_bytes!("fixtures/financial_reports_json.json");
const XLSX_BYTES: &[u8] = b"opaque workbook bytes: not ZIP validated";
const OFFICIAL_XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

fn binary_fixture(
    content_type: Option<&str>,
    disposition: Option<&str>,
    body: impl Into<Vec<u8>>,
) -> TransportResponse {
    let mut headers = HeaderMap::new();
    if let Some(content_type) = content_type {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type).expect("valid fixture content type"),
        );
    }
    if let Some(disposition) = disposition {
        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(disposition).expect("valid fixture disposition"),
        );
    }
    TransportResponse::new(200, headers, body)
}

#[test]
fn descriptors_use_exact_paths_response_contracts_and_unspecified_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let dates = financial_reports_dates(FinancialReportsDatesQuery::new(symbol.clone()));
    let json = financial_reports_json(FinancialReportsJsonQuery::new(
        symbol.clone(),
        Year(2022),
        FiscalPeriod::FullYear,
    ));
    let xlsx = financial_reports_xlsx(FinancialReportsXlsxQuery::new(
        symbol,
        Year(2022),
        FiscalPeriod::FullYear,
    ));

    assert_facts(&dates, "financial-reports-dates");
    assert_facts(&json, "financial-reports-json");
    assert_facts(&xlsx, "financial-reports-xlsx");
    assert_eq!(
        dates.response().expected_content_type(),
        ExpectedContentType::Json
    );
    assert_eq!(
        json.response().expected_content_type(),
        ExpectedContentType::Json
    );
    assert_eq!(
        xlsx.response().expected_content_type(),
        ExpectedContentType::Binary(FINANCIAL_REPORTS_XLSX_CONTENT_TYPES)
    );
    assert_eq!(
        FINANCIAL_REPORTS_XLSX_CONTENT_TYPES,
        [OFFICIAL_XLSX, "application/octet-stream"]
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata(), EndpointMetadata::new());
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_auth_headers_and_all_three_response_shapes() {
    const DISPOSITION: &str = "attachment; filename=provider-report.xlsx";
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(DATES),
        json_fixture(REPORT),
        binary_fixture(Some(OFFICIAL_XLSX), Some(DISPOSITION), XLSX_BYTES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-report-scope", "annual-filings")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let dates = client.financial_reports_dates(&symbol).await.unwrap();
    let report = client
        .financial_reports_json(FinancialReportsJsonQuery::new(
            symbol.clone(),
            Year(2022),
            FiscalPeriod::Q3,
        ))
        .await
        .unwrap();
    let xlsx = client
        .financial_reports_xlsx(FinancialReportsXlsxQuery::new(
            symbol,
            Year(2022),
            FiscalPeriod::FullYear,
        ))
        .await
        .unwrap();

    assert_eq!(dates[0].fiscal_year.get(), 2026);
    assert_eq!(report[0].year.as_str(), "2022");
    assert_eq!(report[0].sections.len(), 67);
    assert_eq!(xlsx.as_bytes(), XLSX_BYTES);
    assert_eq!(xlsx.content_type(), OFFICIAL_XLSX);
    assert_eq!(xlsx.content_disposition(), Some(DISPOSITION));

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-report-scope"] == "annual-filings"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/financial-reports-dates?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/financial-reports-json?symbol=BRK.B+%2F+Class+A&year=2022&period=Q3",
            "https://proxy.example/router/stable/financial-reports-xlsx?symbol=BRK.B+%2F+Class+A&year=2022&period=FY",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_all_three_report_endpoints() {
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
            json_fixture(DATES),
            json_fixture(REPORT),
            binary_fixture(Some("application/octet-stream"), None, XLSX_BYTES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client.financial_reports_dates(&symbol).await.unwrap();
        client
            .financial_reports_json(FinancialReportsJsonQuery::new(
                symbol.clone(),
                Year(2022),
                FiscalPeriod::Q1,
            ))
            .await
            .unwrap();
        client
            .financial_reports_xlsx(FinancialReportsXlsxQuery::new(
                symbol,
                Year(2022),
                FiscalPeriod::Q4,
            ))
            .await
            .unwrap();

        let requests = executor.requests();
        let expected = [
            format!(
                "https://financialmodelingprep.com/stable/financial-reports-dates?symbol=AAPL{suffix}"
            ),
            format!(
                "https://financialmodelingprep.com/stable/financial-reports-json?symbol=AAPL&year=2022&period=Q1{suffix}"
            ),
            format!(
                "https://financialmodelingprep.com/stable/financial-reports-xlsx?symbol=AAPL&year=2022&period=Q4{suffix}"
            ),
        ];
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            expected
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
async fn xlsx_policy_accepts_only_official_and_proxy_mimes_without_zip_validation() {
    const PARAMETERIZED_OFFICIAL: &str =
        "Application/Vnd.Openxmlformats-Officedocument.Spreadsheetml.Sheet; source=provider";
    let executor = Arc::new(FixtureExecutor::new([
        binary_fixture(
            Some(PARAMETERIZED_OFFICIAL),
            Some("attachment; filename=first.xlsx"),
            b"plain bytes under official MIME".as_slice(),
        ),
        binary_fixture(
            Some("application/octet-stream"),
            None,
            b"proxy bytes without ZIP magic".as_slice(),
        ),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let query = || {
        FinancialReportsXlsxQuery::new(
            Ticker::new("AAPL").unwrap(),
            Year(2022),
            FiscalPeriod::FullYear,
        )
    };
    let official: BinaryResponse = client.financial_reports_xlsx(query()).await.unwrap();
    let proxy: BinaryResponse = client.financial_reports_xlsx(query()).await.unwrap();

    assert_eq!(official.as_bytes(), b"plain bytes under official MIME");
    assert_eq!(official.content_type(), PARAMETERIZED_OFFICIAL);
    assert_eq!(
        official.content_disposition(),
        Some("attachment; filename=first.xlsx")
    );
    assert_eq!(proxy.as_bytes(), b"proxy bytes without ZIP magic");
    assert_eq!(proxy.content_type(), "application/octet-stream");
    assert_eq!(proxy.content_disposition(), None);
}

#[tokio::test]
async fn xlsx_policy_rejects_json_text_and_missing_content_type() {
    for response in [
        fixture_response(Some("application/json"), b"{}"),
        fixture_response(Some("text/plain"), b"not a workbook"),
        fixture_response(None, b"untyped bytes"),
    ] {
        let executor = Arc::new(FixtureExecutor::new([response]));
        let client = Client::builder()
            .authentication(Authentication::fmp_header("secret"))
            .executor(executor)
            .build()
            .unwrap();
        let error = client
            .financial_reports_xlsx(FinancialReportsXlsxQuery::new(
                Ticker::new("AAPL").unwrap(),
                Year(2022),
                FiscalPeriod::FullYear,
            ))
            .await
            .unwrap_err();

        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some("financial-reports-xlsx"));
        assert_eq!(error.status_code(), Some(200));
    }
}

#[tokio::test]
async fn rejected_xlsx_diagnostics_are_bounded_and_hide_tokens_and_links() {
    const AUTH_SECRET: &str = "router-auth-secret";
    const BODY_SECRET: &str = "provider-link-secret";
    let body = format!(
        "{{\"link\":\"https://provider.example/reports/private.xlsx?apikey={BODY_SECRET}\",\"detail\":\"{AUTH_SECRET}{}\"}}",
        "界".repeat(MAX_SAFE_BODY_BYTES)
    );
    let executor = Arc::new(FixtureExecutor::new([binary_fixture(
        Some("application/json"),
        None,
        body,
    )]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_query("router_token", AUTH_SECRET))
        .executor(executor)
        .build()
        .unwrap();

    let error = client
        .financial_reports_xlsx(FinancialReportsXlsxQuery::new(
            Ticker::new("AAPL").unwrap(),
            Year(2022),
            FiscalPeriod::FullYear,
        ))
        .await
        .unwrap_err();
    let safe_body = error.body().unwrap();
    let diagnostic = format!("{error:?} {error}");

    assert_eq!(error.category(), ErrorCategory::Decode);
    assert!(safe_body.is_truncated());
    assert!(safe_body.as_str().len() <= MAX_SAFE_BODY_BYTES);
    assert!(!safe_body.as_str().contains(BODY_SECRET));
    assert!(!diagnostic.contains(AUTH_SECRET));
    assert!(!diagnostic.contains(BODY_SECRET));
}
