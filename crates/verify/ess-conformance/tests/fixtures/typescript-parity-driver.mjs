// The emitted TypeScript runner against the same JavaScript target the Rust runner reaches through
// `typescript-parity-proxy.mjs` (`tests/typescript_suite_versions.rs`).
import test from 'node:test';
import { run } from './dist/runtime.js';
import { makeTarget } from './target.mjs';

await test('suite', (t) => run(t, makeTarget));
