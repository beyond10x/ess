// Runs `explore` once per case in `ESS_EXPLORE_CASES` against `explore-external-target.mjs` and
// writes what it found, so `tests/explore_external.rs` can compare this lane with the Go lane
// (`explore_external_driver_test.go`).
//
// A case is `{name, mode, options, allowExcluded}`. Each is its own named test, and each writes
// `<name>.json` (the result) and `<name>.assert` (`ok`, `failed: <first line>` or
// `refused: <message>`) into `ESS_EXPLORE_OUT`.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { assertExplored, explore, goMarshal } from './dist/index.js';
import { newTarget } from './explore-external-target.mjs';

const out = process.env.ESS_EXPLORE_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_EXPLORE_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    let result;
    try {
      result = await explore(() => newTarget(one.mode ?? ''), one.options ?? {});
    } catch (error) {
      write('.assert', `refused: ${error.message}`);
      return;
    }
    write('.json', goMarshal(result));
    try {
      assertExplored(result, { allowExcluded: one.allowExcluded === true });
      write('.assert', 'ok');
    } catch (error) {
      write('.assert', `failed: ${error.message.split('\n')[0]}`);
    }
  });
}
