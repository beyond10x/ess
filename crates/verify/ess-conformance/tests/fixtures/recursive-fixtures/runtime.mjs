import test from 'node:test';
import { run } from './dist/runtime.js';
const mode=process.env.ESS_FIXTURE_CASE;
const finite='{"text":"root","children":[{"text":"a","children":[]},{"text":"b","children":[{"text":"c","children":[]}]}]}';
let supplied=finite;
const canonical=value=>Array.isArray(value)?`[${value.map(canonical).join(",")}]`:value!==null&&typeof value==='object'?`{${Object.keys(value).sort().map(key=>`${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`:JSON.stringify(value);
test('recursive fixture execution',async t=>run(t,()=>({
  identity:()=>({name:'recursive-fixtures',version:'1'}),
  fixtureValues:()=>{
    if(mode==='wrong-leaf')supplied=finite.replace('"text":"c"','"text":5');
    if(mode==='too-deep')supplied='{"text":"deep","children":['.repeat(70)+']}'.repeat(70);
    return {'finite-value':JSON.parse(supplied)};
  },
  beginScenario:()=>console.log('recursive session begun'),
  endScenario:()=>{},
  executeCommand:request=>{
    if(canonical(request.input.value)!==canonical(JSON.parse(supplied)))throw new Error('the command did not receive the supplied tree');
    console.log('recursive fixture received');
    return {outcome:'accepted',directEvents:[{event:'fixtureprobe.recursive.Accepted',payload:{accepted:true}}]};
  },
  queryView:()=>({rows:[]}),observeEvents:()=>[],observeInvocations:()=>[],configureExternalOutcome(){},redeliverEvent(){},
})));
