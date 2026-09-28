// Runs `exploreConcurrent` once per case in `ESS_CONCURRENT_CASES` and writes what it found, so
// `crates/edge/ess-cli/tests/explore_concurrent.rs` can compare this lane with the Go lane
// (`explore_concurrent_driver_test.go`).
//
// A case is `{name, target, mutant, pathEnv, options}`. Each is its own named test. Each writes its
// histories into `ESS_CONCURRENT_OUT/<name>/`, and `<name>.json` (the result) and `<name>.problem`
// (what `concurrentProblem` says, or `none`), or `<name>.refused` (the error), beside them. `pathEnv`, where present, is the PATH the case runs under.
// `ESS_CONCURRENT_SPEC` is the specification `ess` checks against.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { concurrentProblem, exploreConcurrent, goMarshal, SplitMix64 } from './dist/index.js';
import { newBillingTarget } from './explore-concurrent-billing-target.mjs';
import { newTarget } from './explore-target.mjs';

const out = process.env.ESS_CONCURRENT_OUT ?? '.';
const draws = new SplitMix64(1);
const outputs = [];
for (let index = 0; index < 5; index += 1) outputs.push(draws.next().toString());
writeFileSync(join(out, 'splitmix64'), `${outputs.join(' ')}\n`);

const cases = JSON.parse(process.env.ESS_CONCURRENT_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    const path = process.env.PATH;
    if (one.pathEnv !== undefined) process.env.PATH = one.pathEnv;
    try {
      const make =
        one.target === 'billing' ? () => newBillingTarget() : () => newTarget(one.mutant ?? '', 'low');
      const result = await exploreConcurrent(make, {
        ...(one.options ?? {}),
        path: process.env.ESS_CONCURRENT_SPEC,
        out: join(out, one.name),
      });
      write('.json', goMarshal(result));
      write('.problem', concurrentProblem(result) ?? 'none');
    } catch (error) {
      write('.refused', error.message);
    } finally {
      process.env.PATH = path;
    }
  });
}
