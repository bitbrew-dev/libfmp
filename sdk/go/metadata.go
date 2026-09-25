package fmp

import (
	"fmt"
	"strings"
)

// GeographicAvailability is the geographic coverage documented for an
// endpoint. It mirrors the Rust GeographicAvailability; the zero value is
// GeographyUnspecified.
type GeographicAvailability int

const (
	// GeographyUnspecified: the documentation states no coverage.
	GeographyUnspecified GeographicAvailability = iota
	// GeographyWorldwide: the endpoint covers every supported market.
	GeographyWorldwide
	// GeographyUSOnly: the endpoint covers US listings only.
	GeographyUSOnly
)

// String returns a short stable label that is safe to log.
func (g GeographicAvailability) String() string {
	switch g {
	case GeographyUnspecified:
		return "unspecified"
	case GeographyWorldwide:
		return "worldwide"
	case GeographyUSOnly:
		return "us-only"
	default:
		return fmt.Sprintf("GeographicAvailability(%d)", int(g))
	}
}

// AccessKind is the discriminant of AccessRequirement, the Go spelling of
// the Rust enum's variants. The zero value is AccessUnspecified.
type AccessKind int

const (
	// AccessUnspecified: the documentation states no plan requirement.
	AccessUnspecified AccessKind = iota
	// AccessStandard: the endpoint is part of the standard plans.
	AccessStandard
	// AccessNamedAddOn: the endpoint needs the add-on named by
	// AccessRequirement.AddOn.
	AccessNamedAddOn
)

// String returns a short stable label that is safe to log.
func (k AccessKind) String() string {
	switch k {
	case AccessUnspecified:
		return "unspecified"
	case AccessStandard:
		return "standard"
	case AccessNamedAddOn:
		return "named-add-on"
	default:
		return fmt.Sprintf("AccessKind(%d)", int(k))
	}
}

// AccessRequirement is the plan access documented for an endpoint. It
// mirrors the Rust AccessRequirement: Kind selects the variant and AddOn
// carries the add-on name only when Kind is AccessNamedAddOn.
type AccessRequirement struct {
	Kind  AccessKind
	AddOn string
}

// String returns a short label that is safe to log.
func (a AccessRequirement) String() string {
	if a.Kind == AccessNamedAddOn {
		return "add-on " + a.AddOn
	}
	return a.Kind.String()
}

// PlanConditionKind is the discriminant of PlanCondition. The zero value is
// PlanConditionUnspecified.
type PlanConditionKind int

const (
	// PlanConditionUnspecified: no condition is documented.
	PlanConditionUnspecified PlanConditionKind = iota
	// PlanConditionHistoryOlderThanYears: requests for history older than
	// PlanCondition.Years require the plan.
	PlanConditionHistoryOlderThanYears
)

// PlanCondition is the condition under which an additional plan is
// required. It mirrors the Rust PlanCondition: Kind selects the variant and
// Years carries the threshold of PlanConditionHistoryOlderThanYears.
type PlanCondition struct {
	Kind  PlanConditionKind
	Years uint16
}

// String returns a short label that is safe to log.
func (c PlanCondition) String() string {
	switch c.Kind {
	case PlanConditionUnspecified:
		return "unspecified"
	case PlanConditionHistoryOlderThanYears:
		return fmt.Sprintf("history older than %d years", c.Years)
	default:
		return fmt.Sprintf("PlanCondition(%d)", int(c.Kind))
	}
}

// ConditionalPlanRequirement names a plan required only when its condition
// applies. It mirrors the Rust ConditionalPlanRequirement.
type ConditionalPlanRequirement struct {
	Plan      string
	Condition PlanCondition
}

// String returns a short label that is safe to log.
func (r ConditionalPlanRequirement) String() string {
	return r.Plan + " when " + r.Condition.String()
}

// UserDeclarationRequirement reports whether a user declaration is required
// for a real-time feed. It mirrors the Rust UserDeclarationRequirement; the
// zero value is UserDeclarationUnspecified.
type UserDeclarationRequirement int

const (
	// UserDeclarationUnspecified: no declaration is documented.
	UserDeclarationUnspecified UserDeclarationRequirement = iota
	// UserDeclarationRequiredForRealtime: real-time data needs a declaration.
	UserDeclarationRequiredForRealtime
)

// String returns a short stable label that is safe to log.
func (u UserDeclarationRequirement) String() string {
	switch u {
	case UserDeclarationUnspecified:
		return "unspecified"
	case UserDeclarationRequiredForRealtime:
		return "required-for-realtime"
	default:
		return fmt.Sprintf("UserDeclarationRequirement(%d)", int(u))
	}
}

// DelayScope is the market scope a documented data delay applies to. It
// mirrors the Rust DelayScope; the zero value is DelayScopeUnspecified.
type DelayScope int

const (
	// DelayScopeUnspecified: the delay names no market.
	DelayScopeUnspecified DelayScope = iota
	// DelayScopeNasdaq: the delay applies to Nasdaq-listed data.
	DelayScopeNasdaq
)

// String returns a short stable label that is safe to log.
func (s DelayScope) String() string {
	switch s {
	case DelayScopeUnspecified:
		return "unspecified"
	case DelayScopeNasdaq:
		return "nasdaq"
	default:
		return fmt.Sprintf("DelayScope(%d)", int(s))
	}
}

// MarketDataDelay is a documented market-data delay and the scope it
// applies to. It mirrors the Rust MarketDataDelay.
type MarketDataDelay struct {
	Minutes uint16
	Scope   DelayScope
}

// String returns a short label that is safe to log.
func (d MarketDataDelay) String() string {
	return fmt.Sprintf("%d minutes (%s)", d.Minutes, d.Scope)
}

// RealtimeAccess holds the real-time access caveats documented for an
// endpoint. It mirrors the Rust RealtimeAccess: Delay is nil when no delay is
// documented, and UserDeclaration is UserDeclarationUnspecified when no
// declaration is documented.
type RealtimeAccess struct {
	Delay           *MarketDataDelay
	UserDeclaration UserDeclarationRequirement
}

// String returns a short label that is safe to log.
func (r RealtimeAccess) String() string {
	parts := make([]string, 0, 2)
	if r.Delay != nil {
		parts = append(parts, "delay "+r.Delay.String())
	}
	if r.UserDeclaration != UserDeclarationUnspecified {
		parts = append(parts, "declaration "+r.UserDeclaration.String())
	}
	if len(parts) == 0 {
		return "no caveats"
	}
	return strings.Join(parts, ", ")
}

// EndpointBounds holds the inclusive maxima documented for one endpoint. It
// mirrors the Rust EndpointBounds: each member is nil when the documentation
// states no maximum. The bounds are advisory; the client never enforces them.
type EndpointBounds struct {
	Limit         *uint32
	ResponseRows  *uint32
	Page          *uint32
	DateRangeDays *uint32
}

// String returns a short label that is safe to log.
func (b EndpointBounds) String() string {
	parts := make([]string, 0, 4)
	for _, bound := range []struct {
		name  string
		value *uint32
	}{
		{"limit", b.Limit},
		{"response_rows", b.ResponseRows},
		{"page", b.Page},
		{"date_range_days", b.DateRangeDays},
	} {
		if bound.value != nil {
			parts = append(parts, fmt.Sprintf("%s<=%d", bound.name, *bound.value))
		}
	}
	if len(parts) == 0 {
		return "unbounded"
	}
	return strings.Join(parts, ", ")
}

// EndpointMetadata is the advisory documentation metadata of one endpoint
// method, mirroring the Rust EndpointMetadata. The zero value is what
// EndpointMetadata::new() produces on the Rust side: every member
// unspecified, no conditional plan, no realtime caveats, no bounds. The client
// never validates a request against it; callers read it to pick endpoints and
// size requests. Look one up with EndpointMetadataFor or EndpointMetadataByID.
type EndpointMetadata struct {
	Geography       GeographicAvailability
	Access          AccessRequirement
	ConditionalPlan *ConditionalPlanRequirement
	Realtime        *RealtimeAccess
	Bounds          EndpointBounds
}

// String returns a one-line summary that is safe to log.
func (m EndpointMetadata) String() string {
	parts := []string{
		"geography " + m.Geography.String(),
		"access " + m.Access.String(),
	}
	if m.ConditionalPlan != nil {
		parts = append(parts, "plan "+m.ConditionalPlan.String())
	}
	if m.Realtime != nil {
		parts = append(parts, "realtime "+m.Realtime.String())
	}
	parts = append(parts, "bounds "+m.Bounds.String())
	return strings.Join(parts, "; ")
}

// clone returns a copy that shares no pointer with m, so a value handed out
// of the generated table cannot be mutated through a previous lookup.
func (m EndpointMetadata) clone() EndpointMetadata {
	out := m
	if m.ConditionalPlan != nil {
		plan := *m.ConditionalPlan
		out.ConditionalPlan = &plan
	}
	if m.Realtime != nil {
		realtime := *m.Realtime
		if m.Realtime.Delay != nil {
			delay := *m.Realtime.Delay
			realtime.Delay = &delay
		}
		out.Realtime = &realtime
	}
	out.Bounds = EndpointBounds{
		Limit:         cloneMaximum(m.Bounds.Limit),
		ResponseRows:  cloneMaximum(m.Bounds.ResponseRows),
		Page:          cloneMaximum(m.Bounds.Page),
		DateRangeDays: cloneMaximum(m.Bounds.DateRangeDays),
	}
	return out
}

// inclusiveMaximum is the *uint32 the generated table spells a documented
// maximum with.
func inclusiveMaximum(maximum uint32) *uint32 {
	return &maximum
}

func cloneMaximum(maximum *uint32) *uint32 {
	if maximum == nil {
		return nil
	}
	return inclusiveMaximum(*maximum)
}

// EndpointMethodMetadata is the metadata of one method that sends a given
// endpoint id. Method is the EndpointMetadataFor key of that method.
type EndpointMethodMetadata struct {
	Method   string
	Metadata EndpointMetadata
}

// EndpointMetadataByID returns the metadata of every generated method that
// sends the endpoint id carried by Error.Endpoint ("quote-short"), one entry
// per method whose descriptor attaches metadata, sorted by Method. Several
// methods can share one id through helper reuse and carry different
// metadata (the id "quote" serves five namespaces with different
// geography), so the result is never merged: callers that know which method
// they called pick its entry by Method, and callers that do not must treat
// a result with more than one distinct Metadata as ambiguous. A method that
// sends the id but attaches no metadata has no entry. It returns nil for an
// unknown id and for an id none of whose methods attaches metadata. Every
// value is a copy.
func EndpointMetadataByID(id string) []EndpointMethodMetadata {
	var out []EndpointMethodMetadata
	for _, method := range endpointMethodsByID[id] {
		if metadata, ok := EndpointMetadataFor(method); ok {
			out = append(out, EndpointMethodMetadata{Method: method, Metadata: metadata})
		}
	}
	return out
}
