use libfmp::query::{
    ChartTimeframe, EconomicIndicator, FiscalPeriod, PeriodLength, Quarter, RetrievalFrequency,
    StatementPeriod, Year,
};

#[test]
fn shared_query_vocabulary_has_exact_documented_wire_forms() {
    let fiscal = [
        (FiscalPeriod::Q1, "Q1"),
        (FiscalPeriod::Q2, "Q2"),
        (FiscalPeriod::Q3, "Q3"),
        (FiscalPeriod::Q4, "Q4"),
        (FiscalPeriod::FullYear, "FY"),
    ];
    let frequencies = [
        (RetrievalFrequency::Annual, "annual"),
        (RetrievalFrequency::Quarterly, "quarter"),
    ];
    assert_eq!(
        fiscal.map(|(value, _)| value.to_string()),
        ["Q1", "Q2", "Q3", "Q4", "FY"]
    );
    assert_eq!(
        frequencies.map(|(value, _)| value.to_string()),
        ["annual", "quarter"]
    );

    let statement_periods = fiscal
        .into_iter()
        .map(|(value, _)| StatementPeriod::from(value))
        .chain(
            frequencies
                .into_iter()
                .map(|(value, _)| StatementPeriod::from(value)),
        )
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        statement_periods,
        ["Q1", "Q2", "Q3", "Q4", "FY", "annual", "quarter"]
    );

    assert_eq!(
        [Quarter::Q1, Quarter::Q2, Quarter::Q3, Quarter::Q4].map(|value| value.to_string()),
        ["1", "2", "3", "4"]
    );
    assert_eq!(
        [
            ChartTimeframe::OneMinute,
            ChartTimeframe::FiveMinutes,
            ChartTimeframe::FifteenMinutes,
            ChartTimeframe::ThirtyMinutes,
            ChartTimeframe::OneHour,
            ChartTimeframe::FourHours,
            ChartTimeframe::OneDay,
        ]
        .map(|value| value.to_string()),
        ["1min", "5min", "15min", "30min", "1hour", "4hour", "1day"]
    );
    assert_eq!(Year(2026).to_string(), "2026");
    assert_eq!(PeriodLength::new(10).unwrap().to_string(), "10");
    assert!(PeriodLength::new(0).is_none());
}

#[test]
fn all_24_documented_economic_indicators_are_explicit_and_other_is_validated() {
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

    assert_eq!(EconomicIndicator::DOCUMENTED.len(), 24);
    assert_eq!(
        EconomicIndicator::DOCUMENTED
            .iter()
            .map(EconomicIndicator::as_str)
            .collect::<Vec<_>>(),
        documented
    );
    assert!(
        EconomicIndicator::DOCUMENTED
            .iter()
            .all(|value| !matches!(value, EconomicIndicator::Other(_)))
    );
    assert_eq!(
        EconomicIndicator::new("futureProviderIndicator")
            .unwrap()
            .to_string(),
        "futureProviderIndicator"
    );
    assert!(EconomicIndicator::new("").is_err());
    assert!(EconomicIndicator::new("bad\nindicator").is_err());
}
