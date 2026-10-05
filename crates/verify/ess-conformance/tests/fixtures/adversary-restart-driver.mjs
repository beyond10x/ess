// Runs `explore` once per case in `ESS_EXPLORE_CASES` against `adversary-restart-target.mjs`, writing
// `<name>.json`, `<name>.assert` and `<name>.processes` into `ESS_EXPLORE_OUT`, as
// `restart-driver.mjs` does.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { assertExplored, explore, goMarshal } from './dist/index.js';
import { newTarget, processes } from './adversary-restart-target.mjs';

const out = process.env.ESS_EXPLORE_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_EXPLORE_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    const before = processes.size;
    let result;
    try {
      result = await explore(() => newTarget(one.mode), one.options ?? {});
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
