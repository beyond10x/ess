// Runs `explore` once per case in `ESS_EXPLORE_CASES` and writes what it found, so
// `tests/explore.rs` can compare this lane with the Go lane (`explore_driver_test.go`).
//
// A case is `{name, mutant, grade, options, allowExcluded}`. Each is its own named test, and each
// writes `<name>.json` (the result) and `<name>.assert` (`ok`, `failed: <first line>` or
// `refused: <message>`) into `ESS_EXPLORE_OUT`. Files rather than console lines: Node's TAP
// reporter escapes what a test prints.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { assertExplored, explore, goMarshal, Mulberry32 } from './dist/index.js';
import { newTarget } from './explore-target.mjs';

const out = process.env.ESS_EXPLORE_OUT ?? '.';
const random = new Mulberry32(1);
const outputs = [];
for (let index = 0; index < 5; index += 1) outputs.push(random.nextUint32());
writeFileSync(join(out, 'mulberry32'), `${outputs.join(' ')}\n`);

const cases = JSON.parse(process.env.ESS_EXPLORE_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    let result;
    try {
      result = await explore(
        () => newTarget(one.mutant || undefined, one.grade || 'low'),
        one.options ?? {},
      );
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
