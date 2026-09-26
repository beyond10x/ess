import test from 'node:test';
import { run, JsonNumber } from './dist/runtime.js';
class Backend {
  row;
  identity() { return {name:'history',version:'1'}; }
  beginScenario() { this.row = undefined; }
  endScenario() {}
  configureExternalOutcome() { throw Error('history is not externally forced'); }
  executeCommand({command,input}) {
    const result = {consistency:'1'};
    const emit = (name,payload = {}) => result.directEvents = [{event:`calls.core.${name}`,payload}];
    const mutant = process.env.ESS_HISTORY_MUTANT;
    if (command === 'calls.core.Open') {
      this.row = {call_id:'00000000-0000-4000-8000-000000000001',state:'Init',answer_history:'Unanswered',note:input.note,server_stamp:new JsonNumber("9007199254740992")};
      result.outcome='opened';emit('Opened',{call_id:this.row.call_id});
    } else if (!this.row || input.call_id !== this.row.call_id) {
      // The unknown-instance rule: every command here declares `wrong_state` (`gone`).
      result.outcome='gone';result.error='calls.core.Gone';
    } else if (this.row.state === 'Ended') {
      result.outcome='gone';result.error='calls.core.Gone';
    } else if (command === 'calls.core.Answer') {
      if (this.row.answer_history === 'Answered') {
        result.outcome='already-answered';
        if (mutant === 'error') result.error='calls.core.Gone';
        if (mutant === 'rewrite') this.row.server_stamp=new JsonNumber("9007199254740993");
        if (mutant === 'event') emit('Answered');
        if (mutant === 'lost_history') this.row.answer_history='Unanswered';
      } else if (mutant === 'state_only' && this.row.state === 'Bridged') result.outcome='already-answered';
      else {this.row.state='Bridged';this.row.answer_history='Answered';result.outcome='answered';emit('Answered');}
    } else if (command === 'calls.core.Report') {this.row.state='Bridged';result.outcome='observed';emit('Observed');}
    else if (command === 'calls.core.Close') {this.row.state='Ended';result.outcome='closed';emit('Observed');}
    else throw Error('unknown command');
    return result;
  }
  queryView() {
    if (!this.row || process.env.ESS_HISTORY_MUTANT === 'missing') return {rows:[]};
    return {rows:process.env.ESS_HISTORY_MUTANT === 'duplicate' ? [this.row,this.row] : [this.row]};
  }
  observeEvents() {return [];}
  redeliverEvent() {throw Error('no bindings');}
  observeInvocations() {return [];}
}
await test('observed subject history', t => run(t, () => new Backend()));
