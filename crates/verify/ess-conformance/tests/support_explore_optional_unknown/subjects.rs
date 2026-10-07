pub fn source(upsert: bool) -> String {
    let alternatives = if upsert {
        r"      - name: updated
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Changed]
        payload: {demo.items.Changed: {id: input.id, note: input.note}}
        sets: {note: input.note}
      - name: created
        unknown_instance: true
        creates: demo.items.Item
        instance: id
        emits: [demo.items.Created]
        payload: {demo.items.Created: {id: input.id, note: input.note}}
        sets: {note: input.note}
"
    } else {
        r"      - {name: ordinary, error: demo.items.Conflict}
      - name: first-provider
        external: the first provider accepts
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Changed]
        payload: {demo.items.Changed: {id: input.id, note: input.note}}
        sets: {note: input.note}
      - name: second-provider
        external: the second provider accepts
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Changed]
        payload: {demo.items.Changed: {id: input.id, note: input.note}}
        sets: {note: input.note}
      - {name: not-found, unknown_instance: true, error: demo.items.NotFound}
"
    };
    format!(
        r"format: ess/16
system: demo
version: v1
domain: demo.items
types:
  - {{name: demo.items.ItemId, kind: newtype, of: Uuid}}
entities:
  - name: demo.items.Item
    identity: {{name: id, type: demo.items.ItemId}}
    fields: [{{name: note, type: Optional<String>}}]
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: demo.items.NotFound}}
  - {{name: demo.items.Conflict}}
events:
  - {{name: demo.items.Created, fields: [{{name: id, type: demo.items.ItemId}}, {{name: note, type: Optional<String>}}]}}
  - {{name: demo.items.Changed, fields: [{{name: id, type: demo.items.ItemId}}, {{name: note, type: Optional<String>}}]}}
commands:
  - name: demo.items.Create
    input: [{{name: note, type: Optional<String>}}]
    outcomes:
      - name: created
        creates: demo.items.Item
        instance: id
        emits: [demo.items.Created]
        payload: {{demo.items.Created: {{id: {{generated: true}}, note: input.note}}}}
        sets: {{note: input.note}}
  - name: demo.items.Put
    input: [{{name: id, type: demo.items.ItemId}}, {{name: note, type: Optional<String>}}]
    outcomes:
{alternatives}actors:
  - {{name: demo.items.Operator, may: [demo.items.Create, demo.items.Put]}}
views:
  - name: demo.items.Items
    source: demo.items.Item
    consistency: read_your_writes
    fields: [{{name: id, type: demo.items.ItemId}}, {{name: state, type: demo.items.Item.State}}, {{name: note, type: Optional<String>}}]
"
    )
}

pub const GO: &str = r#"package essconform
import("encoding/json";"fmt";"os";"sort";"testing")
type subjectTarget struct{Target; rows map[string]Node; next int; forced string; mutant string; calls *[]map[string]any}
func(p *subjectTarget) BeginScenario(ScenarioContext)error{p.rows=map[string]Node{};p.next=0;p.forced="";return nil}
func(p *subjectTarget) EndScenario(ScenarioContext)error{return nil}
func(p *subjectTarget) ConfigureExternalOutcome(r ExternalOutcomeControl)error{
 if p.calls!=nil{*p.calls=append(*p.calls,map[string]any{"arrange":r.Outcome})}
 if p.mutant=="unarrangeable"&&r.Outcome=="second-provider"{return ErrUnsupported}
 p.forced=r.Outcome;if p.mutant=="ignore"{p.forced=""};return nil
}
func(p *subjectTarget) QueryView(ViewRequest)(ViewResult,error){rows:=[]Row{};ids:=[]string{};for id:=range p.rows{ids=append(ids,id)};sort.Strings(ids);for _,id:=range ids{note:=p.rows[id];if p.mutant=="view-absence"&&note==nil{note="fabricated"};if p.mutant=="view-present"&&note!=nil{note=nil};rows=append(rows,Row{"id":id,"state":"Open","note":note})};return ViewResult{Rows:rows},nil}
func(p *subjectTarget) ExecuteCommand(r CommandRequest)(CommandResult,error){
 if p.calls!=nil{*p.calls=append(*p.calls,map[string]any{"command":r.Command,"input":r.Input})}
 event:=func(name,id,outcome string)CommandResult{return CommandResult{Outcome:outcome,DirectEvents:[]ObservedEvent{{Event:name,Payload:map[string]Node{"id":id,"note":r.Input["note"]}}}}}
 if r.Command=="demo.items.Create"{p.next++;id:=fmt.Sprintf("00000000-0000-4000-8000-%012d",p.next);p.rows[id]=r.Input["note"];return event("demo.items.Created",id,"created"),nil}
 id:=r.Input["id"].(string)
 if UPSERT {
  if _,exists:=p.rows[id];exists{p.rows[id]=r.Input["note"];outcome:="updated";if p.mutant=="update"{outcome="created"};return event("demo.items.Changed",id,outcome),nil}
  p.rows[id]=r.Input["note"];outcome:="created";if p.mutant=="create"{outcome="updated"};return event("demo.items.Created",id,outcome),nil
 }
 if p.forced==""{return CommandResult{Outcome:"ordinary",Error:"demo.items.Conflict"},nil}
 if _,exists:=p.rows[id];!exists{return CommandResult{Outcome:"not-found",Error:"demo.items.NotFound"},nil}
 p.rows[id]=r.Input["note"]
 return event("demo.items.Changed",id,p.forced),nil
}
func TestProbe(t *testing.T){calls:=[]map[string]any{};result,e:=Explore(func()Target{return &subjectTarget{calls:&calls}},ExploreOptions{Seeds:8,Steps:32});if e!=nil{t.Fatal(e)}
 mutants:=map[string]any{};for _,name:=range []string{"ignore","unarrangeable","create","update","view-absence","view-present"}{r,e:=Explore(func()Target{return &subjectTarget{mutant:name}},ExploreOptions{Seeds:8,Steps:32});if e!=nil{t.Fatal(e)};mutants[name]=r}
 raw,_:=json.Marshal(map[string]any{"calls":calls,"result":result,"mutants":mutants});if e:=os.WriteFile("../../go-result.json",raw,0600);e!=nil{t.Fatal(e)}
}
"#;

pub const TS: &str = r"import {writeFileSync} from 'node:fs';
import {explore} from './dist/explore.js';
import {ErrUnsupported} from './dist/runtime.js';
const target=(mutant='',calls)=>{let rows,next,forced;return {
 beginScenario(){rows=new Map();next=0;forced='';},endScenario(){},
 configureExternalOutcome({outcome}){if(calls)calls.push({arrange:outcome});if(mutant==='unarrangeable'&&outcome==='second-provider')throw ErrUnsupported;forced=mutant==='ignore'?'':outcome;},
 queryView(){return {rows:[...rows].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([id,note])=>({id,state:'Open',note:mutant==='view-absence'&&note==null?'fabricated':mutant==='view-present'&&note!=null?null:note}))};},
 executeCommand({command,input}){
  if(calls)calls.push({command,input});const event=(event,id,outcome)=>({outcome,directEvents:[{event,payload:{id,note:input.note}}]});
  if(command==='demo.items.Create'){const id=`00000000-0000-4000-8000-${String(++next).padStart(12,'0')}`;rows.set(id,input.note);return event('demo.items.Created',id,'created');}
  const id=input.id;
  if(UPSERT){const exists=rows.has(id);rows.set(id,input.note);return exists?event('demo.items.Changed',id,mutant==='update'?'created':'updated'):event('demo.items.Created',id,mutant==='create'?'updated':'created');}
  if(!forced)return {outcome:'ordinary',error:'demo.items.Conflict'};
  if(!rows.has(id))return {outcome:'not-found',error:'demo.items.NotFound'};
  rows.set(id,input.note);
  return event('demo.items.Changed',id,forced);
 }
};};
const calls=[];const result=await explore(()=>target('',calls),{seeds:8,steps:32});const mutants={};for(const name of ['ignore','unarrangeable','create','update','view-absence','view-present'])mutants[name]=await explore(()=>target(name),{seeds:8,steps:32});
writeFileSync('../../ts-result.json',JSON.stringify({calls,result,mutants}));
";
