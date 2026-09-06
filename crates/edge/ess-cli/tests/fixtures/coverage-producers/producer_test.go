package billing

import (
	"encoding/json"
	"essbilling/essconform"
	"fmt"
	"os"
	"testing"
)

func recordFixture(method string, request any) {
	file, err := os.OpenFile(os.Getenv("ESS_FIXTURE_CALLBACKS"), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0600)
	if err != nil {
		panic(err)
	}
	defer file.Close()
	if err := json.NewEncoder(file).Encode(map[string]any{"method": method, "request": request}); err != nil {
		panic(err)
	}
}

type fixtureTarget struct{ *Target }

func (target *fixtureTarget) Identity() (essconform.Identity, error) {
	recordFixture("identity", nil)
	if os.Getenv("ESS_FIXTURE_TARGET") == "control" {
		return essconform.Identity{Name: "coverage-producer-control", Version: "1"}, nil
	}
	return target.Target.Identity()
}
func (target *fixtureTarget) BeginScenario(request essconform.ScenarioContext) error {
	recordFixture("begin", request)
	return target.Target.BeginScenario(request)
}
func (target *fixtureTarget) EndScenario(request essconform.ScenarioContext) error {
	recordFixture("end", request)
	if os.Getenv("ESS_FIXTURE_MODE") == "skip-then-teardown-error" {
		return essconform.ErrUnsupported
	}
	return target.Target.EndScenario(request)
}
func (target *fixtureTarget) ExecuteCommand(request essconform.CommandRequest) (essconform.CommandResult, error) {
	recordFixture("execute_command", request)
	if os.Getenv("ESS_FIXTURE_TARGET") == "control" {
		switch os.Getenv("ESS_FIXTURE_MODE") {
		case "failed":
			return essconform.CommandResult{}, fmt.Errorf("independent fixture command error")
		case "skipped", "skip-then-teardown-error":
			return essconform.CommandResult{}, essconform.ErrUnsupported
		default:
			return essconform.CommandResult{Outcome: "accepted"}, nil
		}
	}
	return target.Target.ExecuteCommand(request)
}
func (target *fixtureTarget) QueryView(request essconform.ViewRequest) (essconform.ViewResult, error) {
	recordFixture("query_view", request)
	return target.Target.QueryView(request)
}
func (target *fixtureTarget) ObserveEvents(request essconform.EventObservationRequest) ([]essconform.ObservedEvent, error) {
	recordFixture("observe_events", request)
	return target.Target.ObserveEvents(request)
}
func (target *fixtureTarget) ConfigureExternalOutcome(request essconform.ExternalOutcomeControl) error {
	recordFixture("configure_external_outcome", request)
	return target.Target.ConfigureExternalOutcome(request)
}
func (target *fixtureTarget) RedeliverEvent(request essconform.RedeliveryRequest) error {
	recordFixture("redeliver_event", request)
	return target.Target.RedeliverEvent(request)
}
func (target *fixtureTarget) ObserveInvocations(request essconform.InvocationObservationRequest) ([]essconform.Invocation, error) {
	recordFixture("observe_invocations", request)
	return target.Target.ObserveInvocations(request)
}
func (target *fixtureTarget) MarkInstant(request essconform.InstantMark) error {
	recordFixture("mark_instant", request)
	return target.Target.MarkInstant(request)
}
func (target *fixtureTarget) ObserveElapsed(request essconform.ElapsedRequest) (essconform.ElapsedObservation, error) {
	recordFixture("observe_elapsed", request)
	return target.Target.ObserveElapsed(request)
}
func (target *fixtureTarget) ScanView(request essconform.ScanRequest) (essconform.ScanObservation, error) {
	recordFixture("scan_view", request)
	return target.Target.ScanView(request)
}
func TestConformance(t *testing.T) {
	essconform.Run(t, func() essconform.Target {
		recordFixture("target_factory", nil)
		return &fixtureTarget{New("")}
	})
}
