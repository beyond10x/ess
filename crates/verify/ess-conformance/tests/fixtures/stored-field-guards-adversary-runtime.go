package essconform

import (
	"fmt"
	"os"
	"testing"
)

type parcels struct {
	rows   map[string]map[string]Node
	minted int
}

func (*parcels) Identity() (Identity, error) { return Identity{Name: "parcels", Version: "1"}, nil }
func (p *parcels) BeginScenario(ScenarioContext) error {
	p.rows = map[string]map[string]Node{}
	return nil
}
func (*parcels) EndScenario(ScenarioContext) error { return nil }
func (*parcels) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("not externally forced")
}

func refuses(row map[string]Node) bool {
	weight, _ := asNumber(row["weight_kg"])
	express := row["service"] == "Express"
	switch os.Getenv("ESS_PARCELS_MUTANT") {
	case "weight":
		return express
	case "fields":
		return false
	case "service":
		return weight > 20
	}
	return express && weight > 20
}

func (p *parcels) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	p.minted++
	result := CommandResult{Consistency: fmt.Sprintf("%d", p.minted)}
	switch r.Command {
	case "shipping.parcel.Create":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", p.minted)
		row := map[string]Node{}
		for k, v := range r.Input {
			row[k] = v
		}
		row["parcel_id"] = id
		row["state"] = "Created"
		p.rows[id] = row
		result.Outcome = "created"
		result.DirectEvents = []ObservedEvent{{Event: "shipping.parcel.Created", Payload: map[string]Node{"parcel_id": id}}}
	case "shipping.parcel.Dispatch":
		id, _ := r.Input["parcel_id"].(string)
		row, ok := p.rows[id]
		if !ok || row["state"] != "Created" {
			return result, nil
		}
		if refuses(row) {
			result.Outcome = "refused-overweight"
			result.Error = "shipping.parcel.ExpressOverweight"
			return result, nil
		}
		row["state"] = "Dispatched"
		result.Outcome = "dispatched"
		result.DirectEvents = []ObservedEvent{{Event: "shipping.parcel.Dispatched", Payload: map[string]Node{}}}
	default:
		return result, fmt.Errorf("unknown command %s", r.Command)
	}
	return result, nil
}

func (p *parcels) QueryView(ViewRequest) (ViewResult, error) {
	rows := []Row{}
	for _, row := range p.rows {
		rows = append(rows, row)
	}
	return ViewResult{Rows: rows}, nil
}
func (*parcels) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) { return nil, nil }
func (*parcels) RedeliverEvent(RedeliveryRequest) error                        { return ErrUnsupported }
func (*parcels) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestParcels(t *testing.T) { Run(t, func() Target { return &parcels{} }) }
