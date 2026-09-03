//! Shared query vocabulary with exact provider wire spellings.

use std::{fmt, num::NonZeroU32, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::types::StringValueError;

macro_rules! wire_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident { $($variant:ident => $wire:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// Returns the exact provider query representation.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                match value.as_str() {
                    $($wire => Ok(Self::$variant),)+
                    _ => Err(de::Error::unknown_variant(&value, &[$($wire),+])),
                }
            }
        }
    };
}

wire_enum! {
    /// A fiscal reporting period, excluding retrieval-frequency aliases.
    pub enum FiscalPeriod {
        Q1 => "Q1",
        Q2 => "Q2",
        Q3 => "Q3",
        Q4 => "Q4",
        FullYear => "FY",
    }
}

wire_enum! {
    /// The annual or quarterly retrieval frequency used by segmentation and as-reported APIs.
    pub enum RetrievalFrequency {
        Annual => "annual",
        Quarterly => "quarter",
    }
}

/// The explicit seven-value union documented by standard statement endpoints.
///
/// The composed variants preserve the distinction between a fiscal-period
/// selector and a retrieval frequency. The narrower component enums should be
/// preferred by endpoints that document only one family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StatementPeriod {
    Fiscal(FiscalPeriod),
    Frequency(RetrievalFrequency),
}

impl fmt::Display for StatementPeriod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fiscal(value) => value.fmt(formatter),
            Self::Frequency(value) => value.fmt(formatter),
        }
    }
}

impl From<FiscalPeriod> for StatementPeriod {
    fn from(value: FiscalPeriod) -> Self {
        Self::Fiscal(value)
    }
}

impl From<RetrievalFrequency> for StatementPeriod {
    fn from(value: RetrievalFrequency) -> Self {
        Self::Frequency(value)
    }
}

impl Serialize for StatementPeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for StatementPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "Q1" => Ok(FiscalPeriod::Q1.into()),
            "Q2" => Ok(FiscalPeriod::Q2.into()),
            "Q3" => Ok(FiscalPeriod::Q3.into()),
            "Q4" => Ok(FiscalPeriod::Q4.into()),
            "FY" => Ok(FiscalPeriod::FullYear.into()),
            "annual" => Ok(RetrievalFrequency::Annual.into()),
            "quarter" => Ok(RetrievalFrequency::Quarterly.into()),
            _ => Err(de::Error::unknown_variant(
                &value,
                &["Q1", "Q2", "Q3", "Q4", "FY", "annual", "quarter"],
            )),
        }
    }
}

wire_enum! {
    /// A numeric calendar quarter used by endpoints whose wire values are `1` through `4`.
    pub enum Quarter {
        Q1 => "1",
        Q2 => "2",
        Q3 => "3",
        Q4 => "4",
    }
}

wire_enum! {
    /// A timeframe accepted by the technical-indicator chart endpoints.
    pub enum ChartTimeframe {
        OneMinute => "1min",
        FiveMinutes => "5min",
        FifteenMinutes => "15min",
        ThirtyMinutes => "30min",
        OneHour => "1hour",
        FourHours => "4hour",
        OneDay => "1day",
    }
}

/// An economic-indicator name accepted by the economic indicators endpoint.
///
/// The documented names are represented explicitly. [`Self::Other`] preserves
/// forward compatibility without allowing empty or control-containing values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EconomicIndicator {
    Gdp,
    RealGdp,
    NominalPotentialGdp,
    RealGdpPerCapita,
    FederalFunds,
    Cpi,
    InflationRate,
    Inflation,
    RetailSales,
    ConsumerSentiment,
    DurableGoods,
    UnemploymentRate,
    TotalNonfarmPayroll,
    InitialClaims,
    IndustrialProductionTotalIndex,
    NewPrivatelyOwnedHousingUnitsStartedTotalUnits,
    TotalVehicleSales,
    RetailMoneyFunds,
    SmoothedUsRecessionProbabilities,
    ThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit,
    CommercialBankInterestRateOnCreditCardPlansAllAccounts,
    ThirtyYearFixedRateMortgageAverage,
    FifteenYearFixedRateMortgageAverage,
    TradeBalanceGoodsAndServices,
    Other(OpenEconomicIndicator),
}

/// A validated forward-compatible economic-indicator name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpenEconomicIndicator(String);

impl OpenEconomicIndicator {
    pub fn new(value: impl Into<String>) -> Result<Self, StringValueError> {
        let value = value.into();
        crate::types::ExchangeCode::new(value.clone())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EconomicIndicator {
    /// The 24 names listed by the current provider documentation.
    pub const DOCUMENTED: [Self; 24] = [
        Self::Gdp,
        Self::RealGdp,
        Self::NominalPotentialGdp,
        Self::RealGdpPerCapita,
        Self::FederalFunds,
        Self::Cpi,
        Self::InflationRate,
        Self::Inflation,
        Self::RetailSales,
        Self::ConsumerSentiment,
        Self::DurableGoods,
        Self::UnemploymentRate,
        Self::TotalNonfarmPayroll,
        Self::InitialClaims,
        Self::IndustrialProductionTotalIndex,
        Self::NewPrivatelyOwnedHousingUnitsStartedTotalUnits,
        Self::TotalVehicleSales,
        Self::RetailMoneyFunds,
        Self::SmoothedUsRecessionProbabilities,
        Self::ThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit,
        Self::CommercialBankInterestRateOnCreditCardPlansAllAccounts,
        Self::ThirtyYearFixedRateMortgageAverage,
        Self::FifteenYearFixedRateMortgageAverage,
        Self::TradeBalanceGoodsAndServices,
    ];

    /// Parses a documented name or validates it as an open provider value.
    pub fn new(value: impl Into<String>) -> Result<Self, StringValueError> {
        let value = value.into();
        let known = match value.as_str() {
            "GDP" => Self::Gdp,
            "realGDP" => Self::RealGdp,
            "nominalPotentialGDP" => Self::NominalPotentialGdp,
            "realGDPPerCapita" => Self::RealGdpPerCapita,
            "federalFunds" => Self::FederalFunds,
            "CPI" => Self::Cpi,
            "inflationRate" => Self::InflationRate,
            "inflation" => Self::Inflation,
            "retailSales" => Self::RetailSales,
            "consumerSentiment" => Self::ConsumerSentiment,
            "durableGoods" => Self::DurableGoods,
            "unemploymentRate" => Self::UnemploymentRate,
            "totalNonfarmPayroll" => Self::TotalNonfarmPayroll,
            "initialClaims" => Self::InitialClaims,
            "industrialProductionTotalIndex" => Self::IndustrialProductionTotalIndex,
            "newPrivatelyOwnedHousingUnitsStartedTotalUnits" => {
                Self::NewPrivatelyOwnedHousingUnitsStartedTotalUnits
            }
            "totalVehicleSales" => Self::TotalVehicleSales,
            "retailMoneyFunds" => Self::RetailMoneyFunds,
            "smoothedUSRecessionProbabilities" => Self::SmoothedUsRecessionProbabilities,
            "3MonthOr90DayRatesAndYieldsCertificatesOfDeposit" => {
                Self::ThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit
            }
            "commercialBankInterestRateOnCreditCardPlansAllAccounts" => {
                Self::CommercialBankInterestRateOnCreditCardPlansAllAccounts
            }
            "30YearFixedRateMortgageAverage" => Self::ThirtyYearFixedRateMortgageAverage,
            "15YearFixedRateMortgageAverage" => Self::FifteenYearFixedRateMortgageAverage,
            "tradeBalanceGoodsAndServices" => Self::TradeBalanceGoodsAndServices,
            _ => {
                // Reuse the public open-string validation contract without
                // conflating indicator names with country or exchange codes.
                Self::Other(OpenEconomicIndicator::new(value)?)
            }
        };
        Ok(known)
    }

    /// Returns the exact provider query representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gdp => "GDP",
            Self::RealGdp => "realGDP",
            Self::NominalPotentialGdp => "nominalPotentialGDP",
            Self::RealGdpPerCapita => "realGDPPerCapita",
            Self::FederalFunds => "federalFunds",
            Self::Cpi => "CPI",
            Self::InflationRate => "inflationRate",
            Self::Inflation => "inflation",
            Self::RetailSales => "retailSales",
            Self::ConsumerSentiment => "consumerSentiment",
            Self::DurableGoods => "durableGoods",
            Self::UnemploymentRate => "unemploymentRate",
            Self::TotalNonfarmPayroll => "totalNonfarmPayroll",
            Self::InitialClaims => "initialClaims",
            Self::IndustrialProductionTotalIndex => "industrialProductionTotalIndex",
            Self::NewPrivatelyOwnedHousingUnitsStartedTotalUnits => {
                "newPrivatelyOwnedHousingUnitsStartedTotalUnits"
            }
            Self::TotalVehicleSales => "totalVehicleSales",
            Self::RetailMoneyFunds => "retailMoneyFunds",
            Self::SmoothedUsRecessionProbabilities => "smoothedUSRecessionProbabilities",
            Self::ThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit => {
                "3MonthOr90DayRatesAndYieldsCertificatesOfDeposit"
            }
            Self::CommercialBankInterestRateOnCreditCardPlansAllAccounts => {
                "commercialBankInterestRateOnCreditCardPlansAllAccounts"
            }
            Self::ThirtyYearFixedRateMortgageAverage => "30YearFixedRateMortgageAverage",
            Self::FifteenYearFixedRateMortgageAverage => "15YearFixedRateMortgageAverage",
            Self::TradeBalanceGoodsAndServices => "tradeBalanceGoodsAndServices",
            Self::Other(value) => value.as_str(),
        }
    }
}

/// A representation-preserving provider query year.
///
/// Construction does not enforce an endpoint-specific range. Endpoints encode
/// the supplied value unchanged unless a narrower query type says otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Year(pub u32);

impl fmt::Display for Year {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A strictly positive technical-indicator period length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeriodLength(NonZeroU32);

impl PeriodLength {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for PeriodLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for EconomicIndicator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EconomicIndicator {
    type Err = StringValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for EconomicIndicator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EconomicIndicator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}
