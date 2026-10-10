// A response or struct declared `undeclared_fields: ignored` (suite/48 and /49, beyond10x/ess#500)
// admits keys it does not declare, and nowhere else: what `src/go/prerequisites.go` and
// `src/go/response.go` answer, and what the native observers in `direct_response.rs`,
// `response.rs` and `selection.rs` answer.
import assert from 'node:assert/strict';
import test from 'node:test';

import {
  admitDirectResponse,
  compareDirectResponse,
  undeclaredFieldsMajor,
} from './direct_response.js';
import { admitResponse, compareResponse, decodeResponseObservation } from './response.js';

// ---- documents --------------------------------------------------------------------------------

/** A direct return of `order_ref`, edited by `overrides`. */
function direct(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    command: 'catalog.orders.PlaceOrder',
    fields: [{ name: 'order_ref', type: 'String' }],
    declarations: {},
    expected: {},
    ...overrides,
  };
}

/** A direct return reaching an opened `Vendor` beside a closed `Shipping`. */
function nested(): Record<string, unknown> {
  return direct({
    fields: [
      { name: 'order_ref', type: 'String' },
      { name: 'vendor', type: 'catalog.orders.Vendor' },
      { name: 'shipping', type: 'catalog.orders.Shipping' },
    ],
    declarations: {
      'catalog.orders.Vendor': { kind: 'struct', fields: [{ name: 'vendor_id', type: 'String' }] },
      'catalog.orders.Shipping': { kind: 'struct', fields: [{ name: 'carrier', type: 'String' }] },
    },
    undeclared_fields_ignored: ['catalog.orders.Vendor'],
  });
}

/** A response-mapped event observation of `order_ref`, edited by `overrides`. */
function mapped(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    command: 'catalog.orders.PlaceOrder',
    outcome: { command: 'catalog.orders.PlaceOrder', outcome: 'placed' },
    event: 'catalog.orders.OrderPlaced',
    fields: [{ name: 'order_ref', type: 'String' }],
    declarations: {},
    mappings: { order_ref: 'order_ref' },
    targets: [{ name: 'order_ref', type: 'String' }],
    ...overrides,
  };
}

// ---- direct returns -----------------------------------------------------------------------------

test('an opened response admits an undeclared key and still checks its declared field', () => {
  const contract = admitDirectResponse(direct({ undeclared_fields: 'ignored' }));
  compareDirectResponse(contract, { order_ref: 'o-1', extension: 'x' });
  compareDirectResponse(contract, { order_ref: 'o-1' });
  assert.throws(() => compareDirectResponse(contract, { extension: 'x' }));
  assert.throws(() => compareDirectResponse(contract, { order_ref: 7, extension: 'x' }));
});

test('a closed response still refuses an undeclared key', () => {
  const contract = admitDirectResponse(direct());
  assert.throws(
    () => compareDirectResponse(contract, { order_ref: 'o-1', extension: 'x' }),
    /extra response field/,
  );
});

test('an opened struct admits extras at that struct only', () => {
  const contract = admitDirectResponse(nested());
  const shipping = { carrier: 'c-1' };
  const vendor = { vendor_id: 'v-1', region: 'eu' };
  compareDirectResponse(contract, { order_ref: 'o-1', vendor, shipping });
  assert.throws(
    () =>
      compareDirectResponse(contract, {
        order_ref: 'o-1',
        vendor,
        shipping: { carrier: 'c-1', tracking: 't' },
      }),
    /extra response member/,
  );
  assert.throws(() =>
    compareDirectResponse(contract, { order_ref: 'o-1', vendor: { region: 'eu' }, shipping }),
  );
  assert.throws(() =>
    compareDirectResponse(contract, { order_ref: 'o-1', vendor: { vendor_id: 7 }, shipping }),
  );
  // Opening the struct does not open the response object.
  assert.throws(
    () => compareDirectResponse(contract, { order_ref: 'o-1', vendor, shipping, extension: 'x' }),
    /extra response field/,
  );
});

test('a present member never spells closed or names a stranger', () => {
  assert.throws(() => admitDirectResponse(direct({ undeclared_fields: 'refused' })));
  assert.throws(() => admitDirectResponse(direct({ undeclared_fields_ignored: [] })));
  for (const names of [
    ['catalog.orders.Vendor', 'catalog.orders.Vendor'],
    ['catalog.orders.Absent'],
  ]) {
    assert.throws(() => admitDirectResponse({ ...nested(), undeclared_fields_ignored: names }));
  }
});

test('either member under a major below 48 is refused by name', () => {
  for (const document of [direct({ undeclared_fields: 'ignored' }), nested()]) {
    assert.throws(() => undeclaredFieldsMajor(document, 47), /suite\/48 or \/49/);
    undeclaredFieldsMajor(document, 48);
  }
  undeclaredFieldsMajor(direct(), 47);
});

// ---- response-mapped payloads ---------------------------------------------------------------------

test('an opened response payload observation admits an undeclared key', () => {
  const observation = decodeResponseObservation(mapped({ undeclared_fields: 'ignored' }));
  compareResponse(observation, { order_ref: 'o-1', extension: 'x' }, { order_ref: 'o-1' });
  assert.throws(() =>
    compareResponse(observation, { order_ref: 7, extension: 'x' }, { order_ref: 'o-1' }),
  );
  const closed = decodeResponseObservation(mapped());
  assert.throws(
    () => compareResponse(closed, { order_ref: 'o-1', extension: 'x' }, { order_ref: 'o-1' }),
    /undeclared field/,
  );
});

test('an opened response payload observation is admitted from suite/48 only', () => {
  const document = mapped({ undeclared_fields: 'ignored' });
  assert.throws(() => admitResponse(document, 47), /suite\/48 or \/49/);
  admitResponse(document, 48);
  assert.throws(() => decodeResponseObservation(mapped({ undeclared_fields: 'refused' })));
});
