pub mod matrix;
pub mod precondition;
pub mod subjects;

pub const GO: &str = r#"package essconform

import (
  "encoding/json"
  "fmt"
  "os"
  "testing"
)

type probeTarget struct {
  Target
  calls map[string]int
  rows map[string]string
  next int
  broken string
  draws *[]map[string]any
}

func (p *probeTarget) BeginScenario(ScenarioContext) error { p.rows=map[string]string{}; p.next=0; return nil }
func (p *probeTarget) EndScenario(ScenarioContext) error { return nil }
func (p *probeTarget) QueryView(r ViewRequest) (ViewResult,error) {
  if r.View!="demo.items.Items" { return ViewResult{},fmt.Errorf("unexpected view %s",r.View) }
  rows:=[]Row{}
  for id,state:=range p.rows { rows=append(rows,Row{"id":id,"state":state}) }
  return ViewResult{Rows:rows},nil
}
func (p *probeTarget) ExecuteCommand(r CommandRequest) (CommandResult,error) {
  p.calls[r.Command]++
  if p.draws!=nil { *p.draws=append(*p.draws,map[string]any{"command":r.Command,"input":r.Input}) }
  switch r.Command {
  case "demo.items.Create":
    p.next++
    id:=fmt.Sprintf("00000000-0000-4000-8000-%012d",p.next)
    p.rows[id]="Open"
    return CommandResult{Outcome:"created",DirectEvents:[]ObservedEvent{{Event:"demo.items.Created",Payload:map[string]Node{"id":id}}}},nil
  case "demo.items.PlainNote", "demo.items.OptionalNote":
    flag:=r.Input["flag"]
    if p.broken=="plain" && r.Command=="demo.items.PlainNote" { flag=!(flag.(bool)) }
    note:=r.Input["note"]
    if p.broken=="absent" && r.Command=="demo.items.OptionalNote" && note==nil { note="fabricated" }
    if p.broken=="present" && r.Command=="demo.items.OptionalNote" && note!=nil { note=nil }
    return CommandResult{Outcome:"noted",DirectEvents:[]ObservedEvent{{Event:"demo.items.Noted",Payload:map[string]Node{"flag":flag,"note":note}}}},nil
  case "demo.items.Close":
    if r.Input["deny"]==true && p.broken!="input-precedence" { return CommandResult{Outcome:"denied",Error:"demo.items.Conflict"},nil }
    id:=r.Input["id"].(string)
    state,ok:=p.rows[id]
    if !ok {
      if p.broken=="missing-outcome" { return CommandResult{Outcome:"closed"},nil }
      if p.broken=="missing-error" { return CommandResult{Outcome:"not-found",Error:"demo.items.Conflict"},nil }
      if p.broken=="missing-write" { p.rows[id]="Closed" }
      return CommandResult{Outcome:"not-found",Error:"demo.items.NotFound"},nil
    }
    if state!="Open" { return CommandResult{Outcome:"wrong-state",Error:"demo.items.Conflict"},nil }
    p.rows[id]="Closed"
    return CommandResult{Outcome:"closed",DirectEvents:[]ObservedEvent{{Event:"demo.items.Closed",Payload:map[string]Node{"id":id}}}},nil
  }
  return CommandResult{},fmt.Errorf("unexpected command %s",r.Command)
}

func TestProbe(t *testing.T) {
  ir,err:=exploreLoad(exploreIR,exploreSuite)
  if err!=nil { t.Fatal(err) }
  p:=explorePlanOf(ir)
  drawn:=[]string{}
  for _,command:=range p.commands { drawn=append(drawn,command.name) }
  calls:=map[string]int{}
  draws:=[]map[string]any{}
  result,err:=Explore(func() Target { return &probeTarget{calls:calls,draws:&draws} },ExploreOptions{Seeds:4,Steps:32})
  if err!=nil { t.Fatal(err) }
  control,err:=Explore(func() Target { return &probeTarget{calls:map[string]int{},broken:"plain"} },ExploreOptions{Seeds:4,Steps:32})
  if err!=nil { t.Fatal(err) }
  mutants:=map[string]any{}
  for _,name:=range []string{"absent","present","missing-outcome","missing-error","missing-write","input-precedence"} {
    checked,problem:=Explore(func() Target { return &probeTarget{calls:map[string]int{},broken:name} },ExploreOptions{Seeds:4,Steps:32})
    if problem!=nil { t.Fatal(problem) }; mutants[name]=checked
  }
  value:=map[string]any{"draws":draws,"mutants":mutants,"plan_drawn":drawn,"plan_excluded":p.excluded,"calls":calls,"result":result,"healthy_failure":result.Failure,"control_detected":control.Failure!=nil,"control":control}
  raw,err:=json.MarshalIndent(value,"","  ")
  if err!=nil { t.Fatal(err) }
  if err:=os.WriteFile("../../go-result.json",raw,0600);err!=nil { t.Fatal(err) }
  t.Log(string(raw))
  if result.Failure!=nil { t.Fatal("healthy target failed") }
  if control.Failure==nil { t.Fatal("plain input negative control was not detected") }
}
"#;
pub const TS: &str = r"import {writeFileSync} from 'node:fs';
import {explore} from './dist/explore.js';

const target=(calls,broken='',draws)=>{
  let rows=new Map(), next=0;
  return {
    beginScenario(){rows=new Map();next=0;},
    endScenario(){},
    queryView({view}){
      if(view!=='demo.items.Items') throw new Error(`unexpected view ${view}`);
      return {rows:[...rows].map(([id,state])=>({id,state}))};
    },
    executeCommand({command,input}){
      calls[command]=(calls[command]??0)+1;
      if(draws) draws.push({command,input});
      if(command==='demo.items.Create'){
        const id=`00000000-0000-4000-8000-${String(++next).padStart(12,'0')}`;
        rows.set(id,'Open');
        return {outcome:'created',directEvents:[{event:'demo.items.Created',payload:{id}}]};
      }
      if(command==='demo.items.PlainNote'||command==='demo.items.OptionalNote'){
        const flag=broken==='plain'&&command==='demo.items.PlainNote'?!input.flag:input.flag;
        let note=input.note;
        if(broken==='absent'&&command==='demo.items.OptionalNote'&&note==null) note='fabricated';
        if(broken==='present'&&command==='demo.items.OptionalNote'&&note!=null) note=null;
        return {outcome:'noted',directEvents:[{event:'demo.items.Noted',payload:{flag,note}}]};
      }
      if(command==='demo.items.Close'){
        if(input.deny===true&&broken!=='input-precedence') return {outcome:'denied',error:'demo.items.Conflict'};
        if(!rows.has(input.id)) {
          if(broken==='missing-outcome') return {outcome:'closed'};
          if(broken==='missing-error') return {outcome:'not-found',error:'demo.items.Conflict'};
          if(broken==='missing-write') rows.set(input.id,'Closed');
          return {outcome:'not-found',error:'demo.items.NotFound',directEvents:[]};
        }
        if(rows.get(input.id)!=='Open') return {outcome:'wrong-state',error:'demo.items.Conflict',directEvents:[]};
        rows.set(input.id,'Closed');
        return {outcome:'closed',directEvents:[{event:'demo.items.Closed',payload:{id:input.id}}]};
      }
      throw new Error(`unexpected command ${command}`);
    }
  };
};
const calls=Object.create(null);
const draws=[];
const result=await explore(()=>target(calls,'',draws),{seeds:4,steps:32});
const control=await explore(()=>target(Object.create(null),'plain'),{seeds:4,steps:32});
const mutants={};
for(const name of ['absent','present','missing-outcome','missing-error','missing-write','input-precedence']) mutants[name]=await explore(()=>target(Object.create(null),name),{seeds:4,steps:32});
const value={draws,mutants,calls,result,healthy_failure:result.failure??null,control_detected:control.failure!==undefined,control};
writeFileSync('../../ts-result.json',JSON.stringify(value,null,2));
console.log(JSON.stringify(value,null,2));
if(result.failure!==undefined) throw new Error('healthy target failed');
if(control.failure===undefined) throw new Error('plain input negative control was not detected');
";
