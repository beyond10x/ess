// A small in-memory implementation of `explore-pair/pair.yaml`, two entities each created by its
// own command, for the concurrent explorer lanes in `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore-concurrent-pair-target.mjs` is the same target in TypeScript, line for line. It is not
// an InterleavedTarget, so every call takes effect at its return instant. The mutants:
//
//   box-lost                 every `OpenBox` opens its box and its answer never arrives
//   box-lost-phantom-crate   as `box-lost`, and each such `OpenBox` also opens a crate nobody asked for
//
// A lost box creation accounts for a box nobody names, never for a crate: under
// `box-lost-phantom-crate` a read of `Crates` showing the phantom is a read of an instance no
// operation made.

package essconform

import "fmt"

type explorePairTarget struct {
	mutant string
	boxes  []string
	crates []string
	minted int
}

func newExplorePairTarget(mutant string) *explorePairTarget {
	target := &explorePairTarget{mutant: mutant}
	target.reset()
	return target
}

func (t *explorePairTarget) reset() {
	t.boxes = nil
	t.crates = nil
	t.minted = 0
}

// mint is the next identity, shared by boxes and crates so no two instances share one.
func (t *explorePairTarget) mint() string {
	t.minted++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", t.minted)
}

func (t *explorePairTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-pair-target", Version: "1"}, nil
}

func (t *explorePairTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *explorePairTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *explorePairTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	switch request.Command {
	case "pair.core.OpenBox":
		id := t.mint()
		t.boxes = append(t.boxes, id)
		if t.mutant == "box-lost-phantom-crate" {
			t.crates = append(t.crates, t.mint())
		}
		if t.mutant == "box-lost" || t.mutant == "box-lost-phantom-crate" {
			return CommandResult{}, fmt.Errorf("the answer was lost: %w", ErrIndeterminate)
		}
		return CommandResult{Outcome: "opened", DirectEvents: []ObservedEvent{{Event: "pair.core.BoxOpened", Payload: map[string]Node{"box_id": id}}}}, nil
	case "pair.core.OpenCrate":
		id := t.mint()
		t.crates = append(t.crates, id)
		return CommandResult{Outcome: "opened", DirectEvents: []ObservedEvent{{Event: "pair.core.CrateOpened", Payload: map[string]Node{"crate_id": id}}}}, nil
	default:
		return CommandResult{}, exploreFixtureUnsupported{request.Command + " is not a command of explore-pair"}
	}
}

// QueryView answers `Boxes` and `Crates` with every instance, in the order it was created.
func (t *explorePairTarget) QueryView(request ViewRequest) (ViewResult, error) {
	ids, field := t.boxes, "box_id"
	switch request.View {
	case "pair.core.Boxes":
	case "pair.core.Crates":
		ids, field = t.crates, "crate_id"
	default:
		return ViewResult{}, exploreFixtureUnsupported{request.View + " is not a view of explore-pair"}
	}
	rows := []Row{}
	for _, id := range ids {
		rows = append(rows, Row{field: id})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *explorePairTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *explorePairTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreFixtureUnsupported{"no external outcome"}
}

func (t *explorePairTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreFixtureUnsupported{"no bindings"}
}

func (t *explorePairTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreFixtureUnsupported{"no bindings"}
}
