// An in-memory implementation of `guard-pair-corpus.yaml` at `ess/29` that answers every command it
// serves in one declaration-order pass: the first declared branch whose condition holds answers,
// markers included (beyond10x/ess#554, `story:declaration-order-agrees-with-every-step-pair`). It
// serves the commands the explorer draws; one guarded by a related row is excluded from
// exploration and answered as unsupported.
//
// Modes:
//
//   - "" (declared): every command in declaration order.
//   - "vip-first": `SubjectBesideState` reads `vip` before `released`, the one departure from
//     declaration order the explorer must catch.

package essconform

import "fmt"

type exploreDeclUnsupported struct{ reason string }

func (e exploreDeclUnsupported) Error() string { return e.reason }
func (e exploreDeclUnsupported) Unwrap() error { return ErrUnsupported }

type exploreDeclSeat struct {
	id    string
	tier  Node
	kind  Node
	state string
}

type exploreDeclTier struct {
	id       string
	openness Node
	load     Node
}

// exploreDeclBranch is one declared branch: whether its condition holds for the seat the input
// names (nil where none carries it), and its answer.
type exploreDeclBranch struct {
	name   string
	holds  func(seat *exploreDeclSeat) bool
	answer func(seat *exploreDeclSeat) CommandResult
}

type exploreDeclTarget struct {
	mode    string
	seats   []*exploreDeclSeat
	tiers   []*exploreDeclTier
	created int
	version int
}

func newExploreDeclaredTarget(mode string) *exploreDeclTarget {
	return &exploreDeclTarget{mode: mode}
}

func (t *exploreDeclTarget) reset() {
	t.seats = nil
	t.tiers = nil
	t.created = 0
	t.version = 0
}

func (t *exploreDeclTarget) find(id Node) *exploreDeclSeat {
	for _, seat := range t.seats {
		if seat.id == id {
			return seat
		}
	}
	return nil
}

func (t *exploreDeclTarget) identity() string {
	t.created++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
}

func (t *exploreDeclTarget) answered(outcome, event, field, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{field: id}}},
	}
}

func exploreDeclRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

// pass answers with the first branch, in the order given, whose condition holds.
func exploreDeclPass(seat *exploreDeclSeat, branches []exploreDeclBranch) CommandResult {
	for _, branch := range branches {
		if branch.holds(seat) {
			return branch.answer(seat)
		}
	}
	panic("the last branch of every command holds")
}

func exploreDeclRefusal(name, err string, holds func(seat *exploreDeclSeat) bool) exploreDeclBranch {
	return exploreDeclBranch{name: name, holds: holds, answer: func(*exploreDeclSeat) CommandResult {
		return exploreDeclRefused(name, err)
	}}
}

func exploreDeclUnknown(name string) exploreDeclBranch {
	return exploreDeclRefusal(name, "demo.pairs.UnknownSeat", func(seat *exploreDeclSeat) bool { return seat == nil })
}

func exploreDeclAlways(*exploreDeclSeat) bool { return true }

func (t *exploreDeclTarget) moved(name, to, event string) exploreDeclBranch {
	return exploreDeclBranch{name: name, holds: exploreDeclAlways, answer: func(seat *exploreDeclSeat) CommandResult {
		seat.state = to
		return t.answered(name, event, "seat_id", seat.id)
	}}
}

func (t *exploreDeclTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-declared-order-target", Version: "1"}, nil
}

func (t *exploreDeclTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreDeclTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreDeclTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	seat := t.find(input["seat_id"])
	switch request.Command {
	case "demo.pairs.DefineTier":
		tier := &exploreDeclTier{id: t.identity(), openness: input["openness"], load: input["load"]}
		t.tiers = append(t.tiers, tier)
		return t.answered("defined", "demo.pairs.TierDefined", "tier_id", tier.id), nil
	case "demo.pairs.Issue":
		seat := &exploreDeclSeat{id: t.identity(), tier: input["tier_id"], kind: input["kind"], state: "Held"}
		t.seats = append(t.seats, seat)
		return t.answered("issued", "demo.pairs.Issued", "seat_id", seat.id), nil
	case "demo.pairs.Release":
		return exploreDeclPass(seat, []exploreDeclBranch{
			exploreDeclUnknown("release-unknown"),
			exploreDeclRefusal("release-not-held", "demo.pairs.SeatNotHeld", func(seat *exploreDeclSeat) bool { return seat.state != "Held" }),
			t.moved("released", "Released", "demo.pairs.Released"),
		}), nil
	case "demo.pairs.Retire":
		return exploreDeclPass(seat, []exploreDeclBranch{
			exploreDeclUnknown("retire-unknown"),
			exploreDeclRefusal("retire-not-released", "demo.pairs.SeatNotHeld", func(seat *exploreDeclSeat) bool { return seat.state != "Released" }),
			t.moved("retired", "Retired", "demo.pairs.Retired"),
		}), nil
	case "demo.pairs.SubjectBesideState":
		released := exploreDeclRefusal("released", "demo.pairs.SeatReleased", func(seat *exploreDeclSeat) bool {
			return seat.state == "Released" || seat.state == "Retired"
		})
		vip := exploreDeclRefusal("vip", "demo.pairs.VipLocked", func(seat *exploreDeclSeat) bool { return seat.kind == "vip" })
		changed := exploreDeclBranch{name: "changed", holds: exploreDeclAlways, answer: func(seat *exploreDeclSeat) CommandResult {
			seat.kind = input["kind"]
			return t.answered("changed", "demo.pairs.Changed", "seat_id", seat.id)
		}}
		declared := []exploreDeclBranch{exploreDeclUnknown("unknown-seat"), released, vip, changed}
		if t.mode == "vip-first" {
			declared = []exploreDeclBranch{exploreDeclUnknown("unknown-seat"), vip, released, changed}
		}
		return exploreDeclPass(seat, declared), nil
	case "demo.pairs.StateBesideWrongState":
		return exploreDeclPass(seat, []exploreDeclBranch{
			exploreDeclUnknown("unknown-seat"),
			exploreDeclRefusal("parked", "demo.pairs.SeatReleased", func(seat *exploreDeclSeat) bool { return seat.state == "Released" }),
			exploreDeclRefusal("not-held", "demo.pairs.SeatNotHeld", func(seat *exploreDeclSeat) bool { return seat.state != "Held" }),
			t.moved("released", "Released", "demo.pairs.Released"),
		}), nil
	default:
		return CommandResult{}, exploreDeclUnsupported{request.Command + " is not a command this target serves"}
	}
}

func (t *exploreDeclTarget) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "demo.pairs.Seats":
		for _, seat := range t.seats {
			rows = append(rows, Row{"seat_id": seat.id, "state": seat.state, "tier_id": seat.tier, "kind": seat.kind})
		}
	case "demo.pairs.Tiers":
		for _, tier := range t.tiers {
			rows = append(rows, Row{"tier_id": tier.id, "openness": tier.openness, "load": tier.load})
		}
	default:
		return ViewResult{}, exploreDeclUnsupported{request.View + " is not a view of guard-pair-corpus.yaml"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreDeclTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreDeclTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreDeclUnsupported{"no external outcome of a served command"}
}

func (t *exploreDeclTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreDeclUnsupported{"no redelivery"}
}

func (t *exploreDeclTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreDeclUnsupported{"no bindings"}
}
