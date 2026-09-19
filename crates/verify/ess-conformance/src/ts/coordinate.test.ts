// The coordinate reader answers exactly what `reading/coordinate.go` answers.
//
// Every expectation below was read off the Go file itself: `coordinate.go` was compiled verbatim
// against this vector table and its answers recorded, so a divergence between the two runtimes is a
// failing case here rather than a verdict an adopter has to notice.
import assert from 'node:assert/strict';
import test from 'node:test';

import {
  compareClockReadings,
  resolveClockReading,
  type ClockCoordinate,
  type ClockReadingEvidence,
  type ReadingOriginPair,
} from './coordinate.js';

const OCCURRENCE = 'chronology.Sampled#0.at';
const CORRELATION = 'corr-1';

function evidence(overrides: Partial<ClockReadingEvidence> = {}): ClockReadingEvidence {
  return {
    correlation: CORRELATION,
    occurrence: OCCURRENCE,
    processInstance: 'proc-1',
    epoch: 'epoch-1',
    origin: 'producer_process',
    formatter: 'encoded_offset',
    offsetMinutes: null,
    ...overrides,
  };
}

const OFFSET_ORIGINS: ReadingOriginPair[] = [['producer_process', 'encoded_offset']];
const UNIX_ORIGINS: ReadingOriginPair[] = [['producer_process', 'encoding_defined_epoch']];
const LOCAL_ORIGINS: ReadingOriginPair[] = [['producer_process', 'requires_observation']];

/** name, encoding, origins, value, evidence, expected millis or expected error message. */
type Vector = [string, string, ReadingOriginPair[], string, ClockReadingEvidence, number | string];

const UNIX = evidence({ formatter: 'unix_seconds' });
const LOCAL = evidence({ formatter: 'fixed_offset', offsetMinutes: 120 });

const VECTORS: Vector[] = [
  [
    'offset text, Z',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence(),
    1709296245000,
  ],
  [
    'offset text, millis and Z',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45.123Z',
    evidence(),
    1709296245123,
  ],
  [
    'offset text, plus offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45+02:00',
    evidence(),
    1709289045000,
  ],
  [
    'offset text, minus offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45-05:30',
    evidence(),
    1709316045000,
  ],
  [
    'offset text, max offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45+14:00',
    evidence(),
    1709245845000,
  ],
  [
    'offset text, offset 14:01 refused',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45+14:01',
    evidence(),
    'clock reading: OutOfRange',
  ],
  [
    'offset text, epoch floor',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '1970-01-01T00:00:00Z',
    evidence(),
    0,
  ],
  [
    'offset text, below epoch by offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '1970-01-01T00:00:00+01:00',
    evidence(),
    'clock reading: OutOfRange',
  ],
  [
    'offset text, ceiling',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '9999-12-31T23:59:59.999Z',
    evidence(),
    253402300799999,
  ],
  [
    'offset text, year 10000 refused',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '10000-01-01T00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, year 1969 refused',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '1969-12-31T23:59:59Z',
    evidence(),
    'clock reading: OutOfRange',
  ],
  [
    'offset text, feb 29 leap',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-02-29T00:00:00Z',
    evidence(),
    1709164800000,
  ],
  [
    'offset text, feb 29 non-leap',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2023-02-29T00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, feb 30',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-02-30T00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, month 13',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-13-01T00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, hour 24',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T24:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, second 60',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:60Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, too short',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:0Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, too long',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00.123456+02:00',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, dot but truncated',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00.12Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, bad separator',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01 00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, non-ascii',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:0éZ',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, no suffix',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, lowercase z',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, offset missing colon',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00+0200',
    evidence(),
    'clock reading: InvalidValue',
  ],
  ['unix seconds', 'unix_seconds', UNIX_ORIGINS, '1709296245', UNIX, 1709296245000],
  ['unix seconds zero', 'unix_seconds', UNIX_ORIGINS, '0', UNIX, 0],
  ['unix seconds ceiling', 'unix_seconds', UNIX_ORIGINS, '253402300799', UNIX, 253402300799000],
  [
    'unix seconds above ceiling',
    'unix_seconds',
    UNIX_ORIGINS,
    '253402300800',
    UNIX,
    'clock reading: OutOfRange',
  ],
  ['unix seconds negative', 'unix_seconds', UNIX_ORIGINS, '-1', UNIX, 'clock reading: OutOfRange'],
  [
    'unix seconds leading zero',
    'unix_seconds',
    UNIX_ORIGINS,
    '01',
    UNIX,
    'clock reading: InvalidValue',
  ],
  [
    'unix seconds plus sign',
    'unix_seconds',
    UNIX_ORIGINS,
    '+1',
    UNIX,
    'clock reading: InvalidValue',
  ],
  [
    'unix seconds not a number',
    'unix_seconds',
    UNIX_ORIGINS,
    'abc',
    UNIX,
    'clock reading: InvalidValue',
  ],
  [
    'unix seconds with offset minutes',
    'unix_seconds',
    UNIX_ORIGINS,
    '1',
    evidence({ formatter: 'unix_seconds', offsetMinutes: 0 }),
    'clock reading: IncompatibleFormatter',
  ],
  [
    'local literal z with observed offset',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45.123Z',
    LOCAL,
    1709289045123,
  ],
  [
    'local literal z negative offset',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset', offsetMinutes: -330 }),
    1709316045123,
  ],
  [
    'local literal z without millis',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45Z',
    LOCAL,
    'clock reading: InvalidValue',
  ],
  [
    'local literal z with explicit offset',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45.123+02:00',
    LOCAL,
    'clock reading: InvalidValue',
  ],
  [
    'local literal z offset out of range',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset', offsetMinutes: 900 }),
    'clock reading: OutOfRange',
  ],
  [
    'local literal z missing offset',
    'local_date_time_millis_literal_z',
    LOCAL_ORIGINS,
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset' }),
    'clock reading: IncompatibleFormatter',
  ],
  [
    'unknown origin declared',
    'local_date_time_millis_literal_z',
    [['unknown', 'unknown']],
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset', offsetMinutes: 0 }),
    'clock reading: IncompatibleOrigin',
  ],
  [
    'correlation mismatch',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ correlation: 'other' }),
    'clock reading: MismatchedOccurrence',
  ],
  [
    'occurrence mismatch',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ occurrence: 'other#0.at' }),
    'clock reading: MismatchedOccurrence',
  ],
  [
    'empty process instance',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ processInstance: '' }),
    'clock reading: UnknownEvidence',
  ],
  [
    'empty epoch',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ epoch: '' }),
    'clock reading: UnknownEvidence',
  ],
  [
    'origin unknown observed',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ origin: 'unknown' }),
    'clock reading: UnknownEvidence',
  ],
  [
    'formatter unknown',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ formatter: 'unknown' }),
    'clock reading: UnknownEvidence',
  ],
  [
    'formatter incompatible',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ formatter: 'unix_seconds' }),
    'clock reading: IncompatibleFormatter',
  ],
  [
    'observed origin not declared',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ origin: 'consumer_process' }),
    'clock reading: IncompatibleOrigin',
  ],
  [
    'no origins',
    'offset_date_time_text',
    [],
    '2024-03-01T12:30:45Z',
    evidence(),
    'clock reading: InvalidContract',
  ],
  [
    'four origins',
    'offset_date_time_text',
    [
      ['producer_process', 'encoded_offset'],
      ['consumer_process', 'encoded_offset'],
      ['unknown', 'encoded_offset'],
      ['producer_process', 'encoded_offset'],
    ],
    '2024-03-01T12:30:45Z',
    evidence(),
    'clock reading: InvalidContract',
  ],
  // Four *distinct* origins, so the length bound is the only thing that can refuse them. The
  // four-with-a-duplicate case below is refused by the duplicate check whatever the bound says.
  [
    'four distinct origins',
    'local_date_time_millis_literal_z',
    [
      ['producer_process', 'requires_observation'],
      ['consumer_process', 'requires_observation'],
      ['unknown', 'requires_observation'],
      ['producer_process', 'unknown'],
    ],
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset', offsetMinutes: 0 }),
    'clock reading: InvalidContract',
  ],
  [
    'three distinct origins',
    'local_date_time_millis_literal_z',
    [
      ['producer_process', 'requires_observation'],
      ['consumer_process', 'requires_observation'],
      ['unknown', 'requires_observation'],
    ],
    '2024-03-01T12:30:45.123Z',
    evidence({ formatter: 'fixed_offset', offsetMinutes: 0 }),
    1709296245123,
  ],
  [
    'duplicate origins',
    'offset_date_time_text',
    [
      ['producer_process', 'encoded_offset'],
      ['producer_process', 'encoded_offset'],
    ],
    '2024-03-01T12:30:45Z',
    evidence(),
    'clock reading: InvalidContract',
  ],
  [
    'origin role invalid',
    'offset_date_time_text',
    [['other', 'encoded_offset']],
    '2024-03-01T12:30:45Z',
    evidence(),
    'clock reading: InvalidContract',
  ],
  [
    'origin authority contradicts encoding',
    'offset_date_time_text',
    [['producer_process', 'encoding_defined_epoch']],
    '2024-03-01T12:30:45Z',
    evidence(),
    'clock reading: InvalidContract',
  ],
  [
    'offset text, non-digit year is out of range before it is invalid',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    'abcd-01-01T00:00:00Z',
    evidence(),
    'clock reading: OutOfRange',
  ],
  [
    'offset text, non-digit month',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-ab-01T00:00:00Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, non-digit millis',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00.abcZ',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, non-digit offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00+ab:00',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, three fractional digits',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00.999Z',
    evidence(),
    1704067200999,
  ],
  [
    'offset text, four fractional digits',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-01-01T00:00:00.9999Z',
    evidence(),
    'clock reading: InvalidValue',
  ],
  [
    'offset text, ceiling shifted by max offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '9999-12-31T23:59:59.999+14:00',
    evidence(),
    253402250399999,
  ],
  [
    'offset text, floor shifted by min offset',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '1970-01-01T00:00:00.000-14:00',
    evidence(),
    50400000,
  ],
  [
    'identity over 512',
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    evidence({ processInstance: 'x'.repeat(513) }),
    'clock reading: UnknownEvidence',
  ],
];

test('resolveClockReading answers what the Go coordinate reader answers', async (t) => {
  for (const [name, encoding, origins, value, facts, expected] of VECTORS) {
    await t.test(name, () => {
      if (typeof expected === 'number') {
        const coordinate = resolveClockReading(
          encoding,
          origins,
          value,
          facts,
          CORRELATION,
          OCCURRENCE,
        );
        assert.equal(coordinate.unixMillis, expected);
        assert.equal(coordinate.processInstance, facts.processInstance);
        assert.equal(coordinate.epoch, facts.epoch);
        return;
      }
      assert.throws(
        () => resolveClockReading(encoding, origins, value, facts, CORRELATION, OCCURRENCE),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.equal(error.message, expected);
          return true;
        },
      );
    });
  }
});

test('an identity of exactly 512 characters is still admitted', () => {
  const facts = evidence({ epoch: 'e'.repeat(512) });
  const coordinate = resolveClockReading(
    'offset_date_time_text',
    OFFSET_ORIGINS,
    '2024-03-01T12:30:45Z',
    facts,
    CORRELATION,
    OCCURRENCE,
  );
  assert.equal(coordinate.epoch, 'e'.repeat(512));
});

test('an absent offsetMinutes field reads the same as an explicit null', () => {
  const withoutField = {
    correlation: CORRELATION,
    occurrence: OCCURRENCE,
    processInstance: 'proc-1',
    epoch: 'epoch-1',
    origin: 'producer_process',
    formatter: 'encoded_offset',
  } as ClockReadingEvidence;
  assert.equal(
    resolveClockReading(
      'offset_date_time_text',
      OFFSET_ORIGINS,
      '2024-03-01T12:30:45Z',
      withoutField,
      CORRELATION,
      OCCURRENCE,
    ).unixMillis,
    1709296245000,
  );
});

function coordinate(
  unixMillis: number,
  processInstance = 'proc-1',
  epoch = 'epoch-1',
): ClockCoordinate {
  return { unixMillis, processInstance, epoch };
}

test('compareClockReadings orders only within one observed epoch', () => {
  assert.equal(compareClockReadings(coordinate(1), coordinate(2)), -1);
  assert.equal(compareClockReadings(coordinate(2), coordinate(1)), 1);
  assert.equal(compareClockReadings(coordinate(2), coordinate(2)), 0);
  assert.throws(
    () => compareClockReadings(coordinate(1, 'proc-1'), coordinate(1, 'proc-2')),
    /^Error: clock reading: DifferentClock$/,
  );
  assert.throws(
    () =>
      compareClockReadings(coordinate(1, 'proc-1', 'epoch-1'), coordinate(1, 'proc-1', 'epoch-2')),
    /^Error: clock reading: DifferentClock$/,
  );
  for (const blank of [
    [coordinate(1, ''), coordinate(1)],
    [coordinate(1, 'proc-1', ''), coordinate(1)],
    [coordinate(1), coordinate(1, '')],
    [coordinate(1), coordinate(1, 'proc-1', '')],
  ]) {
    assert.throws(
      () => compareClockReadings(blank[0]!, blank[1]!),
      /^Error: clock reading: UnknownEvidence$/,
    );
  }
});

test('an unknown identity is reported before a different clock is', () => {
  assert.throws(
    () => compareClockReadings(coordinate(1, ''), coordinate(1, 'other', 'other')),
    /UnknownEvidence/,
  );
});
