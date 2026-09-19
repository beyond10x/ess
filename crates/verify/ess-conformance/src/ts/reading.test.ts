// The reading accessor answers exactly what `src/go/reading.go` answers.
//
// Three things are checked here, and they are the three the Go file is careful about: the integer
// canonicalization that refuses a rounded epoch second, the admission of a declared contract whose
// encoding and authority must agree, and the step handler's verdict — pass, fail or *skip*, which
// is a different fact from a failure and the one a target that cannot answer is owed.
import assert from 'node:assert/strict';
import test from 'node:test';

import { ErrUnsupported, JsonNumber, type ObservedEvent, type Step } from './runtime.js';
import {
  admitReading,
  clockIntegerText,
  expectReadingOrder,
  occurrenceKey,
  type ClockReadingTarget,
  type ReadingObservationRequest,
  type ReadingReference,
  type ReadingRun,
} from './reading.js';
import type { ClockReadingEvidence } from './coordinate.js';

// ---- clockIntegerText -------------------------------------------------------------------------

test('clockIntegerText canonicalizes an exact integer token and refuses everything else', () => {
  const admitted: [string, string][] = [
    ['0', '0'],
    ['1', '1'],
    ['1709296245', '1709296245'],
    ['253402300799', '253402300799'],
    ['1.0', '1'],
    ['1.000', '1'],
    ['1e3', '1000'],
    ['1E3', '1000'],
    ['1.5e1', '15'],
    ['1.50e1', '15'],
    ['100e-2', '1'],
    ['0.0', '0'],
    ['-0', '0'],
    ['-0.0', '0'],
    ['-0.000', '0'],
    ['1e+3', '1000'],
  ];
  for (const [raw, expected] of admitted) {
    assert.equal(clockIntegerText(raw), expected, `admitted ${raw}`);
  }

  const refused = [
    '-1', // a negative second is not a coordinate this reader admits
    '-1.0',
    '1.5', // not an exact integer
    '1e-1',
    '0.1',
    '1234567890123', // thirteen digits, past the twelve the reader admits
    '1e12', // 1 followed by twelve zeros is thirteen characters
    '1e513', // past the exponent bound
    '1e-513',
    '', // not a number at all
    'abc',
    '+1',
    '01',
    '000', // the grammar admits one leading zero and no more
    '1.',
    '.1',
    'Infinity',
    'NaN',
    `1${'0'.repeat(600)}`, // past the 512-character bound
  ];
  for (const raw of refused) {
    assert.equal(clockIntegerText(raw), null, `refused ${raw}`);
  }
});

test('clockIntegerText admits the largest coordinate the contract allows', () => {
  assert.equal(clockIntegerText('999999999999'), '999999999999');
  assert.equal(clockIntegerText('1000000000000'), null);
});

// ---- occurrenceKey ----------------------------------------------------------------------------

function reference(overrides: Partial<ReadingReference> = {}): ReadingReference {
  return {
    event: 'chronology.reading.Sampled',
    occurrence: 0,
    field: 'at',
    declared_type: 'Timestamp',
    contract: {
      encoding: 'offset_date_time_text',
      origins: [{ role: 'producer_process', offset: 'encoded_offset' }],
    },
    ...overrides,
  };
}

test('occurrenceKey names the event, the occurrence and the member', () => {
  assert.equal(occurrenceKey(reference()), 'chronology.reading.Sampled#0.at');
  assert.equal(
    occurrenceKey(reference({ occurrence: 7, field: 'observed_at' })),
    'chronology.reading.Sampled#7.observed_at',
  );
});

// ---- admitReading -----------------------------------------------------------------------------

function wire(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    event: 'chronology.reading.Sampled',
    occurrence: new JsonNumber('0'),
    field: 'at',
    declared_type: 'Timestamp',
    contract: {
      encoding: 'offset_date_time_text',
      origins: [{ role: 'producer_process', offset: 'encoded_offset' }],
    },
    ...overrides,
  };
}

test('admitReading admits the three encodings with their own authority', () => {
  admitReading(wire());
  admitReading(
    wire({
      contract: {
        encoding: 'unix_seconds',
        origins: [{ role: 'consumer_process', offset: 'encoding_defined_epoch' }],
      },
    }),
  );
  admitReading(
    wire({
      contract: {
        encoding: 'local_date_time_millis_literal_z',
        origins: [
          { role: 'producer_process', offset: 'requires_observation' },
          { role: 'unknown', offset: 'unknown' },
        ],
      },
    }),
  );
});

test('admitReading refuses what the Go admitter refuses', () => {
  const refusals: [string, Record<string, unknown>, RegExp][] = [
    ['an unknown field', { ...wire(), extra: 1 }, /unknown field extra/],
    [
      'a missing field',
      (() => {
        const value = wire();
        delete value.field;
        return value;
      })(),
      /missing field field/,
    ],
    ['an event that is not a name', wire({ event: '9bad' }), /invalid name/],
    ['a declared type that is not a name', wire({ declared_type: '-bad' }), /invalid name/],
    ['an occurrence that is not a token', wire({ occurrence: 0 }), /invalid reading occurrence/],
    [
      'an occurrence past 65535',
      wire({ occurrence: new JsonNumber('65536') }),
      /invalid reading occurrence/,
    ],
    ['a member that is not text', wire({ field: 7 }), /invalid reading member/],
    ['a member that starts with a digit', wire({ field: '1at' }), /invalid reading member/],
    ['a member with a hyphen', wire({ field: 'at-then' }), /invalid reading member/],
    [
      'a member over 128 characters',
      wire({ field: `a${'b'.repeat(128)}` }),
      /invalid reading member/,
    ],
    [
      'an occurrence key over 512 characters',
      wire({ event: `a.${'b'.repeat(508)}` }),
      /reading occurrence exceeds bound/,
    ],
    [
      'a contract with an unknown field',
      wire({ contract: { encoding: 'unix_seconds', origins: [], extra: 1 } }),
      /unknown field extra/,
    ],
    [
      'no origins',
      wire({ contract: { encoding: 'offset_date_time_text', origins: [] } }),
      /invalid reading origins/,
    ],
    [
      // Four *distinct* origins: the length bound is the only thing that can refuse them, so the
      // duplicate check below cannot stand in for this case.
      'four distinct origins',
      wire({
        contract: {
          encoding: 'local_date_time_millis_literal_z',
          origins: [
            { role: 'producer_process', offset: 'requires_observation' },
            { role: 'consumer_process', offset: 'requires_observation' },
            { role: 'unknown', offset: 'requires_observation' },
            { role: 'producer_process', offset: 'unknown' },
          ],
        },
      }),
      /invalid reading origins/,
    ],
    [
      'origins that are not an array',
      wire({ contract: { encoding: 'offset_date_time_text', origins: {} } }),
      /invalid reading origins/,
    ],
    [
      'a duplicate origin',
      wire({
        contract: {
          encoding: 'offset_date_time_text',
          origins: [
            { role: 'producer_process', offset: 'encoded_offset' },
            { role: 'producer_process', offset: 'encoded_offset' },
          ],
        },
      }),
      /duplicate reading origin/,
    ],
    [
      'an origin role that is not one of the three',
      wire({
        contract: {
          encoding: 'offset_date_time_text',
          origins: [{ role: 'other_process', offset: 'encoded_offset' }],
        },
      }),
      /invalid reading origin/,
    ],
    [
      'an authority that contradicts the encoding',
      wire({
        contract: {
          encoding: 'unix_seconds',
          origins: [{ role: 'producer_process', offset: 'encoded_offset' }],
        },
      }),
      /contradictory reading encoding and authority/,
    ],
    [
      'an unknown authority for an encoding that does not admit one',
      wire({
        contract: {
          encoding: 'offset_date_time_text',
          origins: [{ role: 'producer_process', offset: 'unknown' }],
        },
      }),
      /contradictory reading encoding and authority/,
    ],
    [
      'an encoding that is not text',
      wire({ contract: { encoding: 7, origins: [{ role: 'unknown', offset: 'unknown' }] } }),
      /expected string/,
    ],
  ];
  for (const [what, value, expected] of refusals) {
    assert.throws(() => admitReading(value), expected, what);
  }
});

test('admitReading reads the occurrence bound off the rendered key, not the event name', () => {
  // 512 exactly is admitted; one more is not. `a.` + 505 characters + `#0.at` is 512.
  admitReading(wire({ event: `a.${'b'.repeat(505)}` }));
  assert.throws(() => admitReading(wire({ event: `a.${'b'.repeat(506)}` })), /exceeds bound/);
});

// ---- the wire document is the reference -------------------------------------------------------

test('the suite document casts to a reference with no field renamed away', () => {
  // `decodeStep` casts the parsed JSON, which is the plain `json.Unmarshal` the Go runtime does at
  // the same point. Every member this file reads therefore has to be spelled as the suite spells
  // it, and a camel-cased one would be `undefined` in every request a target is handed.
  const parsed = JSON.parse(
    JSON.stringify({ ...wire(), occurrence: 3 }),
  ) as unknown as ReadingReference;
  assert.equal(occurrenceKey(parsed), 'chronology.reading.Sampled#3.at');
  assert.equal(parsed.declared_type, 'Timestamp');
  assert.equal(parsed.contract.origins[0]?.role, 'producer_process');
});

// ---- expectReadingOrder -------------------------------------------------------------------------

interface Recorded {
  failures: string[];
  skips: string[];
}

function runFor(target: unknown, seen: ObservedEvent[]): { run: ReadingRun; recorded: Recorded } {
  const recorded: Recorded = { failures: [], skips: [] };
  const run = {
    target: target as ReadingRun['target'],
    correlation: 'corr-1',
    seen,
    fail(index: number, message: string): boolean {
      recorded.failures.push(`step ${index}: ${message}`);
      return false;
    },
    skip(message: string): void {
      recorded.skips.push(message);
    },
  } satisfies ReadingRun;
  return { run, recorded };
}

function sampled(value: unknown, event = 'chronology.reading.Sampled'): ObservedEvent {
  return { event, payload: { at: value } };
}

function evidenceFor(
  occurrence: string,
  overrides: Partial<ClockReadingEvidence> = {},
): ClockReadingEvidence {
  return {
    correlation: 'corr-1',
    occurrence,
    processInstance: 'proc-1',
    epoch: 'epoch-1',
    origin: 'producer_process',
    formatter: 'encoded_offset',
    offsetMinutes: null,
    ...overrides,
  };
}

type Answer = (request: ReadingObservationRequest, index: number) => ClockReadingEvidence;

class RecordingTarget implements ClockReadingTarget {
  readonly requests: ReadingObservationRequest[] = [];
  readonly answer: Answer;

  constructor(answer: Answer) {
    this.answer = answer;
  }

  observeClockReading(request: ReadingObservationRequest): ClockReadingEvidence {
    const index = this.requests.length;
    this.requests.push(request);
    return this.answer(request, index);
  }
}

function orderStep(
  order: string,
  left: ReadingReference | null,
  right: ReadingReference | null,
): Step {
  const step: Record<string, unknown> = { step: 'expect_reading_order', order };
  if (left !== null) {
    step.left = left;
  }
  if (right !== null) {
    step.right = right;
  }
  return step as unknown as Step;
}

test('a target without the reading capability is skipped, not failed', () => {
  const { run, recorded } = runFor({}, []);
  assert.equal(expectReadingOrder(run, 3, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.skips, ['clock reading evidence unsupported']);
  assert.deepEqual(recorded.failures, []);
});

test('two readings of one epoch are ordered', () => {
  const target = new RecordingTarget((request) =>
    evidenceFor(
      request.reading.event + '#' + request.reading.occurrence + '.' + request.reading.field,
    ),
  );
  const { run, recorded } = runFor(target, [
    sampled('2024-03-01T12:30:45Z'),
    sampled('2024-03-01T12:30:46Z'),
  ]);
  assert.equal(
    expectReadingOrder(
      run,
      1,
      orderStep('before', reference({ occurrence: 0 }), reference({ occurrence: 1 })),
    ),
    true,
  );
  assert.equal(
    expectReadingOrder(
      run,
      1,
      orderStep('after', reference({ occurrence: 1 }), reference({ occurrence: 0 })),
    ),
    true,
  );
  assert.equal(
    expectReadingOrder(
      run,
      1,
      orderStep('equal', reference({ occurrence: 0 }), reference({ occurrence: 0 })),
    ),
    true,
  );
  assert.deepEqual(recorded.failures, []);
  assert.deepEqual(recorded.skips, []);
});

test('an order that does not hold is a failure naming the claimed order', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run, recorded } = runFor(target, [
    sampled('2024-03-01T12:30:45Z'),
    sampled('2024-03-01T12:30:46Z'),
  ]);
  assert.equal(
    expectReadingOrder(
      run,
      2,
      orderStep('after', reference({ occurrence: 0 }), reference({ occurrence: 1 })),
    ),
    false,
  );
  assert.deepEqual(recorded.failures, ['step 2: clock coordinate order differs from after']);
});

test('two different observed epochs are skipped rather than ordered', () => {
  const target = new RecordingTarget((request, index) =>
    evidenceFor(occurrenceKey(request.reading), { epoch: `epoch-${index}` }),
  );
  const { run, recorded } = runFor(target, [
    sampled('2024-03-01T12:30:45Z'),
    sampled('2024-03-01T12:30:46Z'),
  ]);
  assert.equal(
    expectReadingOrder(
      run,
      4,
      orderStep('before', reference({ occurrence: 0 }), reference({ occurrence: 1 })),
    ),
    false,
  );
  assert.deepEqual(recorded.failures, []);
  assert.equal(recorded.skips.length, 1);
  assert.match(recorded.skips[0]!, /^clock comparison unsupported: clock reading: DifferentClock$/);
});

test('evidence the adapter could not observe is skipped rather than failed', () => {
  const target = new RecordingTarget((request) =>
    evidenceFor(occurrenceKey(request.reading), { processInstance: '' }),
  );
  const { run, recorded } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(expectReadingOrder(run, 5, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.failures, []);
  assert.equal(recorded.skips.length, 1);
  assert.match(recorded.skips[0]!, /clock comparison unsupported: .*UnknownEvidence/);
});

test('a reading the scenario never observed is a failure', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run, recorded } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(
    expectReadingOrder(run, 6, orderStep('before', reference({ occurrence: 1 }), reference())),
    false,
  );
  assert.deepEqual(recorded.failures, [
    'step 6: clock reading: reading event occurrence/member not observed',
  ]);
});

test('a missing reading reference is a failure', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run, recorded } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(expectReadingOrder(run, 7, orderStep('before', null, reference())), false);
  assert.deepEqual(recorded.failures, ['step 7: clock reading: missing reading reference']);
});

test('a scalar that does not match the encoding is a failure', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run, recorded } = runFor(target, [sampled(17)]);
  assert.equal(expectReadingOrder(run, 8, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.failures, [
    'step 8: clock reading: reading scalar does not match encoding',
  ]);
});

test('an error the target itself raised is a failure', () => {
  const unixReference = reference({
    contract: {
      encoding: 'unix_seconds',
      origins: [{ role: 'producer_process', offset: 'encoding_defined_epoch' }],
    },
  });
  const refusing = {
    observeClockReading(): ClockReadingEvidence {
      throw new Error('the adapter has no epoch for this occurrence');
    },
  };
  const { run, recorded } = runFor(refusing, [sampled(new JsonNumber('1709296245'))]);
  assert.equal(
    expectReadingOrder(run, 9, orderStep('before', unixReference, unixReference)),
    false,
  );
  assert.deepEqual(recorded.failures, [
    'step 9: clock reading: the adapter has no epoch for this occurrence',
  ]);
});

test('unix seconds arrive as a token, as a whole number, or not at all', () => {
  const unixReference = reference({
    contract: {
      encoding: 'unix_seconds',
      origins: [{ role: 'producer_process', offset: 'encoding_defined_epoch' }],
    },
  });
  const observe = (value: unknown): { verdict: boolean; recorded: Recorded } => {
    const target = new RecordingTarget((request) =>
      evidenceFor(occurrenceKey(request.reading), { formatter: 'unix_seconds' }),
    );
    const { run, recorded } = runFor(target, [sampled(value)]);
    const verdict = expectReadingOrder(run, 1, orderStep('equal', unixReference, unixReference));
    return { verdict, recorded };
  };
  assert.equal(observe(new JsonNumber('1709296245')).verdict, true);
  assert.equal(observe(new JsonNumber('1.709296245e9')).verdict, true);
  assert.equal(observe(1709296245).verdict, true);
  // A string is not one of the four kinds the unix_seconds arm reads, even though the same string
  // would be the scalar for every other encoding.
  for (const refused of [
    '1709296245',
    1709296245.5,
    -1,
    253402300800,
    Number.NaN,
    Number.POSITIVE_INFINITY,
    true,
    null,
  ]) {
    const { verdict, recorded } = observe(refused);
    assert.equal(verdict, false, `refused ${String(refused)}`);
    assert.deepEqual(recorded.failures, [
      'step 1: clock reading: reading scalar does not match encoding',
    ]);
  }
});

test('the request the target is handed cannot rewrite the contract the assertion uses', () => {
  const declared = reference();
  const target = new RecordingTarget((request) => {
    request.reading.contract.origins[0]!.offset = 'encoding_defined_epoch';
    request.reading.contract.origins.push({ role: 'unknown', offset: 'unknown' });
    return evidenceFor(occurrenceKey(request.reading));
  });
  const { run, recorded } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(expectReadingOrder(run, 1, orderStep('equal', declared, declared)), true);
  assert.deepEqual(recorded.failures, []);
  assert.deepEqual(declared.contract.origins, [
    { role: 'producer_process', offset: 'encoded_offset' },
  ]);
});

test('the request carries the run correlation and the reference the step named', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  expectReadingOrder(run, 1, orderStep('equal', reference(), reference()));
  assert.equal(target.requests.length, 2);
  assert.equal(target.requests[0]!.correlation, 'corr-1');
  assert.equal(target.requests[0]!.reading.field, 'at');
  assert.equal(target.requests[0]!.reading.declared_type, 'Timestamp');
});

test('the occurrence counts only events of the named type', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run } = runFor(target, [
    sampled('2024-03-01T12:30:45Z', 'chronology.reading.Other'),
    sampled('2024-03-01T12:30:45Z'),
    sampled('2024-03-01T12:30:46Z'),
  ]);
  assert.equal(
    expectReadingOrder(
      run,
      1,
      orderStep('before', reference({ occurrence: 0 }), reference({ occurrence: 1 })),
    ),
    true,
  );
});

test('a member the event does not carry is not observed', () => {
  const target = new RecordingTarget((request) => evidenceFor(occurrenceKey(request.reading)));
  const { run, recorded } = runFor(target, [{ event: 'chronology.reading.Sampled', payload: {} }]);
  assert.equal(expectReadingOrder(run, 1, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.failures, [
    'step 1: clock reading: reading event occurrence/member not observed',
  ]);
});

test('a target that answers ErrUnsupported is skipped, not failed', () => {
  const refusing = {
    observeClockReading(): ClockReadingEvidence {
      throw new Error('no epoch for this occurrence', { cause: ErrUnsupported });
    },
  };
  const { run, recorded } = runFor(refusing, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(expectReadingOrder(run, 1, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.failures, []);
  assert.deepEqual(recorded.skips, ['clock comparison unsupported: no epoch for this occurrence']);
});

test('evidence whose identity is unknown is reported as unsupported, wrapped', () => {
  // ResolveClockReading answering UnknownEvidence is the one refusal the accessor promotes to the
  // ErrUnsupported sentinel, so a later `errors.Is` reader sees it as a capability gap.
  const target = new RecordingTarget((request) =>
    evidenceFor(occurrenceKey(request.reading), { epoch: '' }),
  );
  const { run, recorded } = runFor(target, [sampled('2024-03-01T12:30:45Z')]);
  assert.equal(expectReadingOrder(run, 1, orderStep('before', reference(), reference())), false);
  assert.deepEqual(recorded.failures, []);
  assert.equal(recorded.skips.length, 1);
  assert.equal(
    recorded.skips[0],
    'clock comparison unsupported: the target does not expose this: clock reading: UnknownEvidence',
  );
});
