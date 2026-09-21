package fmp

// Hand-written template for the quote domain models (phase 0c, issue #279).
// Every line here is one the gen_go emitter reproduces mechanically from
// crates/libfmp/src/responses/quote.rs: the field order, the camelCase wire
// names, the type table of ADR 0030, and the required-member shadow decode.
// The generator regenerates this file and deletes the hand-written copy.

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
)

// Quote is a detailed real-time stock quote.
type Quote struct {
	Symbol           string      `json:"symbol"`
	Name             string      `json:"name"`
	Price            float64     `json:"price"`
	ChangePercentage float64     `json:"changePercentage"`
	Change           float64     `json:"change"`
	Volume           uint64      `json:"volume"`
	DayLow           float64     `json:"dayLow"`
	DayHigh          float64     `json:"dayHigh"`
	YearHigh         float64     `json:"yearHigh"`
	YearLow          float64     `json:"yearLow"`
	MarketCap        *uint64     `json:"marketCap"`
	PriceAvg50       float64     `json:"priceAvg50"`
	PriceAvg200      float64     `json:"priceAvg200"`
	Exchange         string      `json:"exchange"`
	Open             float64     `json:"open"`
	PreviousClose    float64     `json:"previousClose"`
	Timestamp        UnixSeconds `json:"timestamp"`
}

// quoteShadow mirrors Quote with a pointer for every required member so a
// missing or null member is observable after decoding.
type quoteShadow struct {
	Symbol           *string      `json:"symbol"`
	Name             *string      `json:"name"`
	Price            *float64     `json:"price"`
	ChangePercentage *float64     `json:"changePercentage"`
	Change           *float64     `json:"change"`
	Volume           *uint64      `json:"volume"`
	DayLow           *float64     `json:"dayLow"`
	DayHigh          *float64     `json:"dayHigh"`
	YearHigh         *float64     `json:"yearHigh"`
	YearLow          *float64     `json:"yearLow"`
	MarketCap        *uint64      `json:"marketCap"`
	PriceAvg50       *float64     `json:"priceAvg50"`
	PriceAvg200      *float64     `json:"priceAvg200"`
	Exchange         *string      `json:"exchange"`
	Open             *float64     `json:"open"`
	PreviousClose    *float64     `json:"previousClose"`
	Timestamp        *UnixSeconds `json:"timestamp"`
}

// UnmarshalJSONFrom decodes one JSON object and rejects it with a Decode
// error naming the first required member that is missing or null, as the
// Rust decoder does. Unknown members are ignored.
func (m *Quote) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	var shadow quoteShadow
	if err := json.UnmarshalDecode(dec, &shadow); err != nil {
		return err
	}
	switch {
	case shadow.Symbol == nil:
		return missingMemberError("Quote", "symbol")
	case shadow.Name == nil:
		return missingMemberError("Quote", "name")
	case shadow.Price == nil:
		return missingMemberError("Quote", "price")
	case shadow.ChangePercentage == nil:
		return missingMemberError("Quote", "changePercentage")
	case shadow.Change == nil:
		return missingMemberError("Quote", "change")
	case shadow.Volume == nil:
		return missingMemberError("Quote", "volume")
	case shadow.DayLow == nil:
		return missingMemberError("Quote", "dayLow")
	case shadow.DayHigh == nil:
		return missingMemberError("Quote", "dayHigh")
	case shadow.YearHigh == nil:
		return missingMemberError("Quote", "yearHigh")
	case shadow.YearLow == nil:
		return missingMemberError("Quote", "yearLow")
	case shadow.PriceAvg50 == nil:
		return missingMemberError("Quote", "priceAvg50")
	case shadow.PriceAvg200 == nil:
		return missingMemberError("Quote", "priceAvg200")
	case shadow.Exchange == nil:
		return missingMemberError("Quote", "exchange")
	case shadow.Open == nil:
		return missingMemberError("Quote", "open")
	case shadow.PreviousClose == nil:
		return missingMemberError("Quote", "previousClose")
	case shadow.Timestamp == nil:
		return missingMemberError("Quote", "timestamp")
	}
	*m = Quote{
		Symbol:           *shadow.Symbol,
		Name:             *shadow.Name,
		Price:            *shadow.Price,
		ChangePercentage: *shadow.ChangePercentage,
		Change:           *shadow.Change,
		Volume:           *shadow.Volume,
		DayLow:           *shadow.DayLow,
		DayHigh:          *shadow.DayHigh,
		YearHigh:         *shadow.YearHigh,
		YearLow:          *shadow.YearLow,
		MarketCap:        shadow.MarketCap,
		PriceAvg50:       *shadow.PriceAvg50,
		PriceAvg200:      *shadow.PriceAvg200,
		Exchange:         *shadow.Exchange,
		Open:             *shadow.Open,
		PreviousClose:    *shadow.PreviousClose,
		Timestamp:        *shadow.Timestamp,
	}
	return nil
}

// QuoteShort is the compact response returned by quote-short endpoints across
// asset classes.
type QuoteShort struct {
	Symbol string  `json:"symbol"`
	Price  float64 `json:"price"`
	Change float64 `json:"change"`
	Volume uint64  `json:"volume"`
}

// quoteShortShadow mirrors QuoteShort with a pointer for every required
// member so a missing or null member is observable after decoding.
type quoteShortShadow struct {
	Symbol *string  `json:"symbol"`
	Price  *float64 `json:"price"`
	Change *float64 `json:"change"`
	Volume *uint64  `json:"volume"`
}

// UnmarshalJSONFrom decodes one JSON object and rejects it with a Decode
// error naming the first required member that is missing or null, as the
// Rust decoder does. Unknown members are ignored.
func (m *QuoteShort) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	var shadow quoteShortShadow
	if err := json.UnmarshalDecode(dec, &shadow); err != nil {
		return err
	}
	switch {
	case shadow.Symbol == nil:
		return missingMemberError("QuoteShort", "symbol")
	case shadow.Price == nil:
		return missingMemberError("QuoteShort", "price")
	case shadow.Change == nil:
		return missingMemberError("QuoteShort", "change")
	case shadow.Volume == nil:
		return missingMemberError("QuoteShort", "volume")
	}
	*m = QuoteShort{
		Symbol: *shadow.Symbol,
		Price:  *shadow.Price,
		Change: *shadow.Change,
		Volume: *shadow.Volume,
	}
	return nil
}
