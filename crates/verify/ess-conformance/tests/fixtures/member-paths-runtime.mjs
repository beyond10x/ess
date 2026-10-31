import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { run, unsupported } from './dist/runtime.js';
import { admitDirectResponse, compareDirectResponse, memberPathsMajor } from './dist/direct_response.js';
let calls = 0;
test('member paths', async t => run(t, () => ({
  identity: () => ({ name: 'paths', version: '1' }),
  fixtureValues: () => ({}),
  beginScenario: () => {},
  endScenario: () => {},
  executeCommand: request => {
    if (request.command !== 'relay.loop.Defer') throw unsupported('no other command is sent here');
    calls++;
    const format = `f-${calls}`;
    console.log(`deferred with format ${format}`);
    return { outcome: 'deferred', response: { stop: 'later', checkpoint: { format, checkpoint_id: 'cp-1', call: request.input.call } } };
  },
  queryView: () => { throw unsupported('no view is read here'); }, observeEvents: () => [], observeInvocations: () => [], configureExternalOutcome() {}, redeliverEvent() {},
})));
const contract = expected => {
  const value = JSON.parse(readFileSync(new URL('./contract.json', import.meta.url), 'utf8'));
  if (expected !== undefined) value.expected = expected;
  return value;
};
const returned = (format, id) => ({ stop: 'later', checkpoint: { format, checkpoint_id: id, call: 'lookup' } });
test('member path admission', () => {
  const admitted = admitDirectResponse(contract());
  compareDirectResponse(admitted, returned('other', 'cp-1'));
  assert.throws(() => compareDirectResponse(admitted, returned('f-1', 'cp-2')));
  memberPathsMajor(contract(), 62);
  assert.throws(() => memberPathsMajor(contract(), 61), /suite\/62 or \/63/);
  for (const path of ['checkpoint.checkpoint', 'stop.length', 'checkpoint.'])
    assert.throws(() => admitDirectResponse(contract({ [path]: 'cp-1' })), undefined, path);
  console.log('member path admission checked');
});
