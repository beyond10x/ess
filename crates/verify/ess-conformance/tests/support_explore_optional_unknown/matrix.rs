//! Input-derived echo targets: no suite assertions or model oracle are consulted by either target.

pub fn source() -> String {
    let fields = r"      - {name: gate, type: Optional<Boolean>, presence: null_when_absent}
      - {name: number, type: Optional<Integer>}
      - {name: text, type: Optional<String>, presence: omitted_when_absent}
      - {name: uuid, type: Optional<Uuid>}
      - {name: choice, type: Optional<demo.values.Choice>}
      - {name: label, type: Optional<demo.values.Label>}
      - {name: nested, type: 'Optional<Optional<String>>'}
      - {name: box, type: demo.values.Box}
      - {name: maybe_box, type: Optional<demo.values.Box>}
";
    let payload = [
        "gate",
        "number",
        "text",
        "uuid",
        "choice",
        "label",
        "nested",
        "box",
        "maybe_box",
    ]
    .map(|name| format!("{name}: input.{name}"))
    .join(", ");
    format!(
        r"format: ess/15
system: demo
version: v1
domain: demo.values
types:
  - {{name: demo.values.Label, kind: newtype, of: String}}
  - {{name: demo.values.Choice, kind: enum, variants: [First, Second]}}
  - name: demo.values.Box
    kind: struct
    fields:
      - {{name: required, type: Integer}}
      - {{name: omit, type: Optional<String>, presence: omitted_when_absent}}
      - {{name: nullable, type: Optional<Boolean>, presence: null_when_absent}}
events:
  - name: demo.values.Echoed
    fields:
{fields}commands:
  - name: demo.values.Echo
    input:
{fields}    outcomes:
      - name: present
        when: defined(gate)
        emits: [demo.values.Echoed]
        payload: {{demo.values.Echoed: {{{payload}}}}}
      - name: absent
        emits: [demo.values.Echoed]
        payload: {{demo.values.Echoed: {{{payload}}}}}
actors:
  - {{name: demo.values.Operator, may: [demo.values.Echo]}}
"
    )
}

pub const GO: &str = r#"package essconform
import("encoding/json";"os";"testing")
type matrixTarget struct { Target; mutant string; draws *[]map[string]any }
func(p *matrixTarget) BeginScenario(ScenarioContext)error{return nil}
func(p *matrixTarget) EndScenario(ScenarioContext)error{return nil}
func(p *matrixTarget) ExecuteCommand(r CommandRequest)(CommandResult,error){
 if p.draws!=nil{*p.draws=append(*p.draws,map[string]any{"command":r.Command,"input":r.Input})}
 raw,_:=json.Marshal(r.Input); payload:=map[string]Node{}; json.Unmarshal(raw,&payload)
 outcome:="present"; if r.Input["gate"]==nil{outcome="absent"}
 if p.mutant=="branch"{if outcome=="present"{outcome="absent"}else{outcome="present"}}
 box:=payload["box"].(map[string]any)
 if p.mutant=="fabricate" && box["omit"]==nil{box["omit"]="invented"}
 if p.mutant=="policy" {if _,ok:=box["omit"];!ok{box["omit"]=nil}}
 if p.mutant=="extra" {box["undeclared"]="fabricated"}
 return CommandResult{Outcome:outcome,DirectEvents:[]ObservedEvent{{Event:"demo.values.Echoed",Payload:payload}}},nil
}
func TestProbe(t *testing.T){
 draws:=[]map[string]any{}; healthy,err:=Explore(func()Target{return &matrixTarget{draws:&draws}},ExploreOptions{Seeds:4,Steps:32});if err!=nil{t.Fatal(err)}
 mutants:=map[string]any{};for _,name:=range []string{"branch","fabricate","policy","extra"}{r,e:=Explore(func()Target{return &matrixTarget{mutant:name}},ExploreOptions{Seeds:4,Steps:32});if e!=nil{t.Fatal(e)};mutants[name]=r}
 raw,_:=json.Marshal(map[string]any{"draws":draws,"result":healthy,"mutants":mutants});if e:=os.WriteFile("../../go-result.json",raw,0600);e!=nil{t.Fatal(e)}
}
"#;

pub const TS: &str = r"import {writeFileSync} from 'node:fs';
import {explore} from './dist/explore.js';
const target=(mutant='',draws)=>({beginScenario(){},endScenario(){},executeCommand({command,input}){
 if(draws)draws.push({command,input});const payload=structuredClone(input);
 let outcome=input.gate==null?'absent':'present';if(mutant==='branch')outcome=outcome==='present'?'absent':'present';
 if(mutant==='fabricate'&&payload.box.omit==null)payload.box.omit='invented';
 if(mutant==='policy'&&!Object.hasOwn(payload.box,'omit'))payload.box.omit=null;
 if(mutant==='extra')payload.box.undeclared='fabricated';
 return {outcome,directEvents:[{event:'demo.values.Echoed',payload}]};
}});
const draws=[];const result=await explore(()=>target('',draws),{seeds:4,steps:32});
const mutants={};for(const name of ['branch','fabricate','policy','extra'])mutants[name]=await explore(()=>target(name),{seeds:4,steps:32});
writeFileSync('../../ts-result.json',JSON.stringify({draws,result,mutants}));
";
