import test from 'node:test';
import assert from 'node:assert/strict';
import { run, unsupported } from './dist/runtime.js';
import { admitFixtures } from './dist/fixtures.js';
const APP = '6f1c3c55-2c4e-4c0e-9a51-1a8a3c1e6b01';
let app, links = 0, rows = [];
test('provided rows', async t => run(t, () => ({
  identity: () => ({ name: 'provided', version: '1' }),
  fixtureValues: (_context, contract) => {
    const values = {};
    for (const field of contract.fields) {
      if (field.entity !== 'linking.links.App' || field.state !== 'Active' || field.type !== 'linking.links.AppId')
        throw new Error(`unexpected fixture ${JSON.stringify(field)}`);
      console.log(`provided ${field.entity} ${field.state}`);
      app = APP;
      values[field.name] = APP;
    }
    return values;
  },
  beginScenario: () => {},
  endScenario: () => { app = undefined; rows = []; },
  executeCommand: request => {
    const sent = request.input.app_id;
    if (app === undefined || sent !== app)
      return { outcome: 'unknown-app', error: 'linking.links.AppNotFound', errorPayload: { app_id: sent } };
    links++;
    const link = `00000000-0000-4000-8000-${String(links).padStart(12, '0')}`;
    rows.push({ link_id: link, app_id: sent });
    return { outcome: 'linked', consistency: String(links), directEvents: [{ event: 'linking.links.Linked', payload: { link_id: link, app_id: sent } }] };
  },
  queryView: request => { if (request.view !== 'linking.links.Links') throw unsupported('no other view is read here'); return { rows }; }, observeEvents: () => [], observeInvocations: () => [], configureExternalOutcome() {}, redeliverEvent() {},
})));
test('entity fixture admission', () => {
  const contract = () => ({ fields: [{ name: 'app', type: 'Uuid', entity: 'linking.links.App', state: 'Active' }], declarations: {} });
  assert.equal(admitFixtures(contract(), 60).fields[0].entity, 'linking.links.App');
  assert.throws(() => admitFixtures(contract(), 58));
  const half = contract();
  delete half.fields[0].state;
  assert.throws(() => admitFixtures(half, 60), /entity and its state together/);
  console.log('entity fixture admission checked');
});
