package essconform

import (
	"fmt"
	"testing"
)

// providedTarget is a deployment holding the app rows the suite's entity fixtures name.
type providedTarget struct {
	app   string
	links int
	rows  []Row
}

const providedApp = "6f1c3c55-2c4e-4c0e-9a51-1a8a3c1e6b01"

func (*providedTarget) Identity() (Identity, error) {
	return Identity{Name: "provided", Version: "1"}, nil
}
func (p *providedTarget) FixtureValues(_ ScenarioContext, contract FixtureContract) (map[string]Node, error) {
	values := map[string]Node{}
	for _, field := range contract.Fields {
		if field.Entity != "linking.links.App" || field.State != "Active" || field.Type != "linking.links.AppId" {
			return nil, fmt.Errorf("unexpected fixture %+v", field)
		}
		fmt.Printf("provided %s %s\n", field.Entity, field.State)
		p.app = providedApp
		values[field.Name] = providedApp
	}
	return values, nil
}
func (*providedTarget) BeginScenario(ScenarioContext) error { return nil }
func (p *providedTarget) EndScenario(ScenarioContext) error {
	p.app = ""
	p.rows = nil
	return nil
}
func (p *providedTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	app := request.Input["app_id"]
	if p.app == "" || app != p.app {
		return CommandResult{
			Outcome:      "unknown-app",
			Error:        "linking.links.AppNotFound",
			ErrorPayload: map[string]Node{"app_id": app},
		}, nil
	}
	p.links++
	link := fmt.Sprintf("00000000-0000-4000-8000-%012d", p.links)
	p.rows = append(p.rows, Row{"link_id": link, "app_id": app})
	return CommandResult{
		Outcome:     "linked",
		Consistency: fmt.Sprint(p.links),
		DirectEvents: []ObservedEvent{{
			Event:   "linking.links.Linked",
			Payload: map[string]Node{"link_id": link, "app_id": app},
		}},
	}, nil
}
func (p *providedTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "linking.links.Links" {
		return ViewResult{}, ErrUnsupported
	}
	return ViewResult{Rows: p.rows}, nil
}
func (*providedTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (*providedTarget) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (*providedTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (*providedTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestProvidedRows(t *testing.T) {
	Run(t, func() Target { return &providedTarget{} })
}

// An entity fixture is admitted from suite/60 and refused below it by its unknown key.
func TestEntityFixtureMajor(t *testing.T) {
	contract := func() any {
		return map[string]any{
			"fields": []any{map[string]any{
				"name": "app", "type": "Uuid", "entity": "linking.links.App", "state": "Active",
			}},
			"declarations": map[string]any{},
		}
	}
	if err := admitFixtures(contract(), 60); err != nil {
		t.Fatalf("suite/60 admits an entity fixture: %v", err)
	}
	if err := admitFixtures(contract(), 58); err == nil {
		t.Fatal("suite/58 admitted an entity fixture")
	}
	half := contract()
	delete(half.(map[string]any)["fields"].([]any)[0].(map[string]any), "state")
	if err := admitFixtures(half, 60); err == nil {
		t.Fatal("an entity fixture without its state was admitted")
	}
	fmt.Println("entity fixture admission checked")
}
