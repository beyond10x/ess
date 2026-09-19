// The response decoder answers exactly what `src/go/response.go` answers.
//
// The exactness cases are the ones to read first. A returned Integer is never rounded through a
// binary64 here, so `9007199254740993` and `9007199254740992` are two values and `1.0` and `1` are
// one — which is what the Go runtime answers, checked against it, and what a suite run in two
// languages has to agree on before any verdict it reports means anything.
import assert from 'node:assert/strict';
import test from 'node:test';

import { JsonNumber, type CommandResult, type Node, type Step } from './runtime.js';
import {
  admitResponse,
  compareResponse,
  decodeResponseObservation,
  expectResponsePayload,
  responseAssignable,
  responseEqual,
  responseNumber,
  responsePrimitiveAdmits,
  snapshotResponseResult,
  type ResponseObservation,
  type ResponseRun,
} from './response.js';

// ---- documents --------------------------------------------------------------------------------

function wire(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    command: 'billing.ChargeCard',
    outcome: { command: 'billing.ChargeCard', outcome: 'charged' },
    event: 'billing.CardCharged',
    fields: [{ name: 'receipt', type: 'String' }],
    declarations: {},
    mappings: { receiptId: 'receipt' },
    targets: [{ name: 'receiptId', type: 'String' }],
    ...overrides,
  };
}

function observation(overrides: Record<string, unknown> = {}): ResponseObservation {
  return decodeResponseObservation(wire(overrides));
}

// ---- decoding ---------------------------------------------------------------------------------

test('decodeResponseObservation reads the wire document the suite carries', () => {
  const decoded = observation();
  assert.equal(decoded.command, 'billing.ChargeCard');
  assert.deepEqual(decoded.outcome, { command: 'billing.ChargeCard', outcome: 'charged' });
  assert.equal(decoded.event, 'billing.CardCharged');
  assert.deepEqual(decoded.fields, [{ name: 'receipt', type: 'String' }]);
  assert.deepEqual(decoded.mappings, { receiptId: 'receipt' });
  assert.deepEqual(decoded.targets, [{ name: 'receiptId', type: 'String' }]);
});

test('decodeResponseObservation refuses an unknown field at every level', () => {
  assert.throws(() => decodeResponseObservation({ ...wire(), extra: 1 }), /unknown field/);
  assert.throws(
    () => decodeResponseObservation(wire({ outcome: { command: 'a.B', outcome: 'c', extra: 1 } })),
    /unknown field/,
  );
  assert.throws(
    () =>
      decodeResponseObservation(wire({ fields: [{ name: 'receipt', type: 'String', wire: 'r' }] })),
    /unknown field/,
  );
  assert.throws(
    () =>
      decodeResponseObservation(
        wire({
          fields: [{ name: 'receipt', type: 'billing.Receipt' }],
          declarations: { 'billing.Receipt': { kind: 'newtype', of: 'String', extra: 1 } },
          targets: [{ name: 'receiptId', type: 'billing.Receipt' }],
        }),
      ),
    /unknown field/,
  );
});

test('decodeResponseObservation refuses a field of the wrong kind', () => {
  assert.throws(() => decodeResponseObservation(wire({ command: 7 })), /string/);
  assert.throws(() => decodeResponseObservation(wire({ fields: {} })), /array/);
  assert.throws(() => decodeResponseObservation(wire({ declarations: [] })), /object/);
  assert.throws(() => decodeResponseObservation(wire({ mappings: { receiptId: 7 } })), /string/);
  assert.throws(() => decodeResponseObservation('not a document'), /object/);
});

// ---- the bounds and the owner -------------------------------------------------------------------

test('the observation must name its own command and a branch of it', () => {
  const bounds: [string, Record<string, unknown>][] = [
    [
      'a command that is not the outcome’s',
      { command: 'billing.Other', outcome: { command: 'billing.ChargeCard', outcome: 'charged' } },
    ],
    ['an empty outcome', { outcome: { command: 'billing.ChargeCard', outcome: '' } }],
    ['no fields', { fields: [] }],
    ['no mappings', { mappings: {}, targets: [] }],
    ['more mappings than targets', { mappings: { receiptId: 'receipt', other: 'receipt' } }],
    [
      'more than 256 fields',
      {
        fields: Array.from({ length: 257 }, (_, index) => ({
          name: `field${index}`,
          type: 'String',
        })),
        mappings: { receiptId: 'field0' },
      },
    ],
    [
      'more than 256 targets',
      {
        targets: Array.from({ length: 257 }, (_, index) => ({
          name: `target${index}`,
          type: 'String',
        })),
        mappings: Object.fromEntries(
          Array.from({ length: 257 }, (_, index) => [`target${index}`, 'receipt']),
        ),
      },
    ],
  ];
  for (const [what, overrides] of bounds) {
    assert.throws(
      () => decodeResponseObservation(wire(overrides)),
      /invalid response contract bounds or owner/,
      what,
    );
  }
});

test('the command and the event must be names', () => {
  assert.throws(
    () =>
      decodeResponseObservation(
        wire({ command: '9bad', outcome: { command: '9bad', outcome: 'charged' } }),
      ),
    /invalid name/,
  );
  assert.throws(() => decodeResponseObservation(wire({ event: '' })), /invalid name/);
  assert.throws(() => decodeResponseObservation(wire({ event: 'not a name' })), /invalid name/);
});

test('a duplicate or empty field name is refused, in fields and in targets', () => {
  assert.throws(
    () =>
      decodeResponseObservation(
        wire({
          fields: [
            { name: 'receipt', type: 'String' },
            { name: 'receipt', type: 'String' },
          ],
        }),
      ),
    /duplicate\/empty response field/,
  );
  assert.throws(
    () => decodeResponseObservation(wire({ fields: [{ name: '', type: 'String' }] })),
    /duplicate\/empty response field/,
  );
  assert.throws(
    () =>
      decodeResponseObservation(
        wire({
          targets: [
            { name: 'receiptId', type: 'String' },
            { name: 'receiptId', type: 'String' },
          ],
          mappings: { receiptId: 'receipt', other: 'receipt' },
        }),
      ),
    /duplicate\/empty response field/,
  );
});

test('a declaration nothing reaches is refused', () => {
  assert.throws(
    () =>
      decodeResponseObservation(
        wire({ declarations: { 'billing.Receipt': { kind: 'newtype', of: 'String' } } }),
      ),
    /unrelated response declarations/,
  );
});

test('a mapping must name a declared response field the target can hold', () => {
  assert.throws(
    () => decodeResponseObservation(wire({ mappings: { receiptId: 'absent' } })),
    /invalid response mapping/,
  );
  assert.throws(
    () => decodeResponseObservation(wire({ mappings: { other: 'receipt' } })),
    /invalid response mapping/,
  );
  assert.throws(
    () => decodeResponseObservation(wire({ targets: [{ name: 'receiptId', type: 'Integer' }] })),
    /invalid response mapping/,
  );
  // An Optional target holds what its inner type holds.
  decodeResponseObservation(wire({ targets: [{ name: 'receiptId', type: 'Optional<String>' }] }));
  decodeResponseObservation(
    wire({ targets: [{ name: 'receiptId', type: 'Optional<Optional<String>>' }] }),
  );
});

test('responseAssignable widens into Optional and nowhere else', () => {
  assert.equal(responseAssignable('String', 'String'), true);
  assert.equal(responseAssignable('String', 'Optional<String>'), true);
  assert.equal(responseAssignable('String', 'Optional<Optional<String>>'), true);
  assert.equal(responseAssignable('Optional<String>', 'String'), false);
  assert.equal(responseAssignable('String', 'Integer'), false);
  assert.equal(responseAssignable('Optional<String>', 'Optional<String>'), true);
});

// ---- the type walk --------------------------------------------------------------------------------

function typed(type: string, declarations: Record<string, unknown> = {}): Record<string, unknown> {
  return wire({
    fields: [{ name: 'receipt', type }],
    declarations,
    targets: [{ name: 'receiptId', type }],
  });
}

test('a map key that is not String is refused', () => {
  assert.throws(
    () => decodeResponseObservation(typed('Map<Integer, String>')),
    /response map key must be String/,
  );
  decodeResponseObservation(typed('Map<String, String>'));
});

test('Binary64 is not a response type', () => {
  assert.throws(
    () => decodeResponseObservation(typed('Binary64')),
    /missing\/recursive response type/,
  );
  assert.throws(
    () => decodeResponseObservation(typed('List<Binary64>')),
    /missing\/recursive response type/,
  );
});

test('a declaration that does not exist, or reaches itself, is refused', () => {
  assert.throws(
    () => decodeResponseObservation(typed('billing.Absent')),
    /missing\/recursive response type/,
  );
  assert.throws(
    () =>
      decodeResponseObservation(
        typed('billing.Loop', { 'billing.Loop': { kind: 'newtype', of: 'billing.Loop' } }),
      ),
    /missing\/recursive response type/,
  );
  assert.throws(
    () =>
      decodeResponseObservation(
        typed('billing.A', {
          'billing.A': { kind: 'newtype', of: 'billing.B' },
          'billing.B': { kind: 'newtype', of: 'billing.A' },
        }),
      ),
    /missing\/recursive response type/,
  );
});

test('each declaration kind is walked, and a malformed one is refused', () => {
  decodeResponseObservation(
    typed('billing.Receipt', { 'billing.Receipt': { kind: 'newtype', of: 'String' } }),
  );
  decodeResponseObservation(
    typed('billing.Receipt', {
      'billing.Receipt': {
        kind: 'struct',
        fields: [
          { name: 'id', type: 'String' },
          { name: 'total', type: 'Decimal' },
        ],
      },
    }),
  );
  decodeResponseObservation(
    typed('billing.State', { 'billing.State': { kind: 'enum', variants: ['Open', 'Closed'] } }),
  );
  decodeResponseObservation(
    typed('billing.Method', {
      'billing.Method': {
        kind: 'union',
        tag: 'kind',
        variants: { card: 'String', cash: 'Decimal' },
      },
    }),
  );

  const refusals: [string, Record<string, unknown>, RegExp][] = [
    [
      'an unknown kind',
      { 'billing.X': { kind: 'alias', of: 'String' } },
      /unknown response declaration/,
    ],
    [
      'a struct with a duplicate member',
      {
        'billing.X': {
          kind: 'struct',
          fields: [
            { name: 'id', type: 'String' },
            { name: 'id', type: 'String' },
          ],
        },
      },
      /duplicate\/empty response member/,
    ],
    [
      'a struct with an empty member name',
      { 'billing.X': { kind: 'struct', fields: [{ name: '', type: 'String' }] } },
      /duplicate\/empty response member/,
    ],
    [
      'an enum with no variants',
      { 'billing.X': { kind: 'enum', variants: [] } },
      /invalid response enum/,
    ],
    [
      'an enum whose variants are not a list',
      { 'billing.X': { kind: 'enum', variants: 'Open' } },
      /invalid response enum/,
    ],
    [
      'an enum with a duplicate label',
      { 'billing.X': { kind: 'enum', variants: ['Open', 'Open'] } },
      /duplicate response enum label/,
    ],
    [
      'an enum with an empty label',
      { 'billing.X': { kind: 'enum', variants: [''] } },
      /duplicate response enum label/,
    ],
    [
      'a union with no tag',
      { 'billing.X': { kind: 'union', variants: { card: 'String' } } },
      /invalid response union/,
    ],
    [
      'a union with no variants',
      { 'billing.X': { kind: 'union', tag: 'kind', variants: {} } },
      /invalid response union/,
    ],
    [
      'a union whose variants are not a map of names',
      { 'billing.X': { kind: 'union', tag: 'kind', variants: ['String'] } },
      /invalid response union/,
    ],
  ];
  for (const [what, declarations, expected] of refusals) {
    assert.throws(
      () => decodeResponseObservation(typed('billing.X', declarations)),
      expected,
      what,
    );
  }
});

test('a type nested past the depth bound is refused', () => {
  const deep = `${'Optional<'.repeat(130)}String${'>'.repeat(130)}`;
  assert.throws(() => decodeResponseObservation(typed(deep)), /response type depth limit/);
});

test('a declaration reached twice is walked once', () => {
  // `used` is a set, so a type two fields name is one entry and the inventory still balances.
  decodeResponseObservation(
    wire({
      fields: [
        { name: 'receipt', type: 'billing.Receipt' },
        { name: 'copy', type: 'billing.Receipt' },
      ],
      declarations: { 'billing.Receipt': { kind: 'newtype', of: 'String' } },
      targets: [{ name: 'receiptId', type: 'billing.Receipt' }],
    }),
  );
});

// ---- comparing a returned response against an emitted event ---------------------------------------

test('a response the command did not return is a refusal', () => {
  assert.throws(() => compareResponse(observation(), null, {}), /command returned no response/);
});

test('a response field the contract does not declare is a refusal', () => {
  assert.throws(
    () => compareResponse(observation(), { receipt: 'r-1', extra: 1 }, { receiptId: 'r-1' }),
    /response has an undeclared field/,
  );
});

test('a response field that is not of its declared type is a refusal naming the field', () => {
  assert.throws(
    () => compareResponse(observation(), { receipt: 7 }, { receiptId: 7 }),
    /^Error: response field receipt: invalid_input$/,
  );
});

test('an event field that differs from the response field it maps is a refusal', () => {
  assert.throws(
    () => compareResponse(observation(), { receipt: 'r-1' }, { receiptId: 'r-2' }),
    /^Error: event field receiptId differs from actual response field receipt$/,
  );
  assert.throws(
    () => compareResponse(observation(), { receipt: 'r-1' }, {}),
    /event field receiptId differs from actual response field receipt/,
  );
});

test('a match is silent', () => {
  compareResponse(observation(), { receipt: 'r-1' }, { receiptId: 'r-1' });
});

test('an optional field absent on both sides is a match, and absent on one side is not', () => {
  const optional = observation({
    fields: [{ name: 'receipt', type: 'Optional<String>' }],
    targets: [{ name: 'receiptId', type: 'Optional<String>' }],
  });
  compareResponse(optional, {}, {});
  compareResponse(optional, { receipt: null }, { receiptId: null });
  compareResponse(optional, {}, { receiptId: null });
  compareResponse(optional, { receipt: 'r-1' }, { receiptId: 'r-1' });
  assert.throws(() => compareResponse(optional, { receipt: 'r-1' }, {}), /differs from/);
  assert.throws(() => compareResponse(optional, {}, { receiptId: 'r-1' }), /differs from/);
});

test('a returned number is compared exactly, never through a binary64', () => {
  const numeric = observation({
    fields: [{ name: 'amount', type: 'Integer' }],
    mappings: { amountCents: 'amount' },
    targets: [{ name: 'amountCents', type: 'Integer' }],
  });
  compareResponse(
    numeric,
    { amount: new JsonNumber('9007199254740993') },
    { amountCents: new JsonNumber('9007199254740993') },
  );
  compareResponse(
    numeric,
    { amount: new JsonNumber('1e3') },
    { amountCents: new JsonNumber('1000') },
  );
  assert.throws(
    () =>
      compareResponse(
        numeric,
        { amount: new JsonNumber('9007199254740993') },
        { amountCents: new JsonNumber('9007199254740992') },
      ),
    /differs from/,
  );
});

// ---- the step handler ------------------------------------------------------------------------------

function runFor(
  last: Partial<CommandResult>,
  lastCommand: string,
): { run: ResponseRun; failures: string[] } {
  const failures: string[] = [];
  const run = {
    last: {
      response: last.response,
      outcome: last.outcome ?? '',
      error: last.error ?? '',
      consistency: last.consistency ?? '',
      directEvents: last.directEvents ?? [],
    },
    lastCommand,
    fail(index: number, message: string): boolean {
      failures.push(`step ${index}: ${message}`);
      return false;
    },
  } satisfies ResponseRun;
  return { run, failures };
}

function responseStep(contract: ResponseObservation | null): Step {
  const step: Record<string, unknown> = { step: 'expect_response_payload', order: '' };
  if (contract !== null) {
    step.response = contract;
  }
  return step as unknown as Step;
}

test('a step with no response observation is a failure', () => {
  const { run, failures } = runFor({}, 'billing.ChargeCard');
  assert.equal(expectResponsePayload(run, 1, responseStep(null)), false);
  assert.deepEqual(failures, ['step 1: missing response observation']);
});

test('an observation about a different command or outcome is a failure', () => {
  const contract = observation();
  const other = runFor({ outcome: 'charged' }, 'billing.Other');
  assert.equal(expectResponsePayload(other.run, 2, responseStep(contract)), false);
  assert.deepEqual(other.failures, [
    'step 2: response observation names a different command/outcome',
  ]);

  const declined = runFor({ outcome: 'declined' }, 'billing.ChargeCard');
  assert.equal(expectResponsePayload(declined.run, 3, responseStep(contract)), false);
  assert.deepEqual(declined.failures, [
    'step 3: response observation names a different command/outcome',
  ]);
});

test('the invocation must have emitted the response-mapped event', () => {
  const contract = observation();
  const { run, failures } = runFor(
    {
      outcome: 'charged',
      response: { receipt: 'r-1' },
      directEvents: [{ event: 'billing.Other', payload: { receiptId: 'r-1' } }],
    },
    'billing.ChargeCard',
  );
  assert.equal(expectResponsePayload(run, 4, responseStep(contract)), false);
  assert.deepEqual(failures, ['step 4: same invocation did not emit the response-mapped event']);
});

test('the response and the event it maps agree, or the step fails with the reason', () => {
  const contract = observation();
  const passing = runFor(
    {
      outcome: 'charged',
      response: { receipt: 'r-1' },
      directEvents: [{ event: 'billing.CardCharged', payload: { receiptId: 'r-1' } }],
    },
    'billing.ChargeCard',
  );
  assert.equal(expectResponsePayload(passing.run, 5, responseStep(contract)), true);
  assert.deepEqual(passing.failures, []);

  const failing = runFor(
    {
      outcome: 'charged',
      response: { receipt: 'r-1' },
      directEvents: [{ event: 'billing.CardCharged', payload: { receiptId: 'r-2' } }],
    },
    'billing.ChargeCard',
  );
  assert.equal(expectResponsePayload(failing.run, 6, responseStep(contract)), false);
  assert.deepEqual(failing.failures, [
    'step 6: event field receiptId differs from actual response field receipt',
  ]);
});

test('the first event of the named type decides, and a later one is not consulted', () => {
  const contract = observation();
  const { run } = runFor(
    {
      outcome: 'charged',
      response: { receipt: 'r-1' },
      directEvents: [
        { event: 'billing.CardCharged', payload: { receiptId: 'r-1' } },
        { event: 'billing.CardCharged', payload: { receiptId: 'r-9' } },
      ],
    },
    'billing.ChargeCard',
  );
  assert.equal(expectResponsePayload(run, 7, responseStep(contract)), true);
});

// ---- admission ---------------------------------------------------------------------------------

test('admitResponse admits the document the decoder admits', () => {
  admitResponse(wire());
  admitResponse(
    wire({
      fields: [{ name: 'receipt', type: 'billing.Receipt' }],
      declarations: {
        'billing.Receipt': { kind: 'struct', fields: [{ name: 'id', type: 'String' }] },
      },
      targets: [{ name: 'receiptId', type: 'billing.Receipt' }],
    }),
  );
});

test('admitResponse refuses what the Go admitter refuses', () => {
  assert.throws(() => admitResponse({ ...wire(), extra: 1 }), /unknown field extra/);
  assert.throws(
    () => admitResponse(wire({ outcome: { command: 'a.B', outcome: 'Charged' } })),
    /invalid name/,
  );
  assert.throws(
    () => admitResponse(wire({ declarations: [] })),
    /response declarations must be object/,
  );
  assert.throws(
    () => admitResponse(wire({ declarations: { 'billing.X': 'newtype' } })),
    /invalid response declaration/,
  );
  assert.throws(
    () => admitResponse(wire({ declarations: { 'billing.X': { kind: 'alias' } } })),
    /invalid response declaration kind/,
  );
  assert.throws(
    () => admitResponse(wire({ declarations: { 'billing.X': { kind: 'newtype' } } })),
    /missing field of/,
  );
  assert.throws(
    () =>
      admitResponse(
        wire({
          declarations: {
            'billing.X': { kind: 'struct', fields: [{ name: '9bad', type: 'String' }] },
          },
        }),
      ),
    /invalid accessor field name/,
  );
  assert.throws(() => admitResponse(wire({ fields: {} })), /expected array/);
  assert.throws(
    () => admitResponse(wire({ fields: [{ name: '9bad', type: 'String' }] })),
    /invalid accessor field name/,
  );
  // The admitter permits a wire/display/summary alias on an accessor field; the decoder it ends
  // with does not, and the refusal is the decoder's.
  assert.throws(
    () => admitResponse(wire({ fields: [{ name: 'receipt', type: 'String', wire: 'r' }] })),
    /unknown field wire/,
  );
});

// ---- snapshotting a returned result -----------------------------------------------------------------

test('a snapshot is a copy the target can no longer reach', () => {
  const payload: Record<string, Node> = { receiptId: 'r-1' };
  const response: Record<string, Node> = { receipt: 'r-1', nested: { deep: [1, 2] } };
  const result: CommandResult = {
    response,
    outcome: 'charged',
    error: '',
    consistency: 'token-1',
    directEvents: [{ event: 'billing.CardCharged', payload }],
  };
  const snapshot = snapshotResponseResult(result);
  response.receipt = 'mutated';
  payload.receiptId = 'mutated';
  (response.nested as { deep: number[] }).deep.push(3);

  assert.equal(snapshot.response?.receipt?.toString(), 'r-1');
  assert.equal(snapshot.directEvents?.[0]?.payload.receiptId, 'r-1');
  assert.deepEqual((snapshot.response?.nested as { deep: unknown[] }).deep.map(String), ['1', '2']);
  assert.equal(snapshot.outcome, 'charged');
  assert.equal(snapshot.consistency, 'token-1');
  assert.equal(snapshot.directEvents?.[0]?.event, 'billing.CardCharged');
});

test('a snapshot keeps every number as the token it was written with', () => {
  const snapshot = snapshotResponseResult({
    response: { exact: new JsonNumber('9007199254740993'), plain: 1.5 },
    outcome: 'charged',
    error: '',
    consistency: '',
    directEvents: [],
  });
  assert.equal(snapshot.response?.exact?.toString(), '9007199254740993');
  assert.ok(snapshot.response?.exact instanceof JsonNumber);
  assert.ok(snapshot.response?.plain instanceof JsonNumber);
  assert.equal(snapshot.response?.plain?.toString(), '1.5');
});

test('a result that is not a typed wire value is refused', () => {
  for (const value of [Number.NaN, Number.POSITIVE_INFINITY, () => 1, Symbol('x')]) {
    assert.throws(
      () =>
        snapshotResponseResult({
          response: { bad: value as Node },
          outcome: 'charged',
          error: '',
          consistency: '',
          directEvents: [],
        }),
      /response result is not a typed wire value/,
      `refused ${String(value)}`,
    );
  }
});

test('a result past the byte limit is refused', () => {
  assert.throws(
    () =>
      snapshotResponseResult({
        response: { big: 'x'.repeat(1048577) },
        outcome: 'charged',
        error: '',
        consistency: '',
        directEvents: [],
      }),
    /response result byte limit/,
  );
});

// ---- exact numbers -------------------------------------------------------------------------------

test('responseNumber reads only a number token, and only a bounded one', () => {
  assert.equal(responseNumber('1'), null);
  assert.equal(responseNumber(1), null);
  assert.equal(responseNumber(null), null);
  assert.notEqual(responseNumber(new JsonNumber('1')), null);
  // The length bound is on the token, and the binary64 bound is on the value: 512 ones is 512
  // characters and has no finite image, so both refuse it and only one of them is the reason.
  assert.equal(responseNumber(new JsonNumber('1'.repeat(512))), null);
  assert.notEqual(responseNumber(new JsonNumber(`0.${'1'.repeat(510)}`)), null);
  assert.equal(responseNumber(new JsonNumber(`0.${'1'.repeat(511)}`)), null);
  assert.equal(responseNumber(new JsonNumber('1e513')), null);
  assert.equal(responseNumber(new JsonNumber('1e-513')), null);
  assert.equal(responseNumber(new JsonNumber('1e309')), null, 'a token with no binary64 image');
  assert.notEqual(responseNumber(new JsonNumber('1e-400')), null, 'underflow is not overflow');
});

test('responsePrimitiveAdmits answers what the Go admitter answers', () => {
  // Read off `src/go/response.go` compiled against these same tokens.
  const integers: [string, boolean][] = [
    ['0', true],
    ['-0', true],
    ['1', true],
    ['1.0', true],
    ['1e3', true],
    ['1E3', true],
    ['1.5e1', true],
    ['100e-2', true],
    ['9007199254740993', true],
    ['9223372036854775807', true],
    ['9223372036854775808', false],
    ['-9223372036854775808', true],
    ['-9223372036854775809', false],
    ['12345678901234567890', false],
    ['1.5', false],
    ['-1.5', false],
    ['0.1', false],
    ['1e-400', false],
    ['1e309', false],
    ['1e512', false],
    ['1e513', false],
  ];
  for (const [token, expected] of integers) {
    assert.equal(
      responsePrimitiveAdmits('Integer', new JsonNumber(token)),
      expected,
      `Integer ${token}`,
    );
  }
  const decimals: [string, boolean][] = [
    ['1.5', true],
    ['0.1', true],
    ['1e-400', true],
    ['1e309', false],
    ['1e512', false],
    ['1e513', false],
  ];
  for (const [token, expected] of decimals) {
    assert.equal(
      responsePrimitiveAdmits('Decimal', new JsonNumber(token)),
      expected,
      `Decimal ${token}`,
    );
  }
  assert.equal(responsePrimitiveAdmits('Integer', 1), false, 'a binary64 is not a token');
  assert.equal(responsePrimitiveAdmits('Decimal', 1.5), false, 'a binary64 is not a token');

  assert.equal(responsePrimitiveAdmits('String', 'x'), true);
  assert.equal(responsePrimitiveAdmits('Timestamp', 'x'), true);
  assert.equal(responsePrimitiveAdmits('Duration', 'x'), true);
  assert.equal(responsePrimitiveAdmits('String', 1), false);
  assert.equal(responsePrimitiveAdmits('Boolean', true), true);
  assert.equal(responsePrimitiveAdmits('Boolean', 'true'), false);
  assert.equal(responsePrimitiveAdmits('Uuid', '00000000-0000-0000-0000-000000000000'), true);
  assert.equal(responsePrimitiveAdmits('Uuid', 'not-a-uuid'), false);
  assert.equal(responsePrimitiveAdmits('Bytes', 'AAAA'), true);
  assert.equal(responsePrimitiveAdmits('Bytes', 'AA=A'), false);
  assert.equal(responsePrimitiveAdmits('Binary64', new JsonNumber('1')), false);
  assert.equal(responsePrimitiveAdmits('billing.Receipt', 'x'), false);
});

test('responseEqual compares tokens exactly and everything else structurally', () => {
  const token = (raw: string): Node => new JsonNumber(raw);
  assert.equal(responseEqual(token('1.0'), token('1')), true);
  assert.equal(responseEqual(token('1e3'), token('1000')), true);
  assert.equal(responseEqual(token('0.1'), token('0.10')), true);
  assert.equal(responseEqual(token('-0'), token('0')), true);
  assert.equal(responseEqual(token('9007199254740993'), token('9007199254740992')), false);
  assert.equal(responseEqual(token('1e-400'), token('0')), false, 'underflow is not zero');
  assert.equal(responseEqual(token('0.30000000000000004'), token('0.3')), false);
  assert.equal(responseEqual(token('1'), 1), false, 'a token compares only against a token');
  assert.equal(responseEqual(token('1e309'), token('1e309')), false, 'neither side has a value');

  assert.equal(responseEqual([token('1.0'), 'a'], [token('1'), 'a']), true);
  assert.equal(responseEqual([token('1')], [token('1'), token('2')]), false);
  assert.equal(responseEqual({ a: token('1.0') }, { a: token('1') }), true);
  assert.equal(responseEqual({ a: token('1') }, { a: token('1'), b: token('2') }), false);
  assert.equal(responseEqual({ a: token('1') }, { b: token('1') }), false);
  assert.equal(responseEqual('x', 'x'), true);
  assert.equal(responseEqual(true, true), true);
  assert.equal(responseEqual(null, null), true);
  assert.equal(responseEqual(1, 1), true);
  assert.equal(responseEqual(1, token('1')), true, 'a binary64 on the left falls back to equal');
});
