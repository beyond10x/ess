// The Rust reference runner's way into a JavaScript target (`tests/typescript_suite_versions.rs`).
//
// One JSON request per line on stdin — `{method, args}` — and one answer per line on stdout:
// `{ok}`, `{missing: true}` for an optional method the target does not define, or `{error,
// unsupported}`. `beginScenario` builds a fresh target first, as the TypeScript runner builds one
// per scenario, so the two runners meet the same target in the same state.
import { createInterface } from 'node:readline';
import { isUnsupported, JsonNumber } from './dist/runtime.js';
import { makeTarget } from './target.mjs';

let target = makeTarget();
// A `JsonNumber` travels as its own digits, so an exact integer reaches Rust unrounded.
const marked = (_key, value) => (value instanceof JsonNumber ? `\u0000number:${value.raw}` : value);
const answer = (value) =>
  process.stdout.write(
    `${JSON.stringify(value, marked).replace(/"\\u0000number:([^"]*)"/g, '$1')}\n`,
  );

for await (const line of createInterface({ input: process.stdin })) {
  const { method, args } = JSON.parse(line);
  if (method === 'beginScenario') {
    target = makeTarget();
  }
  if (typeof target[method] !== 'function') {
    answer({ missing: true });
    continue;
  }
  try {
    const value = await target[method](args);
    answer({ ok: value ?? null });
  } catch (error) {
    answer({ error: String(error?.message ?? error), unsupported: isUnsupported(error) });
  }
}
