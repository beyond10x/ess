import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { admitSuiteDocument, run } from './dist/runtime.js';
// Aggregate suites are not a TypeScript lane: the runtime refuses the suite's version before it
// constructs a target, so no callback may run.
let constructed = 0;
await test('sessions', async (t) => {
  try {
    await run(t, () => {
      constructed += 1;
      throw Error('target must not be constructed');
    });
  } finally {
    console.log(`constructed: ${constructed}`);
  }
});
// A coverage document this runtime reads, carrying an aggregate refusal, is refused by the code.
await test('aggregate refusals need coverage suite/17', () => {
  const raw = readFileSync(join(process.env.ESS_AGGREGATE_DOCS, 'coverage-11.json'), 'utf8');
  assert.throws(() => admitSuiteDocument(raw, false), /aggregate view refusals require suite\/17/);
  console.log('aggregate-refusal-admission: refused');
});
