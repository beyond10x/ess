package system

import (
	"example.invalid/projection/types/core"
	"testing"
)

func TestDeclaredNominalConversion(t *testing.T) {
	body := core.Body{Status: core.NewSourceTag("ready")}
	event := core.Arrived{Data: body, Choice: core.ChoiceGone{Value: core.NewSourceTag("gone")}, Wrapped: core.NewWrapped(core.Wrapper{Body: body})}
	if input := Project(event); input.Text.Value() != "ready" {
		t.Fatal("conversion lost terminal value")
	}
}
