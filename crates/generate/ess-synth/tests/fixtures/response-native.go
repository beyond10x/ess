import (
	p "example.invalid/demo/types/primitives"
	"testing"
)

func responseItem() Item {
	return Item{Remaining: 37, Created: p.NewTimestamp("2026-09-11T10:00:00Z"), Ended: p.NewTimestamp("2026-09-11T10:01:00Z"), State: nil, CallType: "incoming", Features: map[string]bool{"enabled": true}}
}
func nativeReturned() CancelOutcomeCancelled {
	return CancelOutcomeCancelled{Response: CancelResponse{Item: responseItem()}, Returned: Returned{Item: responseItem(), Receipt: "generated-37"}}
}
func TestNativeResponseValuesAreActual(t *testing.T) {
	result := nativeReturned()
	if !result.ResponsePayloadMatches() {
		t.Fatal("valid response mismatch")
	}
	result.Response.Item.Remaining = 38
	if result.ResponsePayloadMatches() {
		t.Fatal("different actual response passed")
	}
}
func TestNativeResponseMapMutationIsNotHidden(t *testing.T) {
	result := nativeReturned()
	if !result.ResponsePayloadMatches() {
		t.Fatal("valid response mismatch")
	}
	result.Returned.Item.Features["enabled"] = false
	if result.ResponsePayloadMatches() {
		t.Fatal("different event map passed")
	}
}
