import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { admitSuiteDocument, run } from './dist/runtime.js';
// Aggregate suites are a TypeScript lane (beyond10x/ess#188): the runtime admits the suite and
// constructs a target, which throws here, so the run fails at the target and not at admission.
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
