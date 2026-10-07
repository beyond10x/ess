// Runs `explore` once per case in `ESS_EXPLORE_CASES` against `restart-target.mjs`, so
// `tests/explore_restart.rs` can compare this lane with the Go lane (`restart_driver_test.go`).
//
// A case is `{name, mode, options, allowExcluded, concurrent}`; a `concurrent` case calls
// `exploreConcurrent` instead, which only this lane has a way to pass `restartEvery` to. Each
// writes `<name>.json` (the result), `<name>.assert` (`ok`, `failed: <first line>` or
// `refused: <message>`) and `<name>.processes` (how many distinct backend processes its targets
// started) into `ESS_EXPLORE_OUT`.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { assertExplored, explore, exploreConcurrent, goMarshal } from './dist/index.js';
import { newTarget, processes } from './restart-target.mjs';

const out = process.env.ESS_EXPLORE_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_EXPLORE_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    const before = processes.size;
    let result;
    try {
      const explorer = one.concurrent === true ? exploreConcurrent : explore;
      result = await explorer(() => newTarget(one.mode), one.options ?? {});
    } catch (error) {
      write('.processes', `${processes.size - before}`);
      write('.assert', `refused: ${error.message}`);
      return;
    }
    write('.processes', `${processes.size - before}`);
    write('.json', goMarshal(result));
    try {
      assertExplored(result, { allowExcluded: one.allowExcluded === true });
      write('.assert', 'ok');
    } catch (error) {
      write('.assert', `failed: ${error.message.split('\n')[0]}`);
    }
  });
}
