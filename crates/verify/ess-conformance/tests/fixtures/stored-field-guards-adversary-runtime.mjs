import test from 'node:test';
import { run } from './dist/runtime.js';
const refuses = (row) => {
  const weight = Number(String(row.weight_kg));
  const express = row.service === 'Express';
  switch (process.env.ESS_PARCELS_MUTANT) {
    case 'weight': return express;
    case 'fields': return false;
    case 'service': return weight > 20;
    default: return express && weight > 20;
  }
};
class Parcels {
  rows = new Map();
  minted = 0;
  identity() { return { name: 'parcels', version: '1' }; }
  beginScenario() { this.rows = new Map(); }
  endScenario() {}
  configureExternalOutcome() { throw Error('not externally forced'); }
  executeCommand({ command, input }) {
    this.minted += 1;
    const result = { consistency: String(this.minted) };
    if (command === 'shipping.parcel.Create') {
      const id = `00000000-0000-4000-8000-${String(this.minted).padStart(12, '0')}`;
      this.rows.set(id, { ...input, parcel_id: id, state: 'Created' });
      result.outcome = 'created';
      result.directEvents = [{ event: 'shipping.parcel.Created', payload: { parcel_id: id } }];
    } else if (command === 'shipping.parcel.Dispatch') {
      const row = this.rows.get(input.parcel_id);
      if (!row || row.state !== 'Created') return result;
      if (refuses(row)) {
        result.outcome = 'refused-overweight';
        result.error = 'shipping.parcel.ExpressOverweight';
        return result;
      }
      row.state = 'Dispatched';
      result.outcome = 'dispatched';
      result.directEvents = [{ event: 'shipping.parcel.Dispatched', payload: {} }];
    } else throw Error(`unknown command ${command}`);
    return result;
  }
  queryView() { return { rows: [...this.rows.values()] }; }
  observeEvents() { return []; }
  redeliverEvent() { throw Error('no bindings'); }
  observeInvocations() { return []; }
}
await test('parcels', (t) => run(t, () => new Parcels()));
