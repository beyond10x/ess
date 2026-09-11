package system

import (
	"example.invalid/projection/types/core"
	"testing"
)

func TestNativePresenceAndNestedOptionalIdentity(t *testing.T) {
	var inner *string
	body := core.Body{Status: "ready", Nested: &inner}
	event := core.Arrived{Data: body, Partial: nil, Choice: core.ChoiceGone{Value: "gone"}, Wrapped: core.NewWrapped(core.Wrapper{Body: body})}
	input := Project(event)
	if input.Text != "ready" || input.Partial != nil || input.Choice != nil {
		t.Fatalf("wrong partial mapping: %#v", input)
	}
	if input.Nested == nil || *input.Nested != nil {
		t.Fatal("Some(None) was flattened")
	}
	if input.Deeper == nil || *input.Deeper == nil || **input.Deeper != nil {
		t.Fatal("added Some lost nested identity")
	}
	if input.ThroughWrapper != "ready" || input.Whole.Status != "ready" {
		t.Fatal("newtype or whole value lost")
	}
	event.Partial = &body
	event.Choice = core.ChoiceReady{Value: body}
	event.Data.Nested = nil
	input = Project(event)
	if input.Partial == nil || *input.Partial != "ready" || input.Choice == nil || *input.Choice != "ready" {
		t.Fatal("available path lost")
	}
	if input.Nested != nil || input.Deeper == nil || *input.Deeper != nil {
		t.Fatal("None did not receive exactly one Some")
	}
}
