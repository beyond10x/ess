// Adversary pass 1 over wave D335 unit U3
// (story:a-binary64-value-round-trips-through-a-conformance-suite).
//
// `docs/design/model-binary64.md` § "Comparing a value": two tokens at a Binary64 position are one
// value when the binary64 nearest to each is equal, so `0.1` and `0.10000000000000001` are one
// value. The Rust runner (`Node` number equality) and the Go runner (`numberValue`) read a
// non-integral token through its binary64; a view row reaches this runtime's `rowMatches`, which
// compares by the exact decimal.
import assert from 'node:assert/strict';
import test from 'node:test';

import { JsonNumber, rowMatches, type Node } from './runtime.js';

test('a view row compares a Binary64 by its nearest binary64, as the Rust and Go runners do', () => {
  const token = (raw: string): Node => new JsonNumber(raw);
  for (const [expected, actual] of [
    ['1.5', '1.50'],
    ['1.5', '15e-1'],
    ['0', '-0.0'],
    ['0.1', '0.10000000000000001'],
    ['1.5', '1.50000000000000000001'],
  ]) {
    assert.equal(
      rowMatches({ level: token(actual!) }, { level: token(expected!) }),
      true,
      `${expected} expected, ${actual} answered: one binary64`,
    );
  }
});
