package essconform

import (
 "fmt"
 "os"
 "testing"
)

type guardedBackend struct { forced bool }
func (*guardedBackend) Identity() (Identity, error) { return Identity{Name:"guarded",Version:"1"},nil }
func (b *guardedBackend) BeginScenario(ScenarioContext) error { b.forced=false;return nil }
func (*guardedBackend) EndScenario(ScenarioContext) error { return nil }
func (b *guardedBackend) ConfigureExternalOutcome(ExternalOutcomeControl) error { b.forced=true;return nil }
func (b *guardedBackend) ExecuteCommand(r CommandRequest) (CommandResult,error) {
 if b.forced && r.Input["retry"] == false && r.Input["recipient"] != "" && os.Getenv("ESS_IGNORE_FAULT") != "1" {
  return CommandResult{Outcome:"rejected",Error:"delivery.mail.Rejected"},nil
 }
 return CommandResult{Outcome:"sent",DirectEvents:[]ObservedEvent{{Event:"delivery.mail.Sent",Payload:map[string]Node{}}}},nil
}
func (*guardedBackend) QueryView(ViewRequest) (ViewResult,error) { return ViewResult{},fmt.Errorf("no views") }
func (*guardedBackend) ObserveEvents(EventObservationRequest) ([]ObservedEvent,error) { return nil,nil }
func (*guardedBackend) RedeliverEvent(RedeliveryRequest) error { return fmt.Errorf("no bindings") }
func (*guardedBackend) ObserveInvocations(InvocationObservationRequest) ([]Invocation,error) { return nil,ErrUnsupported }
func TestGuardedExternal(t *testing.T) { Run(t,func() Target { return &guardedBackend{} }) }
