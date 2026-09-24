package fmp

import (
	"errors"
	"fmt"
	"math"
	"slices"
	"strconv"
	"strings"
)

// Reasons a string-like query argument is rejected. They carry the same
// messages as StringValueError in the Rust crate.
var (
	ErrEmptyValue            = errors.New("value must not be empty or whitespace-only")
	ErrControlCharacterValue = errors.New("value must not contain control characters")
	ErrCommaInTicker         = errors.New("ticker must not contain a comma")
)

// validateStringValue mirrors validate_string_value in the Rust crate: the
// original text is preserved and never normalized.
func validateStringValue(value string, rejectComma bool) error {
	if strings.TrimSpace(value) == "" {
		return ErrEmptyValue
	}
	if hasControlCharacter(value) {
		return ErrControlCharacterValue
	}
	if rejectComma && strings.Contains(value, ",") {
		return ErrCommaInTicker
	}
	return nil
}

// tickerParam validates one registry "ticker" argument the way Ticker::new
// does and returns its query pair. The Rust crate validates when the Ticker
// is constructed; the Go SDK validates when the request is built, so the
// failure surfaces as a Validation error from the endpoint method.
func tickerParam(name, value string) (queryParam, error) {
	if err := validateStringValue(value, true); err != nil {
		return queryParam{}, validationError(name, err)
	}
	return queryParam{Name: name, Value: value}, nil
}

// Reasons a non-string query argument is rejected. They carry the same
// messages as the corresponding constructors in the Rust crate.
var (
	ErrEmptyTickerList        = errors.New("ticker list must contain at least one ticker")
	ErrNonFiniteDecimal       = errors.New("decimal value must be finite")
	ErrZeroPeriodLength       = errors.New("period length must be a positive integer")
	ErrInvalidCalendarQuarter = errors.New("calendar quarter must be an integer from 1 through 4")
)

// stringParam validates one registry string argument that is not a ticker
// (exchange codes, CIKs, search terms, and so on): non-empty and free of
// control characters, commas allowed, as the Rust string newtypes do.
func stringParam(name, value string) (queryParam, error) {
	if err := validateStringValue(value, false); err != nil {
		return queryParam{}, validationError(name, err)
	}
	return queryParam{Name: name, Value: value}, nil
}

// textParam encodes a registry "text" argument, which the Rust crate takes as
// a plain String without validation.
func textParam(name, value string) (queryParam, error) {
	return queryParam{Name: name, Value: value}, nil
}

// tickerListParam validates a registry "ticker_list" argument the way
// TickerList::new does: at least one ticker, each validated as a ticker, and
// encoded comma-separated in the given order.
func tickerListParam(name string, values []string) (queryParam, error) {
	if len(values) == 0 {
		return queryParam{}, validationError(name, ErrEmptyTickerList)
	}
	for _, value := range values {
		if err := validateStringValue(value, true); err != nil {
			return queryParam{}, validationError(name, err)
		}
	}
	return queryParam{Name: name, Value: strings.Join(values, ",")}, nil
}

// uint32Param encodes the unvalidated u32 newtypes (Limit, Page, Year,
// CalendarYear) in decimal.
func uint32Param(name string, value uint32) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatUint(uint64(value), 10)}, nil
}

// uint64Param encodes the u64 query aliases (MarketCapitalization and the
// screener volume filters; response Volume is float64) in decimal.
func uint64Param(name string, value uint64) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatUint(value, 10)}, nil
}

// periodLengthParam mirrors PeriodLength::new: the value must be positive.
func periodLengthParam(name string, value uint32) (queryParam, error) {
	if value == 0 {
		return queryParam{}, validationError(name, ErrZeroPeriodLength)
	}
	return uint32Param(name, value)
}

// calendarQuarterParam mirrors CalendarQuarter::new: 1 through 4.
func calendarQuarterParam(name string, value uint8) (queryParam, error) {
	if value < 1 || value > 4 {
		return queryParam{}, validationError(name, ErrInvalidCalendarQuarter)
	}
	return queryParam{Name: name, Value: strconv.FormatUint(uint64(value), 10)}, nil
}

// finiteDecimalParam mirrors FiniteDecimal::new: NaN and infinities are
// rejected; the shortest round-trip decimal text is sent, as Rust's Display
// for f64 does.
func finiteDecimalParam(name string, value float64) (queryParam, error) {
	if math.IsNaN(value) || math.IsInf(value, 0) {
		return queryParam{}, validationError(name, ErrNonFiniteDecimal)
	}
	return queryParam{Name: name, Value: strconv.FormatFloat(value, 'f', -1, 64)}, nil
}

// boolParam encodes a bool or TrueFalseFlag argument as "true" or "false".
func boolParam(name string, value bool) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatBool(value)}, nil
}

// dateParam encodes a Date as its exact "YYYY-MM-DD" wire text. The zero
// value has no wire form and is rejected.
func dateParam(name string, value Date) (queryParam, error) {
	if value.IsZero() {
		return queryParam{}, validationError(name, ErrZeroTemporalValue)
	}
	return queryParam{Name: name, Value: value.String()}, nil
}

// dateTimeParam encodes a DateTime as its exact "YYYY-MM-DD HH:MM:SS" wire
// text. The zero value has no wire form and is rejected.
func dateTimeParam(name string, value DateTime) (queryParam, error) {
	if value.IsZero() {
		return queryParam{}, validationError(name, ErrZeroTemporalValue)
	}
	return queryParam{Name: name, Value: value.String()}, nil
}

// RetrievalFrequency is the annual or quarterly retrieval frequency of the
// endpoints whose Rust query takes a RetrievalFrequency. The constants carry
// the exact provider wire spellings; any other value is rejected when the
// request is built.
type RetrievalFrequency string

// The documented RetrievalFrequency wire values.
const (
	RetrievalFrequencyAnnual    RetrievalFrequency = "annual"
	RetrievalFrequencyQuarterly RetrievalFrequency = "quarter"
)

// ErrUnknownWireValue is returned when an enumerated argument is not one of
// its documented wire values. The Rust crate makes this unrepresentable with
// a closed enum; the Go SDK checks it when the request is built.
//
// Wire enum policy: a query type is closed (RetrievalFrequency, Quarter,
// ChartTimeframe, FiscalPeriod, StatementPeriod, SegmentationStructure) when
// its libfmp counterpart is a closed Rust enum, and then any value outside its
// constants fails with ErrUnknownWireValue before a request is sent. A type
// is open (EconomicIndicator) only when the libfmp type accepts provider
// values it does not list; its constants are the documented spellings and
// any other valid string is forwarded. Widening a closed type follows the
// Rust enum gaining a variant, never the provider documentation alone.
var ErrUnknownWireValue = errors.New("value is not a documented wire value")

// wireEnumParam encodes an enumerated argument after checking that the value
// is one of the documented wire spellings, which are sent verbatim.
func wireEnumParam(name, value string, documented ...string) (queryParam, error) {
	if !slices.Contains(documented, value) {
		return queryParam{}, validationError(name, fmt.Errorf("%w, expected one of %s",
			ErrUnknownWireValue, strings.Join(documented, ", ")))
	}
	return queryParam{Name: name, Value: value}, nil
}

// retrievalFrequencyParam mirrors the libfmp RetrievalFrequency wire enum:
// "annual" or "quarter", nothing else.
func retrievalFrequencyParam(name string, value RetrievalFrequency) (queryParam, error) {
	return wireEnumParam(name, string(value),
		string(RetrievalFrequencyAnnual), string(RetrievalFrequencyQuarterly))
}

// Quarter is the textual calendar quarter taken by the endpoints whose Rust
// query takes a Quarter. The constants carry the exact provider wire
// spellings "1" through "4"; any other value is rejected when the request is
// built. It is distinct from the numeric CalendarQuarter of the response
// models, which is decoded from a JSON integer.
type Quarter string

// The documented Quarter wire values.
const (
	QuarterQ1 Quarter = "1"
	QuarterQ2 Quarter = "2"
	QuarterQ3 Quarter = "3"
	QuarterQ4 Quarter = "4"
)

// quarterParam mirrors the libfmp Quarter wire enum: "1" through "4",
// nothing else.
func quarterParam(name string, value Quarter) (queryParam, error) {
	return wireEnumParam(name, string(value),
		string(QuarterQ1), string(QuarterQ2), string(QuarterQ3), string(QuarterQ4))
}

// ChartTimeframe is the bar timeframe taken by the technical-indicator
// endpoints. The constants carry the exact provider wire spellings of the
// libfmp ChartTimeframe wire enum, in its order; any other value is rejected
// when the request is built.
type ChartTimeframe string

// The documented ChartTimeframe wire values.
const (
	ChartTimeframeOneMinute      ChartTimeframe = "1min"
	ChartTimeframeFiveMinutes    ChartTimeframe = "5min"
	ChartTimeframeFifteenMinutes ChartTimeframe = "15min"
	ChartTimeframeThirtyMinutes  ChartTimeframe = "30min"
	ChartTimeframeOneHour        ChartTimeframe = "1hour"
	ChartTimeframeFourHours      ChartTimeframe = "4hour"
	ChartTimeframeOneDay         ChartTimeframe = "1day"
)

// chartTimeframeParam mirrors the libfmp ChartTimeframe wire enum: one of
// the seven documented spellings, nothing else.
func chartTimeframeParam(name string, value ChartTimeframe) (queryParam, error) {
	return wireEnumParam(name, string(value),
		string(ChartTimeframeOneMinute), string(ChartTimeframeFiveMinutes),
		string(ChartTimeframeFifteenMinutes), string(ChartTimeframeThirtyMinutes),
		string(ChartTimeframeOneHour), string(ChartTimeframeFourHours),
		string(ChartTimeframeOneDay))
}

// FiscalPeriod is a fiscal reporting period of the endpoints whose Rust
// query takes a FiscalPeriod: the four quarters and the full year, without
// the retrieval-frequency aliases. The constants carry the exact provider
// wire spellings; any other value is rejected when the request is built.
type FiscalPeriod string

// The documented FiscalPeriod wire values.
const (
	FiscalPeriodQ1       FiscalPeriod = "Q1"
	FiscalPeriodQ2       FiscalPeriod = "Q2"
	FiscalPeriodQ3       FiscalPeriod = "Q3"
	FiscalPeriodQ4       FiscalPeriod = "Q4"
	FiscalPeriodFullYear FiscalPeriod = "FY"
)

// fiscalPeriodParam mirrors the libfmp FiscalPeriod wire enum: Q1 through
// Q4 or FY, nothing else.
func fiscalPeriodParam(name string, value FiscalPeriod) (queryParam, error) {
	return wireEnumParam(name, string(value),
		string(FiscalPeriodQ1), string(FiscalPeriodQ2), string(FiscalPeriodQ3),
		string(FiscalPeriodQ4), string(FiscalPeriodFullYear))
}

// StatementPeriod is the seven-value period selector of the standard
// statement endpoints: a FiscalPeriod or a RetrievalFrequency, as the libfmp
// StatementPeriod union composes them. The constants repeat the wire
// spellings of both families so a caller can pass either; any other value is
// rejected when the request is built.
type StatementPeriod string

// The documented StatementPeriod wire values: the FiscalPeriod family first,
// then the RetrievalFrequency family.
const (
	StatementPeriodQ1        StatementPeriod = "Q1"
	StatementPeriodQ2        StatementPeriod = "Q2"
	StatementPeriodQ3        StatementPeriod = "Q3"
	StatementPeriodQ4        StatementPeriod = "Q4"
	StatementPeriodFullYear  StatementPeriod = "FY"
	StatementPeriodAnnual    StatementPeriod = "annual"
	StatementPeriodQuarterly StatementPeriod = "quarter"
)

// statementPeriodParam mirrors the libfmp StatementPeriod union: one of the
// five FiscalPeriod spellings or the two RetrievalFrequency spellings, in the
// order the Rust decoder lists them.
func statementPeriodParam(name string, value StatementPeriod) (queryParam, error) {
	return wireEnumParam(name, string(value),
		string(StatementPeriodQ1), string(StatementPeriodQ2), string(StatementPeriodQ3),
		string(StatementPeriodQ4), string(StatementPeriodFullYear),
		string(StatementPeriodAnnual), string(StatementPeriodQuarterly))
}

// SegmentationStructure is the documented response structure of the
// revenue-segmentation endpoints. The provider documents only "flat"; the
// libfmp SegmentationStructure is a closed enum with that one variant, so any
// other value is rejected when the request is built.
type SegmentationStructure string

// The documented SegmentationStructure wire values.
const (
	SegmentationStructureFlat SegmentationStructure = "flat"
)

// segmentationStructureParam mirrors the libfmp SegmentationStructure wire
// enum: "flat", nothing else.
func segmentationStructureParam(name string, value SegmentationStructure) (queryParam, error) {
	return wireEnumParam(name, string(value), string(SegmentationStructureFlat))
}

// EconomicIndicator is the indicator name of the economic-indicators
// endpoint. The constants carry the 24 documented provider spellings, sent
// verbatim. Unlike a closed wire enum, the libfmp EconomicIndicator is open:
// any other non-empty name free of control characters is forwarded as a
// provider value, so a name the documentation gained later still works.
type EconomicIndicator string

// The documented EconomicIndicator wire values, in the order the libfmp
// EconomicIndicator::DOCUMENTED table lists them.
const (
	EconomicIndicatorGdp                                                      EconomicIndicator = "GDP"
	EconomicIndicatorRealGdp                                                  EconomicIndicator = "realGDP"
	EconomicIndicatorNominalPotentialGdp                                      EconomicIndicator = "nominalPotentialGDP"
	EconomicIndicatorRealGdpPerCapita                                         EconomicIndicator = "realGDPPerCapita"
	EconomicIndicatorFederalFunds                                             EconomicIndicator = "federalFunds"
	EconomicIndicatorCpi                                                      EconomicIndicator = "CPI"
	EconomicIndicatorInflationRate                                            EconomicIndicator = "inflationRate"
	EconomicIndicatorInflation                                                EconomicIndicator = "inflation"
	EconomicIndicatorRetailSales                                              EconomicIndicator = "retailSales"
	EconomicIndicatorConsumerSentiment                                        EconomicIndicator = "consumerSentiment"
	EconomicIndicatorDurableGoods                                             EconomicIndicator = "durableGoods"
	EconomicIndicatorUnemploymentRate                                         EconomicIndicator = "unemploymentRate"
	EconomicIndicatorTotalNonfarmPayroll                                      EconomicIndicator = "totalNonfarmPayroll"
	EconomicIndicatorInitialClaims                                            EconomicIndicator = "initialClaims"
	EconomicIndicatorIndustrialProductionTotalIndex                           EconomicIndicator = "industrialProductionTotalIndex"
	EconomicIndicatorNewPrivatelyOwnedHousingUnitsStartedTotalUnits           EconomicIndicator = "newPrivatelyOwnedHousingUnitsStartedTotalUnits"
	EconomicIndicatorTotalVehicleSales                                        EconomicIndicator = "totalVehicleSales"
	EconomicIndicatorRetailMoneyFunds                                         EconomicIndicator = "retailMoneyFunds"
	EconomicIndicatorSmoothedUsRecessionProbabilities                         EconomicIndicator = "smoothedUSRecessionProbabilities"
	EconomicIndicatorThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit EconomicIndicator = "3MonthOr90DayRatesAndYieldsCertificatesOfDeposit"
	EconomicIndicatorCommercialBankInterestRateOnCreditCardPlansAllAccounts   EconomicIndicator = "commercialBankInterestRateOnCreditCardPlansAllAccounts"
	EconomicIndicatorThirtyYearFixedRateMortgageAverage                       EconomicIndicator = "30YearFixedRateMortgageAverage"
	EconomicIndicatorFifteenYearFixedRateMortgageAverage                      EconomicIndicator = "15YearFixedRateMortgageAverage"
	EconomicIndicatorTradeBalanceGoodsAndServices                             EconomicIndicator = "tradeBalanceGoodsAndServices"
)

// DocumentedEconomicIndicators lists the 24 documented indicator names, as
// the libfmp EconomicIndicator::DOCUMENTED table does. Each call returns a
// fresh copy, so modifying it never changes a later result.
func DocumentedEconomicIndicators() []EconomicIndicator {
	return slices.Clone(documentedEconomicIndicators[:])
}

var documentedEconomicIndicators = [...]EconomicIndicator{
	EconomicIndicatorGdp,
	EconomicIndicatorRealGdp,
	EconomicIndicatorNominalPotentialGdp,
	EconomicIndicatorRealGdpPerCapita,
	EconomicIndicatorFederalFunds,
	EconomicIndicatorCpi,
	EconomicIndicatorInflationRate,
	EconomicIndicatorInflation,
	EconomicIndicatorRetailSales,
	EconomicIndicatorConsumerSentiment,
	EconomicIndicatorDurableGoods,
	EconomicIndicatorUnemploymentRate,
	EconomicIndicatorTotalNonfarmPayroll,
	EconomicIndicatorInitialClaims,
	EconomicIndicatorIndustrialProductionTotalIndex,
	EconomicIndicatorNewPrivatelyOwnedHousingUnitsStartedTotalUnits,
	EconomicIndicatorTotalVehicleSales,
	EconomicIndicatorRetailMoneyFunds,
	EconomicIndicatorSmoothedUsRecessionProbabilities,
	EconomicIndicatorThreeMonthOrNinetyDayRatesAndYieldsCertificatesOfDeposit,
	EconomicIndicatorCommercialBankInterestRateOnCreditCardPlansAllAccounts,
	EconomicIndicatorThirtyYearFixedRateMortgageAverage,
	EconomicIndicatorFifteenYearFixedRateMortgageAverage,
	EconomicIndicatorTradeBalanceGoodsAndServices,
}

// economicIndicatorParam mirrors EconomicIndicator::new in the Rust crate: a
// documented name or any open provider name that passes the open-string rule
// (non-empty, no control characters, commas allowed) is sent verbatim.
func economicIndicatorParam(name string, value EconomicIndicator) (queryParam, error) {
	if err := validateStringValue(string(value), false); err != nil {
		return queryParam{}, validationError(name, err)
	}
	return queryParam{Name: name, Value: string(value)}, nil
}
