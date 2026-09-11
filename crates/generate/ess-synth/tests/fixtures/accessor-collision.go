package system

import (
	"example.invalid/projection/types/core"
	"testing"
)

func TestAccessorReadsTheAllocatedEventMember(t *testing.T) {
	wanted := core.Body{Status: "wanted"}
	wrong := core.Body{Status: "wrong"}
	event := core.Arrived{Data: wrong, Data_: wanted, Choice: core.ChoiceGone{Value: "gone"}, Wrapped: core.NewWrapped(core.Wrapper{Body: wanted})}
	if input := Project(event); input.Text != "wanted" {
		t.Fatalf("read the other declared event member: %s", input.Text)
	}
}
