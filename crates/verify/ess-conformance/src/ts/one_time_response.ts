// Closed disclosure authority and private, scenario-local observation for suite versions34/35.
import {
  accessorType,
  admitOutcome,
  array,
  closed,
  equal,
  exactDecimal,
  isObject,
  JsonNumber,
  name,
  text,
  unsigned,
  sortStrings,
  render,
  spellDecimal,
} from './runtime.js';
import type { Node, OutcomeRef } from './runtime.js';
import {
  admitDirectResponse,
  admitDirectResponseField,
  nativeJSONBytes,
  compareDirectResponse,
} from './direct_response.js';
import type { DirectResponse } from './direct_response.js';
import {
  Operand,
  Predicate,
  parseLeaf,
  parseLiteral,
  parseOperand,
  parseDecimalLiteral,
  splitPredicateComparison,
  facts,
} from './predicate.js';

export interface StringConstraints {
  alphabet: string | null;
  prefix: string | null;
  invariants: Predicate[];
}
export interface OneTimeResponse {
  shape: DirectResponse;
  constraints: Record<string, StringConstraints>;
  authority: Node;
}
export interface OneTimeOrigin {
  command: string;
  outcome: OutcomeRef;
  response: OneTimeResponse;
  fields: string[];
}
export interface OneTimeTrace {
  origins: OneTimeOrigin[];
  required_origins: OutcomeRef[];
  events: { event: string; within_ms: bigint }[];
  event_windows: { event: string; after_step: number; within_ms: bigint }[];
  authority: Node;
}
const fail = (): never => {
  throw new Error('invalid one-time response authority');
};
const sequence = (value: Node): Node[] =>
  value == null ? [] : Array.isArray(value) ? value : [value];
function combine(kind: 'all' | 'any', children: Predicate[]): Predicate {
  if (children.length === 0) return new Predicate({ kind: kind === 'all' ? 'always' : 'never' });
  return children.length === 1 ? children[0]! : new Predicate({ kind, children });
}
function negate(body: Predicate): Predicate {
  if (body.kind === 'always') return new Predicate({ kind: 'never' });
  if (body.kind === 'never') return new Predicate({ kind: 'always' });
  if (body.kind === 'not') return body.body!;
  return new Predicate({ kind: 'not', body });
}
const comparisons: Record<string, string> = {
  eq: '==',
  equals: '==',
  '==': '==',
  ne: '!=',
  not_equals: '!=',
  '!=': '!=',
  lt: '<',
  '<': '<',
  le: '<=',
  lte: '<=',
  '<=': '<=',
  gt: '>',
  '>': '>',
  ge: '>=',
  gte: '>=',
  '>=': '>=',
};
function numericLiteral(raw: string): Node {
  const binary = Number(raw);
  if (!Number.isFinite(binary)) return fail();
  if (Object.is(binary, -0)) return -0;
  const exact = exactDecimal(new JsonNumber(raw));
  if (exact != null && exact[1] === 0 && exact[0] >= -(1n << 127n) && exact[0] < 1n << 127n)
    return new JsonNumber(spellDecimal(exact));
  return binary;
}
function literal(value: Node): Node {
  if (typeof value === 'string') {
    if (parseDecimalLiteral(value)[1]) return numericLiteral(value);
    return parseLiteral(value);
  }
  if (value instanceof JsonNumber) return numericLiteral(value.raw);
  if (typeof value === 'boolean') return value;
  return fail();
}
function typedOperand(raw: string): Operand {
  const parsed = parseOperand(raw);
  if (!parsed.isFact && parseDecimalLiteral(raw.trim())[1])
    parsed.literal = numericLiteral(raw.trim());
  return parsed;
}
function constraint(path: string, raw: Node): Predicate {
  const left = new Operand({ path, isFact: true });
  if (Array.isArray(raw)) return new Predicate({ kind: 'any_of', path, values: raw.map(literal) });
  if (!isObject(raw))
    return new Predicate({
      kind: 'compare',
      left,
      op: '==',
      right: new Operand({ literal: literal(raw) }),
    });
  return combine(
    'all',
    sortStrings(Object.keys(raw)).map((op) => {
      const value = raw[op];
      if (Object.hasOwn(comparisons, op))
        return new Predicate({
          kind: 'compare',
          left,
          op: comparisons[op]!,
          right:
            typeof value === 'string'
              ? typedOperand(value)
              : new Operand({ literal: literal(value) }),
        });
      if (['any_of', 'in', 'one_of', 'none_of', 'not_in'].includes(op))
        return new Predicate({
          kind: ['none_of', 'not_in'].includes(op) ? 'none_of' : 'any_of',
          path,
          values: sequence(value).map(literal),
        });
      if (op === 'defined' || op === 'exists') {
        if (typeof value !== 'boolean') return fail();
        const defined = new Predicate({ kind: 'defined', path });
        return value ? defined : negate(defined);
      }
      if (op === 'truthy') return new Predicate({ kind: 'truthy', path });
      if (
        ['starts_with', 'ends_with', 'contains', 'equals_ignore_case', 'in_ignore_case'].includes(
          op,
        )
      )
        return new Predicate({
          kind: op,
          path,
          values: op === 'in_ignore_case' ? array(value) : [value],
        });
      return fail();
    }),
  );
}

// A separate closed String environment uses the shared grammar, including noncanonical aliases.
function predicate(raw: Node, depth = 0): Predicate {
  if (depth > 32) return fail();
  if (typeof raw === 'boolean') return new Predicate({ kind: raw ? 'always' : 'never' });
  if (typeof raw === 'string') {
    const value = raw.trim();
    if (value.startsWith('not ')) return negate(predicate(value.slice(4), depth + 1));
    if (value === 'true' || value === 'false') return predicate(value === 'true', depth);
    const defined = /^(defined|exists|missing)\s*\((.*)\)$/u.exec(value);
    if (defined !== null) {
      const parsed = parseLeaf(`defined(${defined[2]!.trim()})`);
      return defined[1] === 'missing' ? negate(parsed) : parsed;
    }
    const parsed = parseLeaf(value);
    if (parsed.kind === 'compare') {
      const right = splitPredicateComparison(value)[2].trim();
      if (['null', 'Null', 'NULL', '~'].includes(right)) return fail();
      if (
        !right.startsWith('"') &&
        !right.startsWith("'") &&
        right.split(/\s+/u).some((word) => word === '&&' || word === '||')
      )
        return fail();
      parsed.right = typedOperand(right);
    }
    return parsed;
  }
  if (Array.isArray(raw))
    return combine(
      'all',
      raw.map((child) => predicate(child, depth + 1)),
    );
  if (!isObject(raw)) return fail();
  return combine(
    'all',
    sortStrings(Object.keys(raw)).map((key) => {
      const value = raw[key];
      if (['all', 'and', 'all_of', 'any', 'or', 'none', 'none_of_these'].includes(key)) {
        const result = combine(
          ['all', 'and', 'all_of'].includes(key) ? 'all' : 'any',
          sequence(value).map((child) => predicate(child, depth + 1)),
        );
        return ['none', 'none_of_these'].includes(key) ? negate(result) : result;
      }
      if (key === 'not') return negate(predicate(value, depth + 1));
      // The only reachable newtype environment is String; neither value nor its count is a collection.
      if (key === 'forall' || key === 'exists') return fail();
      return constraint(key, value);
    }),
  );
}
function operandKind(operand: Operand): string {
  if (operand.isFact) {
    if (operand.path === 'value') return 'text';
    if (operand.path === 'value.count') return 'number';
    return fail();
  }
  if (typeof operand.literal === 'string') return 'text';
  if (typeof operand.literal === 'boolean') return 'bool';
  if (operand.literal instanceof JsonNumber || typeof operand.literal === 'number') return 'number';
  return fail();
}
function validatePredicate(p: Predicate): void {
  const kind = (path: string) => operandKind(new Operand({ path, isFact: true }));
  switch (p.kind) {
    case 'always':
    case 'never':
      return;
    case 'all':
    case 'any':
      for (const child of p.children) validatePredicate(child);
      return;
    case 'not':
      if (p.body == null) return fail();
      validatePredicate(p.body);
      return;
    case 'truthy':
    case 'defined':
      kind(p.path);
      return;
    case 'compare': {
      const left = operandKind(p.left),
        right = operandKind(p.right);
      if (left !== right || (left === 'bool' && !['==', '!='].includes(p.op))) return fail();
      if (
        (p.left.isFact && p.right.literal === 'value') ||
        (p.right.isFact && p.left.literal === 'value')
      )
        return fail();
      return;
    }
    case 'any_of':
    case 'none_of': {
      const expected = kind(p.path);
      for (const value of p.values)
        if (operandKind(new Operand({ literal: value })) !== expected) return fail();
      return;
    }
    case 'starts_with':
    case 'ends_with':
    case 'contains':
    case 'equals_ignore_case':
    case 'in_ignore_case':
      if (kind(p.path) !== 'text') return fail();
      if (p.values.length === 0) return fail();
      for (const value of p.values) {
        if (typeof value !== 'string' || value === 'value') return fail();
        if (['starts_with', 'ends_with', 'contains'].includes(p.kind) && value.length === 0)
          return fail();
      }
      return;
    default:
      return fail();
  }
}
function literalWire(value: Node): Node {
  if (!(value instanceof JsonNumber) && typeof value !== 'number') return value;
  const binary = Number(value instanceof JsonNumber ? value.raw : value);
  const exact = exactDecimal(value);
  if (exact != null && exact[1] === 0) {
    const carried =
      Math.abs(binary) < 9223372036854775808 && Number.isInteger(binary)
        ? BigInt(binary)
        : exactDecimal(binary)?.[0];
    if (carried !== exact[0]) return new JsonNumber(exact[0].toString());
  }
  const [mantissa, exponentText] = binary.toExponential().split('e');
  const exponent = Number(exponentText);
  if (exponent >= -5 && exponent <= 15) {
    let fixed = render(binary);
    if (!fixed.includes('.')) fixed += '.0';
    return new JsonNumber(fixed);
  }
  return new JsonNumber(`${mantissa}e${exponent < 0 ? '-' : '+'}${Math.abs(exponent)}`);
}
function predicateWire(p: Predicate): Node {
  switch (p.kind) {
    case 'always':
      return true;
    case 'never':
      return false;
    case 'all':
    case 'any':
      return { [p.kind]: p.children.map(predicateWire) };
    case 'not':
      return { not: predicateWire(p.body!) };
    case 'any_of':
    case 'none_of':
    case 'in_ignore_case':
      return { [p.path]: { [p.kind]: p.values.map(literalWire) } };
    case 'starts_with':
    case 'ends_with':
    case 'contains':
    case 'equals_ignore_case':
      return { [p.path]: { [p.kind]: p.values[0] } };
    case 'compare': {
      const operandText = (operand: Operand): string => {
        if (operand.isFact) return operand.path;
        const value = operand.literal;
        if (Object.is(value, -0)) return '0';
        if (typeof value !== 'string')
          return value instanceof JsonNumber ? spellDecimal(exactDecimal(value)!) : render(value);
        if (
          value === '' ||
          value.includes('.') ||
          ['null', 'Null', 'NULL', '~'].includes(value) ||
          value.split(/\s+/u).some((word) => word === '&&' || word === '||')
        )
          return JSON.stringify(value);
        return value;
      };
      const compact = `${operandText(p.left)} ${p.op} ${operandText(p.right)}`;
      if (p.left.isFact && !p.right.isFact && typeof p.right.literal === 'string') {
        let roundtrips = false;
        try {
          roundtrips = equal(parseLeaf(compact), p);
        } catch {
          /* Use the lossless structured spelling. */
        }
        if (!roundtrips) {
          const value = p.right.literal;
          const operand = parseOperand(value);
          const scalar = operand.isFact || operand.literal !== value ? `"${value}"` : value;
          return { [p.left.path]: { [p.op]: scalar } };
        }
      }
      return compact;
    }
    default:
      return p.toString();
  }
}
function canonicalField(raw: Node): Node {
  const result: Node = { name: raw.name, type: accessorType(text(raw.type), 0)[0] };
  const naming = raw.naming ?? raw;
  for (const key of ['wire', 'display', 'summary', 'code'])
    if (naming[key] != null) result[key] = naming[key];
  if (raw.presence != null) result.presence = raw.presence;
  return result;
}
function isString(type: string, shape: DirectResponse): boolean {
  const seen = new Set<string>();
  while (type !== 'String') {
    if (seen.has(type)) return false;
    seen.add(type);
    const body = shape.declarations[type];
    if (body?.kind !== 'newtype') return false;
    type = body.of.trim();
  }
  return true;
}
function response(raw: Node, command: string): OneTimeResponse {
  const value = closed(raw, 'fields declarations constraints', '');
  const normalizeField = (rawField: Node): Node => {
    admitDirectResponseField(rawField);
    return canonicalField(rawField);
  };
  if (!isObject(value.declarations)) return fail();
  const normalizedDeclarations: Node = Object.create(null);
  for (const [label, rawBody] of Object.entries(value.declarations) as [string, Node][]) {
    if (!isObject(rawBody)) return fail();
    const body = { ...rawBody };
    if (body.kind === 'newtype') body.of = accessorType(text(body.of), 0)[0];
    if (body.kind === 'struct') body.fields = array(body.fields).map(normalizeField);
    if (body.kind === 'union') {
      if (!isObject(body.variants)) return fail();
      body.variants = Object.fromEntries(
        Object.entries(body.variants).map(([variant, type]) => [
          variant,
          accessorType(text(type), 0)[0],
        ]),
      );
    }
    normalizedDeclarations[label] = body;
  }
  const shape = admitDirectResponse({
    command,
    fields: array(value.fields).map(normalizeField),
    declarations: normalizedDeclarations,
    expected: {},
  });
  if (!isObject(value.constraints)) return fail();
  const constraints: Record<string, StringConstraints> = Object.create(null);
  const constraintWire: Node = Object.create(null);
  for (const [label, rawRules] of Object.entries(value.constraints)) {
    name(label, false);
    if (shape.declarations[label]?.kind !== 'newtype' || !isString(label, shape)) return fail();
    const rules = closed(rawRules, 'invariants', 'alphabet prefix');
    const alphabet = rules.alphabet == null ? null : text(rules.alphabet);
    const prefix = rules.prefix == null ? null : text(rules.prefix);
    if (
      alphabet != null &&
      (alphabet.length === 0 || new Set([...alphabet]).size !== [...alphabet].length)
    )
      return fail();
    if (prefix === '') return fail();
    const invariants = array(rules.invariants).map((rawPredicate) => {
      const parsed = predicate(rawPredicate);
      validatePredicate(parsed);
      return parsed;
    });
    constraints[label] = { alphabet, prefix, invariants };
    constraintWire[label] = { alphabet, prefix, invariants: invariants.map(predicateWire) };
  }
  for (const label of Object.keys(constraints)) {
    const alphabets: string[] = [],
      prefixes: string[] = [];
    let type = label;
    while (shape.declarations[type]?.kind === 'newtype') {
      const rules = constraints[type];
      if (rules?.alphabet != null) alphabets.push(rules.alphabet);
      if (rules?.prefix != null) prefixes.push(rules.prefix);
      type = shape.declarations[type]!.of;
    }
    for (const a of alphabets)
      for (const b of alphabets) if (![...a].some((char) => b.includes(char))) return fail();
    for (const a of prefixes)
      for (const b of prefixes) if (!a.startsWith(b) && !b.startsWith(a)) return fail();
    for (const prefix of prefixes)
      for (const alphabet of alphabets)
        if ([...prefix].some((char) => !alphabet.includes(char))) return fail();
  }
  const declarations: Node = Object.create(null);
  for (const [label, rawBody] of Object.entries(normalizedDeclarations) as [string, Node][]) {
    const body = { ...rawBody };
    if (body.kind === 'struct') body.fields = body.fields.map(canonicalField);
    declarations[label] = body;
  }
  return {
    shape,
    constraints,
    authority: {
      fields: value.fields.map(canonicalField),
      declarations,
      constraints: constraintWire,
    },
  };
}

export function admitOneTimeTrace(raw: Node, scenario: Node): OneTimeTrace {
  const value = closed(raw, 'origins required_origins events event_windows', '');
  const steps = array(scenario.steps),
    source = array(scenario.source);
  const rawOrigins = array(value.origins),
    rawWindows = array(value.event_windows);
  if (rawOrigins.length === 0 || rawOrigins.length > 256 || rawWindows.length > 65536)
    return fail();
  const reference = (kind: string, ref: Node) =>
    source.some((item) => item.kind === kind && equal(item.name, ref));
  const origins: OneTimeOrigin[] = [];
  const seen = new Set<string>(),
    schemas = new Map<string, Node>();
  const identity = (outcome: OutcomeRef) => `${outcome.command}/${outcome.outcome}`;
  for (const rawOrigin of rawOrigins) {
    const origin = closed(rawOrigin, 'command outcome response fields', '');
    name(origin.command, false);
    admitOutcome(origin.outcome);
    const id = identity(origin.outcome);
    if (origin.outcome.command !== origin.command || seen.has(id)) return fail();
    seen.add(id);
    if (
      !steps.some(
        (step) =>
          ['execute_command', 'execute_command_without_input'].includes(step.step) &&
          step.command === origin.command,
      )
    )
      return fail();
    if (!reference('command', origin.command) || !reference('outcome', origin.outcome))
      return fail();
    const declared = response(origin.response, origin.command);
    const prior = schemas.get(origin.command);
    if (prior !== undefined && !equal(prior, declared.authority)) return fail();
    schemas.set(origin.command, declared.authority);
    const fields = array(origin.fields).map(text);
    if (fields.length === 0 || fields.length > 256 || new Set(fields).size !== fields.length)
      return fail();
    for (const field of fields) {
      const declaredField = declared.shape.fields.find((f) => f.name === field);
      if (declaredField === undefined || !isString(declaredField.type, declared.shape))
        return fail();
    }
    origins.push({ command: origin.command, outcome: origin.outcome, response: declared, fields });
  }
  const required_origins: OutcomeRef[] = array(value.required_origins);
  const required = new Set<string>();
  if (required_origins.length === 0) return fail();
  for (const origin of required_origins) {
    admitOutcome(origin);
    const id = identity(origin);
    if (!seen.has(id) || required.has(id)) return fail();
    required.add(id);
  }
  const events: OneTimeTrace['events'] = [];
  const eventNames = new Set<string>();
  for (const rawEvent of array(value.events)) {
    const event = closed(rawEvent, 'event within_ms', '');
    name(event.event, false);
    if (eventNames.has(event.event) || !reference('event', event.event)) return fail();
    eventNames.add(event.event);
    events.push({ event: event.event, within_ms: unsigned(event.within_ms) });
  }
  const operations = new Set([
    'execute_command',
    'execute_command_without_input',
    'query_view',
    'eventually_view',
  ]);
  const event_windows: OneTimeTrace['event_windows'] = [];
  const windows = new Set<string>();
  const windowKey = (event: string, step: number, within: bigint) => `${event}/${step}/${within}`;
  for (const rawWindow of rawWindows) {
    const window = closed(rawWindow, 'event after_step within_ms', '');
    name(window.event, false);
    const position = unsigned(window.after_step),
      within = unsigned(window.within_ms);
    if (
      position >= BigInt(steps.length) ||
      !eventNames.has(window.event) ||
      !operations.has(steps[Number(position)].step)
    )
      return fail();
    const key = windowKey(window.event, Number(position), within);
    if (windows.has(key)) return fail();
    windows.add(key);
    event_windows.push({ event: window.event, after_step: Number(position), within_ms: within });
  }
  for (const [index, step] of steps.entries())
    if (operations.has(step.step))
      for (const event of events)
        if (
          !windows.has(windowKey(event.event, index, 0n)) ||
          !windows.has(windowKey(event.event, index, event.within_ms))
        )
          return fail();
  const authority = {
    ...value,
    origins: origins.map((origin) => ({ ...origin, response: origin.response.authority })),
  };
  if (nativeJSONBytes(authority) > 1048576) return fail();
  return { origins, required_origins, events, event_windows, authority };
}

export interface DisclosureCell {
  origin: OutcomeRef;
  field: string;
  aspect: string[];
  actor: string | null;
}

export function parseDisclosureId(id: string): DisclosureCell {
  const parts = id.split('/');
  const [command, family, outcome, field] = parts;
  if (family !== 'disclosure' || !/^_*[A-Za-z][A-Za-z0-9_]*$/.test(field ?? '')) return fail();
  const origin = { command: command!, outcome: outcome! };
  admitOutcome(origin);
  let actor: string | null;
  let aspect: string[];
  if (parts.at(-2) === 'as' && parts.at(-1) === 'anonymous') {
    actor = null;
    aspect = parts.slice(4, -2);
  } else if (parts.at(-3) === 'as' && parts.at(-2) === 'actor') {
    actor = parts.at(-1)!;
    name(actor, false);
    aspect = parts.slice(4, -3);
  } else return fail();
  if (aspect.length === 1 && ['origin', 'retry', 'rotation'].includes(aspect[0]!)) {
    // These three cells have no additional source reference.
  } else if (aspect.length === 2 && ['read', 'denied'].includes(aspect[0]!)) {
    name(aspect[1], false);
  } else if (aspect.length === 3 && aspect[0] === 'command') {
    admitOutcome({ command: aspect[1], outcome: aspect[2] });
  } else return fail();
  return { origin, field: field!, aspect, actor };
}

export function admitDisclosureId(
  id: string,
  scenario: Node,
  trace: OneTimeTrace,
  version: string,
): DisclosureCell {
  if (!['ess-conformance/34', 'ess-conformance/35'].includes(version)) return fail();
  const cell = parseDisclosureId(id);
  if (
    !trace.required_origins.some((origin) => equal(origin, cell.origin)) ||
    !trace.origins.some(
      (origin) => equal(origin.outcome, cell.origin) && origin.fields.includes(cell.field),
    )
  )
    return fail();
  const steps = array(scenario.steps);
  const invocation = (step: Node): boolean =>
    ['execute_command', 'execute_command_without_input'].includes(step.step);
  const selected = (
    steps: Node[],
    expected: OutcomeRef,
    actor?: string | null,
  ): [number, number] | null => {
    let last: number | null = null;
    for (const [index, step] of steps.entries()) {
      if (invocation(step)) last = index;
      if (last === null) continue;
      const invoked = steps[last];
      if (
        invoked.command === expected.command &&
        (actor === undefined || (invoked.actor ?? null) === actor) &&
        step.step === 'expect_outcome' &&
        equal(step.outcome, expected)
      )
        return [last, index];
    }
    return null;
  };
  const origin = selected(steps, cell.origin);
  if (origin === null) return fail();
  const invoked = steps[origin[0]],
    tail = steps.slice(origin[1] + 1);
  const sameActor = (invoked.actor ?? null) === cell.actor;
  let fulfilled = false;
  switch (cell.aspect[0]) {
    case 'origin':
      fulfilled = sameActor;
      break;
    case 'retry':
      fulfilled =
        sameActor &&
        tail.some(
          (step) =>
            invocation(step) &&
            step.step === invoked.step &&
            step.command === invoked.command &&
            (step.actor ?? null) === (invoked.actor ?? null) &&
            equal(step.input ?? {}, invoked.input ?? {}) &&
            equal(step.caller ?? {}, invoked.caller ?? {}),
        );
      break;
    case 'rotation':
      fulfilled = selected(tail, cell.origin, cell.actor) !== null;
      break;
    case 'read':
      fulfilled =
        sameActor &&
        tail.some(
          (step) =>
            ['query_view', 'eventually_view'].includes(step.step) && step.view === cell.aspect[1],
        );
      break;
    case 'command':
      fulfilled =
        selected(tail, { command: cell.aspect[1]!, outcome: cell.aspect[2]! }, cell.actor) !== null;
      break;
    case 'denied': {
      let last: Node = null;
      fulfilled =
        cell.actor !== null &&
        tail.some((step) => {
          if (invocation(step)) last = step;
          return (
            step.step === 'expect_not_granted' &&
            step.actor === cell.actor &&
            last !== null &&
            last.command === cell.aspect[1] &&
            (last.actor ?? null) === cell.actor
          );
        });
      break;
    }
  }
  if (!fulfilled) return fail();
  return cell;
}

export function refusePrivateExploration(ir: Node): void {
  if (
    isObject(ir.commands) &&
    Object.values(ir.commands).some(
      (command: Node) =>
        Array.isArray(command.outcomes) &&
        command.outcomes.some(
          (outcome: Node) =>
            Array.isArray(outcome.one_time_response) && outcome.one_time_response.length > 0,
        ),
    )
  )
    throw new Error('UnsupportedOneTimeDisclosure');
}

/** A closed, value-free failure: captured observations never become messages. */
export class DisclosureViolation extends Error {
  readonly kind: 'disclosure' | 'payload' | 'resource';
  constructor(kind: 'disclosure' | 'payload' | 'resource') {
    super(
      kind === 'resource'
        ? 'ESS-CF-TARGET'
        : kind === 'payload'
          ? 'ESS-CF-PAYLOAD'
          : 'ESS-CF-DISCLOSURE',
    );
    this.kind = kind;
  }
}

/** Private per-scenario state. It has no serialization or diagnostic operation. */
export class DisclosureCaptures {
  #values: string[] = [];
  #bytes = 0;
  #observed = new Set<string>();
  readonly policy: OneTimeTrace;
  constructor(policy: OneTimeTrace) {
    this.policy = policy;
  }
  private text(text: string, values = this.#values): void {
    if (values.some((value) => text.includes(value))) throw new DisclosureViolation('disclosure');
  }
  private scan(value: Node, values: string[], depth: number, budget: { members: number }): void {
    if (depth > 128 || budget.members > 65536) throw new DisclosureViolation('resource');
    if (typeof value === 'string') this.text(value, values);
    else if (Array.isArray(value))
      for (const child of value) {
        budget.members += 1;
        this.scan(child, values, depth + 1, budget);
      }
    else if (isObject(value) && !(value instanceof JsonNumber)) {
      for (const [key, child] of Object.entries(value)) {
        if (++budget.members > 65536) throw new DisclosureViolation('resource');
        this.text(key, values);
        this.scan(child, values, depth + 1, budget);
      }
    }
  }
  observe(value: Node, values = this.#values): void {
    // Resource refusal precedes substring comparison, including over-budget leaked values.
    this.scan(value, [], 0, { members: 0 });
    let bytes = 0;
    const add = (count: number) => {
      bytes += count;
      if (bytes > 1048576) throw new DisclosureViolation('resource');
    };
    const string = (text: string) => {
      add(2);
      for (const character of text) {
        const code = character.codePointAt(0)!;
        if (code >= 0xd800 && code <= 0xdfff) throw new DisclosureViolation('resource');
        add(
          code === 34 || code === 92 || [8, 9, 10, 12, 13].includes(code)
            ? 2
            : code < 32
              ? 6
              : code < 128
                ? 1
                : code < 2048
                  ? 2
                  : code < 65536
                    ? 3
                    : 4,
        );
      }
    };
    const write = (node: Node): void => {
      if (typeof node === 'string') string(node);
      else if (node instanceof JsonNumber || typeof node === 'number') {
        const normalized = literalWire(
          node instanceof JsonNumber ? numericLiteral(node.raw) : node,
        );
        add(normalized instanceof JsonNumber ? normalized.raw.length : String(normalized).length);
      } else if (Array.isArray(node)) {
        add(2 + Math.max(0, node.length - 1));
        for (const child of node) write(child);
      } else if (isObject(node)) {
        const entries = Object.entries(node);
        add(2 + Math.max(0, entries.length - 1));
        for (const [key, child] of entries) {
          string(key);
          add(1);
          write(child);
        }
      } else if (node === null) add(4);
      else if (typeof node === 'boolean') add(node ? 4 : 5);
      else throw new DisclosureViolation('resource');
    };
    write(value);
    this.scan(value, values, 0, { members: 0 });
  }
  maps(rows: Record<string, Node>[]): void {
    this.observe(rows);
  }
  private constraints(authority: OneTimeResponse, written: string, value: Node): void {
    const type = accessorType(written, 0)[0];
    if (type.startsWith('Optional<')) {
      if (value != null) this.constraints(authority, type.slice(9, -1), value);
      return;
    }
    if (type.startsWith('List<')) {
      for (const child of array(value)) this.constraints(authority, type.slice(5, -1), child);
      return;
    }
    if (type.startsWith('Map<')) {
      for (const child of Object.values(value))
        this.constraints(authority, type.slice(type.indexOf(',') + 1, -1).trim(), child);
      return;
    }
    const rules = authority.constraints[type];
    if (rules !== undefined) {
      if (
        typeof value !== 'string' ||
        (rules.alphabet !== null &&
          [...value].some((character) => !rules.alphabet!.includes(character))) ||
        (rules.prefix !== null && !value.startsWith(rules.prefix))
      )
        throw new DisclosureViolation('payload');
      for (const predicate of rules.invariants) {
        const result = predicate.evaluate(facts({ value }));
        if (result !== 'true')
          throw new DisclosureViolation(result === 'unknown' ? 'resource' : 'payload');
      }
    }
    const declaration = authority.shape.declarations[type];
    if (declaration?.kind === 'newtype') this.constraints(authority, declaration.of, value);
    else if (declaration?.kind === 'struct') {
      for (const field of declaration.fields)
        if (Object.hasOwn(value, field.name))
          this.constraints(authority, field.type, value[field.name]);
    } else if (declaration?.kind === 'union') {
      const key = declaration.tag === 'value' ? 'content' : 'value';
      if (Object.hasOwn(value, key))
        this.constraints(authority, declaration.variants[value[declaration.tag]], value[key]);
    }
  }
  command(command: string, result: Node): void {
    if (result.response != null) this.observe(result.response);
    const origin = this.policy.origins.find(
      (origin) => origin.command === command && origin.outcome.outcome === result.outcome,
    );
    if (origin !== undefined) {
      if (result.error) throw new DisclosureViolation('payload');
      try {
        compareDirectResponse(origin.response.shape, result.response);
      } catch {
        throw new DisclosureViolation('payload');
      }
      for (const field of origin.response.shape.fields)
        if (Object.hasOwn(result.response, field.name))
          this.constraints(origin.response, field.type, result.response[field.name]);
      const added: string[] = [];
      for (const field of origin.fields) {
        const value = result.response[field];
        if (typeof value !== 'string' || value.length === 0)
          throw new DisclosureViolation('payload');
        if (added.some((other) => other.includes(value) || value.includes(other)))
          throw new DisclosureViolation('disclosure');
        added.push(value);
      }
      const bytes = added.reduce((sum, value) => sum + new TextEncoder().encode(value).length, 0);
      if (this.#values.length + added.length > 256 || this.#bytes + bytes > 1048576)
        throw new DisclosureViolation('resource');
      for (const [key, value] of Object.entries(result.response)) {
        this.text(key, added);
        if (!origin.fields.includes(key)) this.scan(value, added, 0, { members: 0 });
      }
      this.#values.push(...added);
      this.#bytes += bytes;
      this.#observed.add(`${origin.command}/${origin.outcome.outcome}`);
    }
    this.maps((result.directEvents ?? []).map((event: Node) => event.payload));
    if (result.errorPayload != null) this.observe(result.errorPayload);
  }
  complete(): boolean {
    return this.policy.required_origins.every((origin) =>
      this.#observed.has(`${origin.command}/${origin.outcome}`),
    );
  }
}
