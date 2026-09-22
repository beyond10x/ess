import test from 'node:test';
import { run, unsupported } from './dist/runtime.js';
const mode=process.env.ESS_FIXTURE_CASE;
let resolved=0,values;
test('fixture execution',async t=>run(t,()=>({
  identity:()=>({name:'fixtures',version:'1'}),
  fixtureValues:(_context,contract)=>{
    resolved++;
    contract.fields[0].type='Boolean';
    values={'current-principal-id':'d248c830-37a4-466b-8319-af71667f1364','current-principal-email':'principal-1@example.com'};
    if(mode==='invalid')values['current-principal-id']=true;
    if(mode==='missing')delete values['current-principal-email'];
    if(mode==='unknown')values.unknown=null;
    if(mode==='unsupported')throw unsupported('fixture unavailable');
    return values;
  },
  beginScenario:()=>{
    console.log('fixture session begun');
    if(resolved!==1)throw new Error('fixtures must resolve once before session startup');
    values['current-principal-email']='changed@example.com';
  },
  endScenario:()=>console.log('fixture session ended'),
  executeCommand:request=>{
    if(request.input.email!=='principal-1@example.com')throw new Error('expected independent fixture input');
    const payload=request.input,events=[];
    if(mode==='wrong-then-right')events.push({event:'fixturetest.session.Opened',payload:{...payload,email:'wrong@example.com'}});
    if(mode==='split-output')events.push({event:'fixturetest.session.Opened',payload:{...payload,region:'wrong-region'}});
    if(mode==='wrong-output'||mode==='split-output')payload.email='different@example.com';
    events.push({event:'fixturetest.session.Opened',payload});
    return {outcome:'opened',directEvents:events};
  },
  queryView:()=>({rows:[]}),observeEvents:()=>[],observeInvocations:()=>[],configureExternalOutcome(){},redeliverEvent(){},
})));
