package system_test

import (
	"example.invalid/selection/system"
	"example.invalid/selection/types/core"
	"testing"
)

func leg(id string, domain core.Domain, source core.Source) *core.Leg {
	value := &core.Leg{Id: id, From: "remote"}
	if domain != nil {
		value.Domain = &domain
	}
	if source != nil {
		value.Source = &source
	}
	return value
}
func text(value string) *string { return &value }
func same(a, b *string) bool {
	if a == nil || b == nil {
		return a == b
	}
	return *a == *b
}
func TestMixedOccurrencesAndFallbacks(t *testing.T) {
	I, E, W := core.DomainInternal{}, core.DomainExternal{}, core.SourceWebrtc{}
	rows := []struct {
		data            []*core.Leg
		agent, external *string
	}{
		{[]*core.Leg{}, nil, nil},
		{[]*core.Leg{nil, leg("x", nil, nil)}, text("x"), text("x")},
		{[]*core.Leg{leg("x", E, nil), leg("y", I, nil)}, text("y"), text("x")},
		{[]*core.Leg{leg("x", E, W)}, text("x"), text("x")},
		{[]*core.Leg{leg("x", E, W), leg("y", E, nil)}, text("x"), text("y")},
		{[]*core.Leg{leg("x", I, nil), leg("y", E, W)}, text("x"), text("y")},
		{[]*core.Leg{leg("x", E, W), leg("y", E, W)}, text("x"), text("y")},
		{[]*core.Leg{leg("x", E, nil), leg("y", E, nil)}, text("x"), text("x")},
		{[]*core.Leg{nil, leg("", nil, nil), leg("z", nil, nil)}, text("z"), text("z")},
		{[]*core.Leg{leg("", I, nil), leg("y", E, nil)}, text(""), text("y")},
		{[]*core.Leg{leg("", E, nil), leg("y", nil, nil)}, text("y"), text("")},
		{[]*core.Leg{leg("x", I, nil), leg("y", I, nil), leg("z", E, nil)}, text("x"), text("z")},
		{[]*core.Leg{leg("same", E, W), leg("same", E, nil)}, text("same"), text("same")},
	}
	for index, row := range rows {
		value, failure := system.Choose(core.Arrived{Data: row.data})
		if failure != nil {
			t.Fatalf("row%d: %v", index, failure)
		}
		if !same(value.AgentId, row.agent) || !same(value.ExternalId, row.external) {
			t.Fatalf("row%d wrong occurrences", index)
		}
	}
}
func TestOverLimitTail(t *testing.T) {
	data := make([]*core.Leg, 65)
	for index := range data {
		data[index] = leg("x", core.DomainInternal{}, nil)
	}
	_, failure := system.Choose(core.Arrived{Data: data})
	if failure == nil || failure.Cause != system.SelectionResource {
		t.Fatalf("expected resource refusal: %v", failure)
	}
}
func TestMalformedEnumTail(t *testing.T) {
	var invalid core.Domain
	tail := leg("bad", nil, nil)
	tail.Domain = &invalid
	_, failure := system.Choose(core.Arrived{Data: []*core.Leg{leg("x", core.DomainInternal{}, nil), tail}})
	if failure == nil || failure.Cause != system.SelectionInvalidInput || failure.Index == nil || *failure.Index != 1 {
		t.Fatalf("expected typed malformed tail refusal: %v", failure)
	}
}
