import test from 'node:test';
import { run } from './dist/runtime.js';
import { servedTarget } from './served.mjs';

const target = servedTarget();
try {
  await test('suite', (t) => run(t, () => target));
} finally {
  target.close();
}
