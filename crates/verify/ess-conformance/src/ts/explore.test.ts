// The explorer's `ess-history/2` writer and decision receipt (beyond10x/ess#244), held to the same
// committed documents and instant vectors the Rust reader and the Go explorer are
// (`tests/history_format2.rs` in `ess-conformance`).
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';

import {
  completeRecorded,
  decisionInstantOf,
  historyText,
  parseDecisionInstant,
  type HistoryOperation,
  type PendingCommand,
} from './explore.js';

const FIXTURES = 'crates/verify/ess-conformance/tests/fixtures/history2';

function fixture(name: string): string {
  let directory = import.meta.dirname;
  for (let depth = 0; depth < 12; depth += 1) {
    const candidate = join(directory, FIXTURES, name);
    if (existsSync(candidate)) return readFileSync(candidate, 'utf8');
    directory = dirname(directory);
  }
  throw new Error(`${FIXTURES}/${name} is not above ${import.meta.dirname}`);
}

function operations(times: boolean): HistoryOperation[] {
  const at = (text: string): { decisionTime?: string } =>
    times ? { decisionTime: parseDecisionInstant(text) } : {};
  const subject = '00000000-0000-4000-8000-00000000a001';
  const base = { retryOf: 0, creates: '', source: '' };
  return [
    {
      ...base,
      client: 0,
      command: 'demo.offers.OpenOffer',
      subjectKey: subject,
      invokedAt: 1,
      returnedAt: 2,
      outcome: 'opened',
      creates: 'demo.offers.Offer',
      ...at('2000-06-01T00:00:00Z'),
    },
    {
      ...base,
      client: 0,
      command: 'demo.offers.AcceptOffer',
      subjectKey: subject,
      invokedAt: 3,
      ...at('2026-10-04T12:00:00.123456789Z'),
    },
    {
      ...base,
      client: 1,
      command: 'demo.offers.AcceptOffer',
      subjectKey: subject,
      invokedAt: 4,
      returnedAt: 5,
      outcome: 'lapsed',
    },
    {
      ...base,
      client: 1,
      command: 'demo.offers.Offers',
      subjectKey: '',
      invokedAt: 6,
      returnedAt: 7,
      outcome: 'read',
      source: 'demo.offers.Offer',
      rows: [subject],
    },
    {
      ...base,
      client: 0,
      command: 'demo.offers.AcceptOffer',
      subjectKey: subject,
      invokedAt: 8,
      returnedAt: 9,
      outcome: 'accepted',
      retryOf: 2,
      ...at('2000-07-01T00:00:00.5Z'),
    },
  ];
}

const DIGEST = 'ab'.repeat(32);

test('a recorded decision time is written as ess-history/2, byte for byte', () => {
  assert.equal(historyText(DIGEST, 7, 2, operations(true)), fixture('written.json'));
});

test('a history with no decision time is written as ess-history/1, byte for byte', () => {
  assert.equal(historyText(DIGEST, 7, 2, operations(false)), fixture('written-format1.json'));
});

test('a decision instant has the one spelling the Rust and Go readers admit', () => {
  const vectors = JSON.parse(fixture('decision-time-vectors.json')) as {
    valid: string[];
    invalid: string[];
  };
  assert.ok(vectors.valid.length >= 8 && vectors.invalid.length >= 25, 'the vectors were read');
  for (const valid of vectors.valid) assert.equal(parseDecisionInstant(valid), valid);
  for (const invalid of vectors.invalid) {
    assert.throws(() => parseDecisionInstant(invalid), Error, JSON.stringify(invalid));
  }
  assert.equal(
    decisionInstantOf(new Date(Date.UTC(2026, 9, 4, 12, 0, 0, 0))),
    '2026-10-04T12:00:00Z',
  );
  assert.equal(
    decisionInstantOf(new Date(Date.UTC(2026, 9, 4, 12, 0, 0, 120))),
    '2026-10-04T12:00:00.12Z',
  );
});

test("the receipt is the call's own, and the default is complete once with no time", async () => {
  let completed = 0;
  let recorded = 0;
  const plain: PendingCommand = {
    complete: () => {
      completed += 1;
      return { outcome: 'lapsed', directEvents: [] };
    },
  };
  assert.deepEqual(await completeRecorded(plain), {
    result: { outcome: 'lapsed', directEvents: [] },
  });
  assert.equal(completed, 1);

  const lost: PendingCommand = {
    complete: () => {
      throw new Error('never called');
    },
    completeRecorded: () => {
      recorded += 1;
      return { error: new Error('lost'), decisionTime: '2026-10-04T12:00:00.123456789Z' };
    },
  };
  const receipt = await completeRecorded(lost);
  assert.equal(recorded, 1);
  assert.equal(receipt.decisionTime, '2026-10-04T12:00:00.123456789Z', 'an error keeps its time');
  assert.ok(receipt.error instanceof Error);

  const failing: PendingCommand = {
    complete: () => {
      throw new Error('refused before deciding');
    },
  };
  const refused = await completeRecorded(failing);
  assert.equal(refused.decisionTime, undefined);
  assert.ok(refused.error instanceof Error);
});
