package essconform

import (
 "fmt"
 "encoding/json"
 "os"
 "testing"
)

type historyBackend struct { row map[string]Node }
func (*historyBackend) Identity() (Identity,error) { return Identity{Name:"history",Version:"1"},nil }
func (b *historyBackend) BeginScenario(ScenarioContext) error { b.row=nil;return nil }
func (*historyBackend) EndScenario(ScenarioContext) error { return nil }
func (*historyBackend) ConfigureExternalOutcome(ExternalOutcomeControl) error { return fmt.Errorf("history is not externally forced") }
func (b *historyBackend) ExecuteCommand(r CommandRequest) (CommandResult,error) {
 result:=CommandResult{Consistency:"1"}
 emit:=func(name string,payload map[string]Node) { result.DirectEvents=[]ObservedEvent{{Event:"calls.core."+name,Payload:payload}} }
 // The unknown-instance rule: every command here declares `wrong_state` (`gone`), so one no
 // record carries is answered with it.
 if r.Command!="calls.core.Open" && (b.row==nil || r.Input["call_id"]!=b.row["call_id"]) {
  result.Outcome="gone";result.Error="calls.core.Gone";return result,nil
 }
 switch r.Command {
 case "calls.core.Open":
  b.row=map[string]Node{"call_id":"00000000-0000-4000-8000-000000000001","state":"Init","answer_history":"Unanswered","note":r.Input["note"],"server_stamp":json.Number("9007199254740992")}
  result.Outcome="opened";emit("Opened",map[string]Node{"call_id":b.row["call_id"]})
 case "calls.core.Answer":
  if b.row["state"]=="Ended" { result.Outcome="gone";result.Error="calls.core.Gone";break }
  if b.row["answer_history"]=="Answered" {
   result.Outcome="already-answered"
   switch os.Getenv("ESS_HISTORY_MUTANT") {
   case "error":result.Error="calls.core.Gone"
   case "rewrite":b.row["server_stamp"]=json.Number("9007199254740993")
   case "event":emit("Answered",map[string]Node{})
   case "lost_history":b.row["answer_history"]="Unanswered"
   }
  } else if os.Getenv("ESS_HISTORY_MUTANT")=="state_only" && b.row["state"]=="Bridged" {
   result.Outcome="already-answered"
  } else { b.row["state"]="Bridged";b.row["answer_history"]="Answered";result.Outcome="answered";emit("Answered",map[string]Node{}) }
 case "calls.core.Report", "calls.core.Close":
  if b.row["state"]=="Ended" {result.Outcome="gone";result.Error="calls.core.Gone";break}
  if r.Command=="calls.core.Report" {b.row["state"]="Bridged";result.Outcome="observed"} else {b.row["state"]="Ended";result.Outcome="closed"}
  emit("Observed",map[string]Node{})
 default:return result,fmt.Errorf("unknown command")
 }
 return result,nil
}
func (b *historyBackend) QueryView(ViewRequest) (ViewResult,error) {
 if b.row==nil {return ViewResult{},nil}
 rows:=[]Row{b.row}
 if os.Getenv("ESS_HISTORY_MUTANT")=="duplicate" {rows=append(rows,b.row)}
 if os.Getenv("ESS_HISTORY_MUTANT")=="missing" {rows=nil}
 return ViewResult{Rows:rows},nil
}
func (*historyBackend) ObserveEvents(EventObservationRequest) ([]ObservedEvent,error) {return nil,nil}
func (*historyBackend) RedeliverEvent(RedeliveryRequest) error {return ErrUnsupported}
func (*historyBackend) ObserveInvocations(InvocationObservationRequest) ([]Invocation,error) {return nil,ErrUnsupported}
func TestSubjectHistory(t *testing.T) {Run(t,func() Target {return &historyBackend{}})}
