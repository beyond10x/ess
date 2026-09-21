import test from 'node:test';
import { run } from './dist/runtime.js';
class Backend {
  forced = false;
  identity() { return {name:'guarded',version:'1'}; }
  beginScenario() { this.forced = false; }
  endScenario() {}
  configureExternalOutcome() { this.forced = true; }
  executeCommand({input}) {
    if (this.forced && input.retry === false && input.recipient !== '' && process.env.ESS_IGNORE_FAULT !== '1') {
      return {outcome:'rejected',error:'delivery.mail.Rejected'};
    }
    return {outcome:'sent',directEvents:[{event:'delivery.mail.Sent',payload:{}}]};
  }
  queryView() { throw Error('no views'); }
  observeEvents() { return []; }
  redeliverEvent() { throw Error('no bindings'); }
  observeInvocations() { throw Error('no bindings'); }
}
await test('guarded external', t => run(t, () => new Backend()));
