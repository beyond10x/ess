// The emitted Go runner's Target, over the gatepass Go module linked with
// examples/gatepass-go-realization.
//
// An adapter, not an implementation: suite values into the generated types on the way in, typed
// outcomes, events and rows back into suite values on the way out. Every observation is read off
// the linked system — its log and its view ports; nothing is decided here. The Rust twin of this
// file is `Synthesized` in `tests/conformance.rs`.
package gatepass

import (
	"encoding/base64"
	"fmt"
	"math"
	"strconv"

	"essgatepass/essconform"

	realization "example.invalid/gatepass-realization"
	"example.invalid/gatepass/system"
	"example.invalid/gatepass/types/obligation"
	"example.invalid/gatepass/types/primitives"
	"example.invalid/gatepass/types/visit"
)

// Target is the linked gatepass system, relinked for every scenario.
type Target struct {
	assembled *realization.Assembled
}

// New is a target with no scenario open.
func New() *Target {
	return &Target{}
}

func (t *Target) Identity() (essconform.Identity, error) {
	return essconform.Identity{Name: "gatepass-go-synthesized", Version: "v1"}, nil
}

func (t *Target) BeginScenario(essconform.ScenarioContext) error {
	assembled, err := realization.Link()
	if err != nil {
		return err
	}
	t.assembled = assembled
	return nil
}

func (t *Target) EndScenario(essconform.ScenarioContext) error {
	t.assembled = nil
	return nil
}

func (t *Target) ConfigureExternalOutcome(control essconform.ExternalOutcomeControl) error {
	return fmt.Errorf("`examples/gatepass/` declares no external outcome to force `%s/%s`", control.Command, control.Outcome)
}

func (t *Target) RedeliverEvent(request essconform.RedeliveryRequest) error {
	return fmt.Errorf("`examples/gatepass/` declares no binding to deliver `%s` to", request.Event)
}

func (t *Target) ObserveInvocations(essconform.InvocationObservationRequest) ([]essconform.Invocation, error) {
	return nil, nil
}

// unmet is an unmet obligation surfacing to the runner — on the served surface, the 501 naming it.
func unmet(refusal *obligation.UnmetObligation) error {
	return fmt.Errorf("driving the linked system: the %s `%s` is not implemented", refusal.Capability, refusal.Source)
}

func (t *Target) ExecuteCommand(request essconform.CommandRequest) (essconform.CommandResult, error) {
	if t.assembled == nil {
		return essconform.CommandResult{}, fmt.Errorf("no scenario is open")
	}
	var result essconform.CommandResult
	var err error
	switch request.Command {
	case "gatepass.visit.RegisterVisit":
		result, err = t.registerVisit(request.Input)
	case "gatepass.visit.AdmitVisitor":
		result, err = t.admitVisitor(request.Input)
	case "gatepass.visit.SignOutVisitor":
		result, err = t.signOutVisitor(request.Input)
	default:
		return essconform.CommandResult{}, fmt.Errorf("`%s` is not a command `examples/gatepass/` declares", request.Command)
	}
	if err != nil {
		return essconform.CommandResult{}, err
	}
	if refusal := t.assembled.System.Pump(); refusal != nil {
		return essconform.CommandResult{}, unmet(refusal)
	}
	return result, nil
}

func (t *Target) registerVisit(input map[string]essconform.Node) (essconform.CommandResult, error) {
	command, err := registration(input)
	if err != nil {
		return essconform.CommandResult{}, err
	}
	outcome, refusal := t.assembled.System.PassService.RegisterVisit(command)
	if refusal != nil {
		return essconform.CommandResult{}, unmet(refusal)
	}
	switch taken := outcome.(type) {
	case visit.RegisterVisitOutcomeRegistered:
		return essconform.CommandResult{
			Outcome:      "registered",
			DirectEvents: []essconform.ObservedEvent{observed(system.SystemEventVisitRegistered{Event: taken.VisitRegistered})},
		}, nil
	case visit.RegisterVisitOutcomeRefused:
		return essconform.CommandResult{Outcome: "refused", Error: "gatepass.visit.InvalidVisitLength"}, nil
	}
	return essconform.CommandResult{}, fmt.Errorf("RegisterVisit answered %T, which it does not declare", outcome)
}

func (t *Target) admitVisitor(input map[string]essconform.Node) (essconform.CommandResult, error) {
	id, err := visitID(input)
	if err != nil {
		return essconform.CommandResult{}, err
	}
	printed, err := badge(input["badge"])
	if err != nil {
		return essconform.CommandResult{}, err
	}
	outcome, refusal := t.assembled.System.PassService.AdmitVisitor(visit.AdmitVisitor{VisitId: id, Badge: printed})
	if refusal != nil {
		return essconform.CommandResult{}, unmet(refusal)
	}
	switch taken := outcome.(type) {
	case visit.AdmitVisitorOutcomeAdmitted:
		return essconform.CommandResult{
			Outcome:      "admitted",
			DirectEvents: []essconform.ObservedEvent{observed(system.SystemEventVisitorAdmitted{Event: taken.VisitorAdmitted})},
		}, nil
	case visit.AdmitVisitorOutcomeWrongState:
		return essconform.CommandResult{Outcome: "wrong-state", Error: "gatepass.visit.VisitStateConflict"}, nil
	case visit.AdmitVisitorOutcomeWrongStateUnknownInstance:
		return essconform.CommandResult{Outcome: "wrong-state", Error: "gatepass.visit.VisitStateConflict"}, nil
	}
	return essconform.CommandResult{}, fmt.Errorf("AdmitVisitor answered %T, which it does not declare", outcome)
}

func (t *Target) signOutVisitor(input map[string]essconform.Node) (essconform.CommandResult, error) {
	id, err := visitID(input)
	if err != nil {
		return essconform.CommandResult{}, err
	}
	outcome, refusal := t.assembled.System.PassService.SignOutVisitor(visit.SignOutVisitor{VisitId: id})
	if refusal != nil {
		return essconform.CommandResult{}, unmet(refusal)
	}
	switch taken := outcome.(type) {
	case visit.SignOutVisitorOutcomeSignedOut:
		return essconform.CommandResult{
			Outcome:      "signed-out",
			DirectEvents: []essconform.ObservedEvent{observed(system.SystemEventVisitorDeparted{Event: taken.VisitorDeparted})},
		}, nil
	case visit.SignOutVisitorOutcomeWrongState:
		return essconform.CommandResult{Outcome: "wrong-state", Error: "gatepass.visit.VisitStateConflict"}, nil
	case visit.SignOutVisitorOutcomeWrongStateUnknownInstance:
		return essconform.CommandResult{Outcome: "wrong-state", Error: "gatepass.visit.VisitStateConflict"}, nil
	}
	return essconform.CommandResult{}, fmt.Errorf("SignOutVisitor answered %T, which it does not declare", outcome)
}

func (t *Target) QueryView(request essconform.ViewRequest) (essconform.ViewResult, error) {
	if t.assembled == nil {
		return essconform.ViewResult{}, fmt.Errorf("no scenario is open")
	}
	service := t.assembled.System.PassService
	switch request.View {
	case "gatepass.visit.ExpectedVisits":
		rows, refusal := service.ExpectedVisits()
		if refusal != nil {
			return essconform.ViewResult{}, unmet(refusal)
		}
		out := make([]essconform.Row, 0, len(rows))
		for _, row := range rows {
			out = append(out, essconform.Row{
				"visit_id": row.VisitId.Value().Value(),
				"visitor":  row.Visitor.Value(),
				"building": buildingName(row.Building),
				"deposit":  depositNode(row.Deposit),
			})
		}
		return essconform.ViewResult{Rows: out}, nil
	case "gatepass.visit.VisitById":
		rows, refusal := service.VisitById()
		if refusal != nil {
			return essconform.ViewResult{}, unmet(refusal)
		}
		out := make([]essconform.Row, 0, len(rows))
		for _, row := range rows {
			escorts := make([]essconform.Node, 0, len(row.Escorts))
			for _, name := range row.Escorts {
				escorts = append(escorts, name.Value())
			}
			notes := map[string]essconform.Node{}
			for key, value := range row.Notes {
				notes[key] = value
			}
			var held essconform.Node
			if row.Badge != nil {
				held = badgeNode(*row.Badge)
			}
			out = append(out, essconform.Row{
				"visit_id": row.VisitId.Value().Value(),
				"visitor":  row.Visitor.Value(),
				"host":     hostNode(row.Host),
				"escorts":  escorts,
				"notes":    notes,
				"badge":    held,
			})
		}
		return essconform.ViewResult{Rows: out}, nil
	}
	return essconform.ViewResult{}, fmt.Errorf("`%s` is not a view `examples/gatepass/` declares", request.View)
}

func (t *Target) ObserveEvents(request essconform.EventObservationRequest) ([]essconform.ObservedEvent, error) {
	if t.assembled == nil {
		return nil, fmt.Errorf("no scenario is open")
	}
	var out []essconform.ObservedEvent
	for _, event := range t.assembled.System.Published() {
		if rendered := observed(event); rendered.Event == request.Event {
			out = append(out, rendered)
		}
	}
	return out, nil
}

// ---- representation: nodes in -----------------------------------------------------------------

func text(input map[string]essconform.Node, name string) (string, error) {
	value, ok := input[name].(string)
	if !ok {
		return "", fmt.Errorf("the input `%s` is not text", name)
	}
	return value, nil
}

func visitID(input map[string]essconform.Node) (visit.VisitId, error) {
	id, err := text(input, "visit_id")
	if err != nil {
		return visit.VisitId{}, err
	}
	return visit.NewVisitId(primitives.NewUuid(id)), nil
}

func registration(input map[string]essconform.Node) (visit.RegisterVisit, error) {
	var command visit.RegisterVisit
	visitor, err := text(input, "visitor")
	if err != nil {
		return command, err
	}
	command.Visitor = visit.NewVisitorName(visitor)
	building, err := text(input, "building")
	if err != nil {
		return command, err
	}
	switch building {
	case "North":
		command.Building = visit.BuildingNorth{}
	case "South":
		command.Building = visit.BuildingSouth{}
	case "Annex":
		command.Building = visit.BuildingAnnex{}
	default:
		return command, fmt.Errorf("`%s` is not a declared Building", building)
	}
	host, ok := input["host"].(map[string]essconform.Node)
	if !ok {
		return command, fmt.Errorf("the input `host` is not a tagged map")
	}
	value, _ := host["value"].(string)
	switch host["kind"] {
	case "employee":
		command.Host = visit.HostEmployee{Value: visit.NewEmployeeId(value)}
	case "contractor":
		command.Host = visit.HostContractor{Value: visit.NewVendorRef(value)}
	default:
		return command, fmt.Errorf("the input `host.kind` is %v", host["kind"])
	}
	minutes, ok := input["expected_minutes"].(float64)
	if !ok || minutes != math.Trunc(minutes) {
		return command, fmt.Errorf("the input `expected_minutes` is not an integral number")
	}
	command.ExpectedMinutes = int64(minutes)
	stay, err := text(input, "expected_stay")
	if err != nil {
		return command, err
	}
	command.ExpectedStay = primitives.NewDuration(stay)
	deposit, ok := input["deposit"].(map[string]essconform.Node)
	if !ok {
		return command, fmt.Errorf("the input `deposit` is not a map")
	}
	amount, ok := deposit["amount"].(float64)
	if !ok {
		return command, fmt.Errorf("the input `deposit.amount` is not a number")
	}
	currency, _ := deposit["currency"].(string)
	command.Deposit = visit.Deposit{
		Amount:   primitives.NewDecimal(strconv.FormatFloat(amount, 'f', -1, 64)),
		Currency: currency,
	}
	escorts, ok := input["escorts"].([]essconform.Node)
	if !ok {
		return command, fmt.Errorf("the input `escorts` is not a list")
	}
	for _, escort := range escorts {
		name, _ := escort.(string)
		command.Escorts = append(command.Escorts, visit.NewVisitorName(name))
	}
	notes, ok := input["notes"].(map[string]essconform.Node)
	if !ok {
		return command, fmt.Errorf("the input `notes` is not a map")
	}
	command.Notes = map[string]string{}
	for key, note := range notes {
		command.Notes[key], _ = note.(string)
	}
	watched, ok := input["on_watchlist"].(bool)
	if !ok {
		return command, fmt.Errorf("the input `on_watchlist` is not a boolean")
	}
	command.OnWatchlist = watched
	return command, nil
}

func badge(node essconform.Node) (visit.Badge, error) {
	fields, ok := node.(map[string]essconform.Node)
	if !ok {
		return visit.Badge{}, fmt.Errorf("the input `badge` is not a map")
	}
	serial, _ := fields["serial"].(string)
	encoded, _ := fields["signature"].(string)
	signature, err := base64.StdEncoding.DecodeString(encoded)
	if err != nil {
		return visit.Badge{}, fmt.Errorf("the input `badge.signature` is not base64: %v", err)
	}
	held := visit.Badge{Serial: serial, Signature: signature}
	if at, ok := fields["printed_at"].(string); ok {
		printed := primitives.NewTimestamp(at)
		held.PrintedAt = &printed
	}
	return held, nil
}

// ---- representation: nodes out ----------------------------------------------------------------

func buildingName(building visit.Building) essconform.Node {
	switch building.(type) {
	case visit.BuildingNorth:
		return "North"
	case visit.BuildingSouth:
		return "South"
	case visit.BuildingAnnex:
		return "Annex"
	}
	return nil
}

func hostNode(host visit.Host) essconform.Node {
	switch shape := host.(type) {
	case visit.HostEmployee:
		return map[string]essconform.Node{"kind": "employee", "value": shape.Value.Value()}
	case visit.HostContractor:
		return map[string]essconform.Node{"kind": "contractor", "value": shape.Value.Value()}
	}
	return nil
}

func depositNode(deposit visit.Deposit) essconform.Node {
	var amount essconform.Node = deposit.Amount.Value()
	if number, err := strconv.ParseFloat(deposit.Amount.Value(), 64); err == nil {
		amount = number
	}
	return map[string]essconform.Node{"amount": amount, "currency": deposit.Currency}
}

func badgeNode(held visit.Badge) essconform.Node {
	var printed essconform.Node
	if held.PrintedAt != nil {
		printed = held.PrintedAt.Value()
	}
	return map[string]essconform.Node{
		"serial":     held.Serial,
		"printed_at": printed,
		"signature":  base64.StdEncoding.EncodeToString(held.Signature),
	}
}

func observed(event system.SystemEvent) essconform.ObservedEvent {
	switch value := event.(type) {
	case system.SystemEventVisitRegistered:
		return essconform.ObservedEvent{Event: "gatepass.visit.VisitRegistered", Payload: map[string]essconform.Node{
			"visit_id": value.Event.VisitId.Value().Value(),
			"visitor":  value.Event.Visitor.Value(),
			"building": buildingName(value.Event.Building),
		}}
	case system.SystemEventVisitorAdmitted:
		return essconform.ObservedEvent{Event: "gatepass.visit.VisitorAdmitted", Payload: map[string]essconform.Node{
			"visit_id": value.Event.VisitId.Value().Value(),
			"badge":    badgeNode(value.Event.Badge),
		}}
	case system.SystemEventVisitorDeparted:
		return essconform.ObservedEvent{Event: "gatepass.visit.VisitorDeparted", Payload: map[string]essconform.Node{
			"visit_id": value.Event.VisitId.Value().Value(),
		}}
	}
	return essconform.ObservedEvent{}
}
