package essconform

import (
	"fmt"
	"testing"
)

// An entity fixture whose entity and state are both empty text names no entity; the Rust reader
// refuses it, and so must this runtime, rather than reading it as a type-only fixture.
func TestAdversaryD325EmptyEntityFixture(t *testing.T) {
	contract := map[string]any{
		"fields": []any{map[string]any{
			"name": "app", "type": "Uuid", "entity": "", "state": "",
		}},
		"declarations": map[string]any{},
	}
	if err := admitFixtures(contract, 60); err == nil {
		t.Fatal("suite/60 admitted an entity fixture naming the empty entity and the empty state")
	}
	fmt.Println("empty entity fixture refused")
}
