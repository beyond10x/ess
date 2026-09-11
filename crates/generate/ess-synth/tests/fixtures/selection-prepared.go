package system_test

import (
	"example.invalid/selection/system"
	"example.invalid/selection/types/core"
	"testing"
)

func TestHostPreparesOnceThenGeneratedCodeSelects(t *testing.T) {
	event := core.Arrived{Data: "host-owned raw representation"}
	calls := 0
	prepare := func(_ string) []*core.Leg {
		calls++
		var external core.Domain = core.DomainExternal{}
		var internal core.Domain = core.DomainInternal{}
		return []*core.Leg{{Id: "external", From: "remote", Domain: &external}, {Id: "agent", From: "remote", Domain: &internal}}
	}
	prepared := prepare(event.Data)
	result, failure := system.ChooseFromPrepared(event, prepared)
	if failure != nil {
		t.Fatal(failure)
	}
	if calls != 1 || result.AgentId == nil || *result.AgentId != "agent" || result.ExternalId == nil || *result.ExternalId != "external" {
		t.Fatalf("wrong preparation/selection result: %d %+v", calls, result)
	}
}
