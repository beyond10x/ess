// `predicate.ts` held to the vectors the Go evaluator is held to.
//
// This file is not part of the emitted package. The Go target emits `predicate.go` and no
// `predicate_test.go`; the checks that hold the Go evaluator live in this repository, under
// `crates/verify/ess-conformance/tests/fixtures/` (`quoted-predicates.go`,
// `quoted-special-boundaries.go`, `adversary-quoted-specials.go`), and are compiled beside a
// freshly emitted package by `tests/quoted_predicates.rs`. Every vector in those three fixtures is
// answered below, in TypeScript, against the same expressions and the same expected literals — so
// a divergence between the two evaluators is a red test here rather than a suite that passes in
// one language and fails in the other.
//
// It also answers the normative primitive corpus,
// `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`, for the part of it a
// predicate evaluator can answer: the spelling of every number in it, and every ordering it
// states. See `the corpus` below for the one ordering both evaluators collapse and why.

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';

import {
  Operand,
  Predicate,
  admitPredicateExpression,
  admitPredicateLeaf,
  admitPredicatePath,
  admitQuotedOperand,
  facts,
  factPath,
  fromNode,
  goTrimSpace,
  isTruthy,
  meaningDecimal,
  parseDecimalLiteral,
  parseLeaf,
  parseLiteral,
  parseOperand,
  parsePredicate,
  splitPredicateComparison,
  truthAnd,
  TruthFalse,
  truthNot,
  truthOf,
  truthOr,
  TruthTrue,
  TruthUnknown,
} from './predicate.js';
import type { FactSource, Truth } from './predicate.js';
import type { Node, Row } from './runtime.js';

const source = (entries: Record<string, Node>): FactSource => new Map(Object.entries(entries));

const raised = (body: () => unknown): string => {
  try {
    body();
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
  throw new assert.AssertionError({ message: 'nothing was thrown' });
};

// ---- three-valued logic -------------------------------------------------------------------

test('and is false when either side is, unknown when either side is, true otherwise', () => {
  const table: [Truth, Truth, Truth][] = [
    [TruthTrue, TruthTrue, TruthTrue],
    [TruthTrue, TruthFalse, TruthFalse],
    [TruthTrue, TruthUnknown, TruthUnknown],
    [TruthFalse, TruthTrue, TruthFalse],
    [TruthFalse, TruthFalse, TruthFalse],
    [TruthFalse, TruthUnknown, TruthFalse],
    [TruthUnknown, TruthTrue, TruthUnknown],
    [TruthUnknown, TruthFalse, TruthFalse],
    [TruthUnknown, TruthUnknown, TruthUnknown],
  ];
  for (const [left, right, want] of table) {
    assert.equal(truthAnd(left, right), want, `${left} and ${right}`);
  }
});

test('or is true when either side is, unknown when either side is, false otherwise', () => {
  const table: [Truth, Truth, Truth][] = [
    [TruthTrue, TruthTrue, TruthTrue],
    [TruthTrue, TruthFalse, TruthTrue],
    [TruthTrue, TruthUnknown, TruthTrue],
    [TruthFalse, TruthTrue, TruthTrue],
    [TruthFalse, TruthFalse, TruthFalse],
    [TruthFalse, TruthUnknown, TruthUnknown],
    [TruthUnknown, TruthTrue, TruthTrue],
    [TruthUnknown, TruthFalse, TruthUnknown],
    [TruthUnknown, TruthUnknown, TruthUnknown],
  ];
  for (const [left, right, want] of table) {
    assert.equal(truthOr(left, right), want, `${left} or ${right}`);
  }
});

test('not leaves unknown alone, which is the whole reason there are three values', () => {
  assert.equal(truthNot(TruthTrue), TruthFalse);
  assert.equal(truthNot(TruthFalse), TruthTrue);
  assert.equal(truthNot(TruthUnknown), TruthUnknown);
  assert.equal(truthOf(true), TruthTrue);
  assert.equal(truthOf(false), TruthFalse);
});

// ---- flattening a row ---------------------------------------------------------------------

test('a row flattens to a scalar per dotted path, and a collection publishes its size', () => {
  const row: Row = {
    total: { amount: 3, currency: 'EUR' },
    lines: [{ sku: 'a' }, { sku: 'b' }],
    tags: [],
    meta: {},
    note: null,
    flag: false,
  };
  assert.deepEqual([...facts(row).entries()].sort(), [
    ['flag', false],
    ['lines.0.sku', 'a'],
    ['lines.1.sku', 'b'],
    ['lines.count', 2],
    ['note', null],
    ['tags.count', 0],
    ['total.amount', 3],
    ['total.currency', 'EUR'],
  ]);
});

test('an empty mapping binds no fact at all, so reading it is unknown and not false', () => {
  assert.equal(facts({ meta: {} }).size, 0);
  assert.equal(parseLeaf('meta').evaluate(facts({ meta: {} })), TruthUnknown);
});

test('a nested collection publishes a count at every level', () => {
  const flattened = facts({ order: { lines: [[1, 2], [3]] } });
  assert.equal(flattened.get('order.lines.count'), 2);
  assert.equal(flattened.get('order.lines.0.count'), 2);
  assert.equal(flattened.get('order.lines.0.1'), 2);
  assert.equal(flattened.get('order.lines.1.count'), 1);
});

// ---- the quoted-operand vectors -------------------------------------------------------------
//
// `tests/fixtures/quoted-predicates.go` `TestQuotedTrailingTokensRefused`.

test('a quoted operand delimits exactly one operand and trailing tokens are refused', () => {
  for (const expression of [
    'to == "" or text == ""',
    "to == '' and text == ''",
    'to == "done" trailing',
    'to == "done""next"',
    'not to == "" or text == ""',
    'to == "unterminated',
    "to == 'unterminated",
    String.raw`to == "escaped\"`,
    String.raw`to == "slash\\" or text == ""`,
  ]) {
    assert.match(
      raised(() => parseLeaf(expression)),
      /structured any\/all\/not/,
      expression,
    );
  }
});

// `tests/fixtures/quoted-predicates.go` `TestQuotedValuesRemainExact`.
test('a quoted value reaches the comparison as the exact text it was written with', () => {
  for (const [expression, value] of [
    ['to == ""', ''],
    ["to == 'true'", 'true'],
    ['to == "or and not"', 'or and not'],
    [" \tto == '  spaced  ' \n", '  spaced  '],
    [String.raw`to == "say \"or\""`, String.raw`say \"or\"`],
    [String.raw`to == 'it\'s valid'`, String.raw`it\'s valid`],
    [String.raw`to == "path\\"`, String.raw`path\\`],
    ['to == "雪 and ☃"', '雪 and ☃'],
    ['to == "a == b or c != d"', 'a == b or c != d'],
  ] as [string, string][]) {
    const parsed = parseLeaf(expression);
    assert.equal(parsed.right.literal, value, expression);
    assert.equal(parsed.right.isFact, false, expression);
    assert.equal(parsed.evaluate(source({ to: value })), TruthTrue, expression);
  }
});

// `tests/fixtures/quoted-predicates.go` `TestQuotedStructuredAndPrefixNot`.
test('a structured composition and a prefix `not` compose to the same answer', () => {
  for (const expression of [
    { any: ['to == ""', { all: ["text == ''", { not: 'disabled' }] }] },
    'not to == ""',
  ] as Node[]) {
    assert.equal(
      fromNode(expression).evaluate(source({ to: 'filled', text: '', disabled: false })),
      TruthTrue,
      JSON.stringify(expression),
    );
  }
});

// `tests/fixtures/quoted-predicates.go` `TestQuotedComparisonOperatorsInsideLiteralRemainData`.
test('a comparison operator inside a quoted literal stays data', () => {
  for (const expression of ['to < "a == b"', 'to < "a != b"', "to < 'a >= b'"]) {
    const parsed = parseLeaf(expression);
    assert.equal(parsed.left.path, 'to', expression);
    assert.equal(parsed.op, '<', expression);
    assert.equal(parsed.evaluate(source({ to: '0' })), TruthTrue, expression);
  }
});

// `tests/fixtures/adversary-quoted-specials.go`.
test('a structured scalar keeps the exact text Rust keeps', () => {
  assert.equal(parsePredicate({ to: { eq: 'Ready' } }).right.literal, 'Ready');
  const parsed = parsePredicate({ to: { eq: 'NaN' } });
  assert.equal(parsed.right.literal, 'NaN');
  assert.equal(parsed.evaluate(source({ to: 'NaN' })), TruthTrue);
});

// `tests/fixtures/quoted-special-boundaries.go`, the text half: Go's `strconv` extensions
// (NaN, infinities, hexadecimal floats, underscores) and out-of-range exponents must not turn
// text into numbers.
test('a literal spelling no finite decimal grammar admits stays text', () => {
  for (const text of [
    'Ready',
    'NaN',
    'nan',
    'NAN',
    'Inf',
    '+Inf',
    '-Inf',
    'Infinity',
    '+Infinity',
    '-Infinity',
    'infinity',
    '0x1p2',
    '0x1.8p+1',
    '-0x1.8p+1',
    '1_000',
    '1e400',
    '-1e400',
  ]) {
    for (const parsed of [parsePredicate({ to: { eq: text } }), parseLeaf(`to == ${text}`)]) {
      assert.equal(parsed.right.isFact, false, text);
      assert.equal(parsed.right.literal, text, text);
      assert.equal(parsed.evaluate(source({ to: text })), TruthTrue, text);
    }
  }
});

// `tests/fixtures/quoted-special-boundaries.go`, the number half. `assert` here is
// `node:assert/strict`, whose `equal` is SameValue, so `-0` and `0` are distinguished.
test('a finite decimal spelling is a number, and its sign of zero survives', () => {
  for (const [text, want] of [
    ['0', 0],
    ['-0', -0],
    ['+1', 1],
    ['-1.25', -1.25],
    ['.5', 0.5],
    ['1.', 1],
    ['1e3', 1000],
    ['+1.25E-2', 0.0125],
    ['-2e+2', -200],
    ['1e-400', 0],
    ['-1e-400', -0],
  ] as [string, number][]) {
    for (const parsed of [parsePredicate({ to: { eq: text } }), parseLeaf(`to == ${text}`)]) {
      assert.equal(parsed.right.isFact, false, text);
      assert.equal(parsed.right.literal, want, text);
      assert.equal(parsed.evaluate(source({ to: want })), TruthTrue, text);
    }
  }
});

test('the two zeroes are one value to a comparison and two values to a literal', () => {
  assert.equal(parseLeaf('to == -0').right.literal, -0);
  assert.equal(parseLeaf('to == 0').evaluate(source({ to: -0 })), TruthTrue);
  assert.equal(parseLeaf('to == -0').evaluate(source({ to: 0 })), TruthTrue);
  assert.equal(parseLeaf('to < -0').evaluate(source({ to: 0 })), TruthFalse);
});

// ---- operands -------------------------------------------------------------------------------

test('a dotted bare word is a fact path and everything else is a literal', () => {
  assert.deepEqual(
    parseOperand('total.amount'),
    new Operand({ path: 'total.amount', isFact: true }),
  );
  assert.deepEqual(
    parseOperand(' total.amount '),
    new Operand({ path: 'total.amount', isFact: true }),
  );
  assert.deepEqual(parseOperand('"total.amount"'), new Operand({ literal: 'total.amount' }));
  assert.deepEqual(parseOperand("'total.amount'"), new Operand({ literal: 'total.amount' }));
  assert.deepEqual(parseOperand('1.25'), new Operand({ literal: 1.25 }));
  assert.deepEqual(parseOperand('Bridged'), new Operand({ literal: 'Bridged' }));
  assert.deepEqual(parseOperand('true'), new Operand({ literal: true }));
  assert.deepEqual(parseOperand('9.a'), new Operand({ literal: '9.a' }));
});

test('the left of a comparison is always a fact path, never a literal', () => {
  const parsed = parseLeaf('state == Bridged');
  assert.equal(parsed.left.isFact, true);
  assert.equal(parsed.left.path, 'state');
  assert.equal(parsed.right.isFact, false);
  assert.equal(parsed.right.literal, 'Bridged');
  assert.equal(parsed.evaluate(source({ state: 'Bridged' })), TruthTrue);
  assert.equal(parsed.evaluate(source({ state: 'Ringing' })), TruthFalse);
  assert.equal(parsed.evaluate(source({})), TruthUnknown);
});

test('a comparison against a fact path on the right reads a second fact', () => {
  const parsed = parseLeaf('total.paid >= total.due');
  assert.equal(parsed.right.isFact, true);
  assert.equal(parsed.evaluate(facts({ total: { paid: 5, due: 3 } })), TruthTrue);
  assert.equal(parsed.evaluate(facts({ total: { paid: 1, due: 3 } })), TruthFalse);
  assert.equal(parsed.evaluate(facts({ total: { paid: 1 } })), TruthUnknown);
});

test('a literal reads the same whether it arrives quoted, bare, boolean or numeric', () => {
  assert.equal(parseLiteral('"x"'), 'x');
  assert.equal(parseLiteral("'x'"), 'x');
  assert.equal(parseLiteral('""'), '');
  assert.equal(parseLiteral('true'), true);
  assert.equal(parseLiteral('false'), false);
  assert.equal(parseLiteral('3'), 3);
  assert.equal(parseLiteral('"'), '"');
  assert.equal(parseLiteral('word'), 'word');
});

test('the decimal grammar admits exactly what Rust `Number::parse_decimal` admits', () => {
  for (const text of ['0', '-0', '+1', '1.', '.5', '1e3', '1e-400', '-1e-400', '1E+2']) {
    assert.equal(parseDecimalLiteral(text)[1], true, text);
  }
  for (const text of [
    '',
    'NaN',
    'Inf',
    '0x1p2',
    '1_000',
    '1e400',
    '-1e400',
    '.',
    'e3',
    '1e',
    ' 1',
  ]) {
    assert.deepEqual(parseDecimalLiteral(text), [0, false], text);
  }
  assert.equal(
    meaningDecimal.test('1e400'),
    true,
    'the grammar admits it and the range refuses it',
  );
  assert.equal(factPath.test('total.amount'), true);
  assert.equal(factPath.test('9lives'), false);
  assert.equal(factPath.test('a-b.c_d'), true);
  assert.equal(factPath.test('a..b'), false);
});

// ---- comparison splitting ---------------------------------------------------------------------

test('the first operator outside quotes splits, two bytes before one at that position', () => {
  assert.deepEqual(splitPredicateComparison('a == b'), ['a ', '==', ' b', true]);
  assert.deepEqual(splitPredicateComparison('a <= b'), ['a ', '<=', ' b', true]);
  assert.deepEqual(splitPredicateComparison('a < b'), ['a ', '<', ' b', true]);
  assert.deepEqual(splitPredicateComparison('a >= b'), ['a ', '>=', ' b', true]);
  assert.deepEqual(splitPredicateComparison('a != b'), ['a ', '!=', ' b', true]);
  assert.deepEqual(splitPredicateComparison('"a > b" < c'), ['"a > b" ', '<', ' c', true]);
  assert.deepEqual(splitPredicateComparison('flag'), ['', '', '', false]);
});

test('a bare word with neither operator nor bracket is a truthy read', () => {
  const parsed = parseLeaf('active');
  assert.equal(parsed.kind, 'truthy');
  assert.equal(parsed.evaluate(source({ active: true })), TruthTrue);
  assert.equal(parsed.evaluate(source({ active: false })), TruthFalse);
  assert.equal(parsed.evaluate(source({ active: 0 })), TruthFalse);
  assert.equal(parsed.evaluate(source({ active: '' })), TruthFalse);
  assert.equal(parsed.evaluate(source({ active: null })), TruthFalse);
  assert.equal(parsed.evaluate(source({ active: 'no' })), TruthTrue);
  assert.equal(parsed.evaluate(source({})), TruthUnknown);
});

test('isTruthy answers a value the row could not flatten as present-and-true', () => {
  assert.equal(isTruthy([]), true);
  assert.equal(isTruthy({}), true);
  assert.equal(isTruthy(-0), false);
  assert.equal(isTruthy(0), false);
  assert.equal(isTruthy(1), true);
});

test('defined is never unknown, because absence is exactly what it asks about', () => {
  const parsed = parseLeaf('defined(total.amount)');
  assert.equal(parsed.kind, 'defined');
  assert.equal(parsed.path, 'total.amount');
  assert.equal(parsed.evaluate(facts({ total: { amount: 0 } })), TruthTrue);
  assert.equal(parsed.evaluate(facts({ total: {} })), TruthFalse);
  assert.equal(parseLeaf('defined( total.amount )').path, 'total.amount');
});

test('always and never are constants', () => {
  assert.equal(parseLeaf('always').evaluate(source({})), TruthTrue);
  assert.equal(parseLeaf('never').evaluate(source({})), TruthFalse);
  assert.equal(fromNode(true).evaluate(source({})), TruthTrue);
  assert.equal(fromNode(false).evaluate(source({})), TruthFalse);
});

// ---- membership ---------------------------------------------------------------------------

test('any_of and none_of are unknown when the fact is not published', () => {
  const included = fromNode({ status: { any_of: ['open', 'closed'] } });
  assert.equal(included.evaluate(source({ status: 'open' })), TruthTrue);
  assert.equal(included.evaluate(source({ status: 'held' })), TruthFalse);
  assert.equal(included.evaluate(source({})), TruthUnknown);

  const excluded = fromNode({ status: { none_of: ['open'] } });
  assert.equal(excluded.evaluate(source({ status: 'open' })), TruthFalse);
  assert.equal(excluded.evaluate(source({ status: 'held' })), TruthTrue);
  assert.equal(excluded.evaluate(source({})), TruthUnknown);

  assert.equal(
    raised(() => fromNode({ status: { any_of: 'open' } })),
    '`any_of` takes a list',
  );

  // Go reads the two keys from a fixed list, `any_of` first, so a constraint carrying both is
  // read the same way twice. Only the operator table below it is randomised.
  const both = fromNode({ status: { any_of: ['open'], none_of: ['open'] } });
  assert.equal(both.kind, 'any_of');
  assert.equal(both.evaluate(source({ status: 'open' })), TruthTrue);
});

// ---- quantifiers ----------------------------------------------------------------------------

test('a quantifier walks the collection with the body rebound onto each element', () => {
  const every = fromNode({ forall: { in: 'lines', as: 'line', that: 'line.sku' } });
  assert.equal(every.evaluate(facts({ lines: [{ sku: 'a' }, { sku: 'b' }] })), TruthTrue);
  assert.equal(every.evaluate(facts({ lines: [{ sku: 'a' }, { sku: '' }] })), TruthFalse);

  const some = fromNode({ exists: { in: 'lines', as: 'line', that: { 'line.sku': { eq: 'b' } } } });
  assert.equal(some.evaluate(facts({ lines: [{ sku: 'a' }, { sku: 'b' }] })), TruthTrue);
  assert.equal(some.evaluate(facts({ lines: [{ sku: 'a' }] })), TruthFalse);
});

test('the bound name reads the element itself when the element is a scalar', () => {
  const every = fromNode({ forall: { in: 'lines', as: 'item', that: 'item' } });
  assert.equal(every.evaluate(facts({ lines: ['a', 'b'] })), TruthTrue);
  assert.equal(every.evaluate(facts({ lines: ['a', ''] })), TruthFalse);
});

test('an unobserved collection is unknown and an empty one is vacuously true', () => {
  const every = fromNode({ forall: { in: 'lines', as: 'line', that: 'line.sku' } });
  const some = fromNode({ exists: { in: 'lines', as: 'line', that: 'line.sku' } });
  assert.equal(every.evaluate(source({})), TruthUnknown);
  assert.equal(some.evaluate(source({})), TruthUnknown);
  assert.equal(every.evaluate(facts({ lines: [] })), TruthTrue);
  assert.equal(some.evaluate(facts({ lines: [] })), TruthFalse);
});

test('a count that is not a whole non-negative number leaves the quantifier unknown', () => {
  const every = fromNode({ forall: { in: 'lines', as: 'line', that: 'line.sku' } });
  for (const count of [
    1.5,
    -1,
    Number.NaN,
    Number.POSITIVE_INFINITY,
    Number.NEGATIVE_INFINITY,
    2 ** 63,
    'two',
    null,
    true,
  ]) {
    assert.equal(
      every.evaluate(source({ 'lines.count': count as Node })),
      TruthUnknown,
      String(count),
    );
  }
});

test('an element the collection claims but does not publish is unknown, not false', () => {
  const every = fromNode({ forall: { in: 'lines', as: 'line', that: 'line.sku' } });
  assert.equal(every.evaluate(source({ 'lines.count': 1 })), TruthUnknown);
});

test('a decided quantifier is not overturned by a later element nobody could read', () => {
  // Go breaks out of the walk here. The break is a cost and not a semantic — `and` is
  // false-dominant and `or` is true-dominant, so an unknown element after the deciding one
  // cannot change the answer, and removing the break from either runtime moves no verdict.
  // What is checked is the answer, in both directions: decided stays decided, undecided does not.
  const every = fromNode({ forall: { in: 'lines', as: 'line', that: 'line.ok' } });
  assert.equal(every.evaluate(source({ 'lines.count': 2, 'lines.0.ok': false })), TruthFalse);
  const some = fromNode({ exists: { in: 'lines', as: 'line', that: 'line.ok' } });
  assert.equal(some.evaluate(source({ 'lines.count': 2, 'lines.0.ok': true })), TruthTrue);
  // And an undecided walk stays undecided rather than collapsing to the elements it did read.
  assert.equal(
    fromNode({ forall: { in: 'lines', as: 'line', that: 'line.ok' } }).evaluate(
      source({ 'lines.count': 2, 'lines.0.ok': true }),
    ),
    TruthUnknown,
  );
});

test('a read outside the bound name passes through the rebinding untouched', () => {
  const every = fromNode({
    forall: { in: 'lines', as: 'line', that: { all: ['line.sku', 'open'] } },
  });
  assert.equal(every.evaluate(facts({ open: true, lines: [{ sku: 'a' }] })), TruthTrue);
  assert.equal(every.evaluate(facts({ open: false, lines: [{ sku: 'a' }] })), TruthFalse);
});

// ---- composition ------------------------------------------------------------------------------

test('a list is a conjunction and a mapping with one entry is that entry', () => {
  assert.equal(fromNode(['a', 'b']).kind, 'all');
  assert.equal(fromNode({ a: 1 }).kind, 'compare');
  assert.equal(fromNode({ a: 1, b: 2 }).kind, 'all');
  assert.equal(fromNode({ any: ['a', 'b'] }).kind, 'any');
  assert.equal(fromNode({ or: ['a', 'b'] }).kind, 'any');
  assert.equal(fromNode({ and: ['a', 'b'] }).kind, 'all');
  assert.equal(fromNode({ all_of: ['a', 'b'] }).kind, 'all');
  assert.equal(fromNode({ not: 'a' }).kind, 'not');
  // A single child where a list was expected is that one child.
  assert.equal(fromNode({ any: 'a' }).children.length, 1);
});

test('a conjunction is unknown only when nothing in it is false', () => {
  const both = fromNode(['a', 'b']);
  assert.equal(both.evaluate(source({ a: true, b: true })), TruthTrue);
  assert.equal(both.evaluate(source({ a: true })), TruthUnknown);
  assert.equal(both.evaluate(source({ a: false })), TruthFalse);
  const either = fromNode({ any: ['a', 'b'] });
  assert.equal(either.evaluate(source({ a: true })), TruthTrue);
  assert.equal(either.evaluate(source({ a: false })), TruthUnknown);
  assert.equal(either.evaluate(source({ a: false, b: false })), TruthFalse);
});

test('an entry mapping parses its children in sorted key order, the same way twice', () => {
  assert.equal(String(fromNode({ b: 1, a: 2 })), '(a == 2 and b == 1)');
  assert.equal(String(fromNode({ a: 2, b: 1 })), '(a == 2 and b == 1)');
});

test('keys sort by code point, which is the order Go sorts bytes in', () => {
  // JavaScript's default sort is UTF-16 code unit order, which puts an astral key before
  // U+E000; Go's `sort.Strings` compares UTF-8 bytes, which is code point order.
  const parsed = fromNode({ '\u{10000}b': true, 'a': true });
  assert.equal(String(parsed), '(a == true and \u{10000}b == true)');
});

// ---- rendering --------------------------------------------------------------------------------

test('a predicate renders as the expression a failure report quotes', () => {
  assert.equal(String(parseLeaf('always')), 'always');
  assert.equal(String(parseLeaf('never')), 'never');
  assert.equal(String(parseLeaf('active')), 'active');
  assert.equal(String(parseLeaf('defined(total.amount)')), 'defined(total.amount)');
  assert.equal(String(parseLeaf('not active')), 'not (active)');
  assert.equal(String(parseLeaf('total.amount >= 0')), 'total.amount >= 0');
  assert.equal(String(parseLeaf('state == Bridged')), 'state == "Bridged"');
  assert.equal(String(parseLeaf('a.b == c.d')), 'a.b == c.d');
  assert.equal(String(fromNode({ any: ['a', 'b'] })), '(a or b)');
  assert.equal(
    String(fromNode({ status: { any_of: ['open', 'held'] } })),
    'status in ["open", "held"]',
  );
  assert.equal(String(fromNode({ status: { none_of: ['open'] } })), 'status not in ["open"]');
  assert.equal(
    String(fromNode({ forall: { in: 'lines', as: 'line', that: 'line.sku' } })),
    'forall line in lines: (line.sku)',
  );
});

// ---- admission --------------------------------------------------------------------------------

test('admission refuses a fact path the grammar does not admit, and says which', () => {
  assert.equal(
    raised(() => admitPredicatePath('a b')),
    'invalid predicate fact path "a b"',
  );
  assert.equal(
    raised(() => admitPredicatePath('')),
    'invalid predicate fact path ""',
  );
  assert.equal(
    raised(() => parseLeaf('a b')),
    'invalid predicate fact path "a b"',
  );
  assert.equal(
    raised(() => parseLeaf('defined(x')),
    'invalid predicate fact path "defined(x"',
  );
  assert.equal(admitPredicatePath('total.amount'), undefined);
});

test('admission names the four constants and reads every function form', () => {
  for (const constant of ['always', 'true', 'never', 'false']) {
    assert.equal(admitPredicateLeaf(constant), undefined, constant);
  }
  for (const call of ['defined(a)', 'exists(a)', 'missing(a)', 'defined ( a )']) {
    assert.equal(admitPredicateLeaf(call), undefined, call);
  }
  // Admitted, and then refused by the evaluator, which implements only `defined(`.
  assert.equal(
    raised(() => parseLeaf('exists(a)')),
    '`exists(a)` is an expression this generated runner does not implement',
  );
});

test('admission refuses an empty right operand and spends a bounded depth budget', () => {
  assert.equal(
    raised(() => admitPredicateLeaf('a ==')),
    'predicate comparison requires a right operand',
  );
  assert.equal(admitPredicateExpression(`${'not '.repeat(32)}x`), undefined);
  assert.equal(
    raised(() => admitPredicateExpression(`${'not '.repeat(33)}x`)),
    'predicate exceeds maximum depth 32',
  );
});

test('a quoted operand is closed, or the whole leaf is refused', () => {
  assert.equal(admitQuotedOperand('bare'), undefined);
  assert.equal(admitQuotedOperand(''), undefined);
  assert.equal(admitQuotedOperand('"closed"'), undefined);
  assert.equal(admitQuotedOperand(' "closed" '), undefined);
  assert.equal(
    raised(() => admitQuotedOperand('"open')),
    'quoted operand is not closed; close the quote and use structured any/all/not to combine predicates',
  );
  assert.equal(
    raised(() => admitQuotedOperand('"a" b')),
    'tokens after a quoted operand are unsupported; use structured any/all/not',
  );
});

// ---- whitespace -------------------------------------------------------------------------------

test('whitespace is Go `unicode.IsSpace`, not JavaScript `String.trim`', () => {
  // U+0085 is whitespace to Go and not to JavaScript; U+FEFF is the other way around.
  assert.equal(goTrimSpace('\u0085 to  '), 'to');
  assert.equal(goTrimSpace('﻿to'), '﻿to');
  assert.equal(parseLeaf('\u0085to == "x"').left.path, 'to');
  assert.equal(
    raised(() => parseLeaf('﻿to == "x"')),
    'invalid predicate fact path "\\ufeffto"',
  );
});

// ---- parse refusals ---------------------------------------------------------------------------

test('a predicate that is not an expression, a list or a mapping is refused by kind', () => {
  assert.equal(
    raised(() => parsePredicate(undefined)),
    'no predicate is written',
  );
  assert.equal(
    raised(() => fromNode(3)),
    'a predicate is an expression, a list or a mapping, not float64',
  );
  assert.equal(
    raised(() => fromNode(null)),
    'a predicate is an expression, a list or a mapping, not <nil>',
  );
});

test('a quantifier needs a mapping with `in`, `as` and `that`', () => {
  assert.equal(
    raised(() => fromNode({ forall: 'lines' })),
    'a quantifier is a mapping with `in`, `as` and `that`',
  );
  assert.equal(
    raised(() => fromNode({ forall: { in: 'lines' } })),
    'a quantifier needs both `in` and `as`',
  );
  assert.equal(
    raised(() => fromNode({ exists: { as: 'line', that: 'line.sku' } })),
    'a quantifier needs both `in` and `as`',
  );
  assert.equal(
    raised(() => fromNode({ forall: { in: 'lines', as: 'line' } })),
    'a predicate is an expression, a list or a mapping, not <nil>',
  );
});

test('a constraint mapping carrying no operator this runner knows is refused by name', () => {
  assert.equal(
    raised(() => fromNode({ total: { approximately: 1 } })),
    '`total` carries no operator this runner knows',
  );
  // Admitted by the suite reader and still unimplemented here, which is Go's behaviour too.
  assert.equal(
    raised(() => fromNode({ total: { truthy: true } })),
    '`total` carries no operator this runner knows',
  );
});

test('every operator spelling the runner knows maps to its symbol', () => {
  for (const [spelling, op] of [
    ['eq', '=='],
    ['equals', '=='],
    ['==', '=='],
    ['ne', '!='],
    ['not_equals', '!='],
    ['!=', '!='],
    ['lt', '<'],
    ['<', '<'],
    ['le', '<='],
    ['lte', '<='],
    ['<=', '<='],
    ['gt', '>'],
    ['>', '>'],
    ['ge', '>='],
    ['gte', '>='],
    ['>=', '>='],
  ] as [string, string][]) {
    assert.equal(fromNode({ total: { [spelling]: 1 } }).op, op, spelling);
  }
});

test('an ordering comparison of values nothing orders is unknown', () => {
  const parsed = fromNode({ total: { gt: 1 } });
  assert.equal(parsed.evaluate(source({ total: 2 })), TruthTrue);
  assert.equal(parsed.evaluate(source({ total: 'two' })), TruthUnknown);
  assert.equal(parsed.evaluate(source({ total: null })), TruthUnknown);
  assert.equal(parsed.evaluate(source({})), TruthUnknown);
  // Equality, unlike ordering, answers false rather than unknown for a present mismatch.
  assert.equal(fromNode({ total: { eq: 1 } }).evaluate(source({ total: 'two' })), TruthFalse);
  assert.equal(fromNode({ total: { ne: 1 } }).evaluate(source({ total: 'two' })), TruthTrue);
});

test('an unparsed predicate value is a leaf compared for equality', () => {
  const parsed = fromNode({ state: 'Bridged' });
  assert.equal(parsed.kind, 'compare');
  assert.equal(parsed.op, '==');
  assert.equal(parsed.right.literal, 'Bridged');
  assert.equal(parsed.evaluate(source({ state: 'Bridged' })), TruthTrue);
  assert.equal(fromNode({ open: true }).evaluate(source({ open: true })), TruthTrue);
  assert.equal(fromNode({ count: 2 }).evaluate(source({ count: 2 })), TruthTrue);
});

test('an unknown kind evaluates to unknown rather than throwing', () => {
  assert.equal(new Predicate({ kind: 'invented' }).evaluate(source({})), TruthUnknown);
  assert.equal(String(new Predicate({ kind: 'invented' })), 'invented');
});

// ---- the corpus ---------------------------------------------------------------------------

// `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`, the one document Rust,
// the Go runtime and the browser adapter all answer. Its `admission` section is answered by the
// runtime's `primitive`, not here; its `numbers` and `orderings` sections are number semantics,
// and the part of them a predicate evaluator decides is checked below.
const corpus = ((): {
  numbers: { name: string; from: Record<string, unknown>; display: string }[];
  orderings: { name: string; left: string; right: string; ordering: string }[];
  text_orderings: { name: string; left: string; right: string; ordering: string }[];
} => {
  const relative = 'crates/specify/ess-primitives/tests/vectors/primitive-semantics.json';
  let directory = import.meta.dirname;
  for (let depth = 0; depth < 12; depth += 1) {
    const candidate = join(directory, relative);
    if (existsSync(candidate)) return JSON.parse(readFileSync(candidate, 'utf8'));
    directory = dirname(directory);
  }
  throw new Error(`the primitive corpus is not at ${relative} above ${import.meta.dirname}`);
})();

// The spelling a predicate would carry: the authored decimal where there is one, because JSON
// parsing has already collapsed an integer token above 2^53 by the time it reaches here.
const spelling = (name: string): string => {
  const vector = corpus.numbers.find((candidate) => candidate.name === name);
  if (vector === undefined) throw new Error(`the corpus names no number ${name}`);
  return typeof vector.from.decimal === 'string' ? vector.from.decimal : vector.display;
};

test('every number the corpus states is read by the evaluator as that number', () => {
  assert.ok(corpus.numbers.length >= 14, 'the corpus is not a token corpus');
  for (const vector of corpus.numbers) {
    const text = spelling(vector.name);
    assert.deepEqual(parseDecimalLiteral(text), [Number(text), true], vector.name);
    const parsed = parseLeaf(`amount == ${text}`);
    assert.equal(parsed.right.isFact, false, vector.name);
    assert.equal(parsed.right.literal, Number(text), vector.name);
    assert.equal(parsed.evaluate(source({ amount: Number(text) })), TruthTrue, vector.name);
  }
});

test('every ordering the corpus states is the ordering the evaluator answers', () => {
  // One exception, and it is the same exception in Go: the evaluator reads every literal as
  // binary64, so the two integers either side of 2^53 are one value to it. `Number` keeps them
  // apart; a predicate literal cannot. Both runtimes answer `equal` here, together.
  const collapsed = new Set(['two-to-53 is below two-to-53-plus-one']);
  assert.ok(corpus.orderings.length >= 7, 'the corpus states fewer orderings than it did');
  for (const ordering of corpus.orderings) {
    const left = source({ amount: Number(spelling(ordering.left)) });
    const right = spelling(ordering.right);
    const less = ordering.ordering === 'less' && !collapsed.has(ordering.name);
    assert.equal(parseLeaf(`amount < ${right}`).evaluate(left), truthOf(less), ordering.name);
    assert.equal(parseLeaf(`amount == ${right}`).evaluate(left), truthOf(!less), ordering.name);
  }
});

// Text is ordered by its UTF-8 bytes in every lane (ess#94): `B` is below `a`, which a locale
// order reverses, and U+FFFF is below U+10000, which UTF-16 code units reverse.
test('every text ordering the corpus states is the byte ordering the evaluator answers', () => {
  assert.ok(corpus.text_orderings.length >= 8, 'the corpus states fewer text orderings');
  for (const ordering of corpus.text_orderings) {
    const left = source({ caller: ordering.left });
    const less = ordering.ordering === 'less';
    const greater = ordering.ordering === 'greater';
    for (const [op, holds] of [
      ['<', less],
      ['<=', !greater],
      ['>', greater],
      ['>=', !less],
    ] as const) {
      const expression = `caller ${op} ${JSON.stringify(ordering.right)}`;
      assert.equal(
        parseLeaf(expression).evaluate(left),
        truthOf(holds),
        `${ordering.name}: ${JSON.stringify(ordering.left)} ${expression}`,
      );
    }
  }
});

// ---- text lengths ------------------------------------------------------------------------------

// `.count` on text (beyond10x/ess#104): a bound fact wins; otherwise, when the last segment is
// `count` and the parent is bound to text, the text's number of code points. The corpus path comes
// from `ESS_PRIMITIVE_VECTORS`, which `tests/typescript_runtime.rs` sets, and this case fails rather
// than skipping without it: a lane that selects nothing is indistinguishable from a green one.
test('every text length the corpus states is the count the evaluator reads', () => {
  const at = process.env.ESS_PRIMITIVE_VECTORS;
  assert.ok(at, 'ESS_PRIMITIVE_VECTORS names the primitive corpus');
  const vectors = JSON.parse(readFileSync(at, 'utf8')).text_lengths as {
    name: string;
    facts: Record<string, Node>;
    path: string;
    expected: number | string;
  }[];
  assert.ok(vectors.length >= 8, `the corpus states ${vectors.length} text lengths`);
  const equals = (path: string, value: number): Predicate =>
    new Predicate({
      kind: 'compare',
      left: new Operand({ path, isFact: true }),
      op: '==',
      right: new Operand({ literal: value }),
    });
  for (const vector of vectors) {
    const facts = source(vector.facts);
    if (typeof vector.expected === 'number') {
      assert.equal(equals(vector.path, vector.expected).evaluate(facts), TruthTrue, vector.name);
      assert.equal(
        equals(vector.path, vector.expected + 1).evaluate(facts),
        TruthFalse,
        vector.name,
      );
    } else {
      assert.equal(vector.expected, 'unknown', vector.name);
      assert.equal(equals(vector.path, 0).evaluate(facts), TruthUnknown, vector.name);
    }
  }
});

test('a text reads as if its count were bound, in every leaf kind, and a quantifier over it stays unknown', () => {
  const bare = source({ keys: 'abc' });
  const bound = source({ keys: 'abc', 'keys.count': 3 });
  // One per leaf kind; the two membership leaves have no expression spelling, so they are built.
  const leaves = [
    parseLeaf('keys.count == 3'),
    parseLeaf('defined(keys.count)'),
    parseLeaf('keys.count'),
    new Predicate({ kind: 'any_of', path: 'keys.count', values: [3, 4] }),
    new Predicate({ kind: 'none_of', path: 'keys.count', values: [1, 2] }),
  ];
  for (const leaf of leaves) {
    assert.equal(leaf.evaluate(bare), leaf.evaluate(bound), String(leaf));
    assert.notEqual(leaf.evaluate(bare), TruthUnknown, String(leaf));
  }
  const quantified = new Predicate({
    kind: 'forall',
    over: 'keys',
    bind: 'k',
    body: parseLeaf('k == x'),
  });
  assert.equal(quantified.evaluate(source({ keys: '' })), TruthUnknown);
});
