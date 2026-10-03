//! A precondition can send an absent Optional even when its present kind is not drawable.

pub const SOURCE: &str = r"format: ess/16
system: setup
version: v1
domain: setup.core
preconditions:
  - command: setup.core.Echo
    input: {}
events:
  - {name: setup.core.Pinged}
  - name: setup.core.Echoed
    fields: [{name: values, type: 'Optional<List<Integer>>'}]
actors:
  - {name: setup.core.Agent, may: [setup.core.Echo, setup.core.Ping]}
commands:
  - name: setup.core.Echo
    input: [{name: values, type: 'Optional<List<Integer>>'}]
    outcomes:
      - name: echoed
        emits: [setup.core.Echoed]
        payload: {setup.core.Echoed: {values: input.values}}
  - name: setup.core.Ping
    outcomes: [{name: ok, emits: [setup.core.Pinged]}]
";

pub const GO: &str = r#"package essconform
import ("encoding/json"; "os"; "testing")
type setupTarget struct { Target; broken bool; calls *int }
func (p *setupTarget) BeginScenario(ScenarioContext) error { return nil }
func (p *setupTarget) EndScenario(ScenarioContext) error { return nil }
func (p *setupTarget) ExecuteCommand(r CommandRequest) (CommandResult,error) {
  if r.Command=="setup.core.Ping" { return CommandResult{Outcome:"ok",DirectEvents:[]ObservedEvent{{Event:"setup.core.Pinged"}}},nil }
  *p.calls++
  value:=r.Input["values"]
  if p.broken { value=[]any{1} }
  return CommandResult{Outcome:"echoed",DirectEvents:[]ObservedEvent{{Event:"setup.core.Echoed",Payload:map[string]Node{"values":value}}}},nil
}
func TestProbe(t *testing.T) {
  healthyCalls,brokenCalls:=0,0
  healthy,err:=Explore(func() Target{return &setupTarget{calls:&healthyCalls}},ExploreOptions{Seeds:1,Steps:2})
  if err!=nil { t.Fatal(err) }
  _,refusal:=Explore(func() Target{return &setupTarget{broken:true,calls:&brokenCalls}},ExploreOptions{Seeds:1,Steps:2})
  value:=map[string]any{"healthy":healthy,"healthyCalls":healthyCalls,"brokenCalls":brokenCalls,"refused":refusal!=nil}
  raw,err:=json.Marshal(value); if err!=nil { t.Fatal(err) }
  if err=os.WriteFile("../../go-result.json",raw,0600);err!=nil {t.Fatal(err)}
}
"#;

pub const TS: &str = r"import {writeFileSync} from 'node:fs';
import {explore} from './dist/explore.js';
let healthyCalls=0,brokenCalls=0;
const target=(broken)=>({
  beginScenario(){},endScenario(){},
  executeCommand({command,input}){
    if(command==='setup.core.Ping') return {outcome:'ok',directEvents:[{event:'setup.core.Pinged',payload:{}}]};
    if(broken) brokenCalls++; else healthyCalls++;
    return {outcome:'echoed',directEvents:[{event:'setup.core.Echoed',payload:{values:broken?[1]:(input.values??null)}}]};
  }
});
const healthy=await explore(()=>target(false),{seeds:1,steps:2});
let refused=false;
try {await explore(()=>target(true),{seeds:1,steps:2});} catch {refused=true;}
writeFileSync('../../ts-result.json',JSON.stringify({healthy,healthyCalls,brokenCalls,refused}));
";
