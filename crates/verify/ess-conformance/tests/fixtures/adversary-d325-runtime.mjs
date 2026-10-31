import test from 'node:test';
import assert from 'node:assert/strict';
import { admitFixtures } from './dist/fixtures.js';

// An entity fixture whose entity and state are both empty text names no entity; the Rust reader
// refuses it, and so must this runtime.
test('empty entity fixture', () => {
  const contract = { fields: [{ name: 'app', type: 'Uuid', entity: '', state: '' }], declarations: {} };
  assert.throws(() => admitFixtures(contract, 60));
  console.log('empty entity fixture refused');
});
