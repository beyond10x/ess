// Actual command responses, compared under immutable typed suite authority.
//
// Ported from `src/go/response.go`. Two properties decide whether this file is right, and both are
// about *exactness*:
//
//   - A returned Integer is never rounded through a binary64. `9007199254740993` and
//     `9007199254740992` are one value to `JSON.parse` and two to the specification, so every
//     number this file compares is compared as the digits it was written with.
//   - What the command returned and what the event carried are compared field by field against the
//     contract the suite admitted, and a target cannot widen that contract by returning more.

import {
  JsonNumber,
  SelectionObservation,
  accessorCollection,
  accessorOptional,
  accessorPrimitive,
  admitAccessorField,
  admitOutcome,
  array,
  canonicalUUID,
  closed,
  equal,
  errorText,
  goMarshal,
  name,
  paddedBase64,
} from './runtime.js';
import type {
  AccessorField,
  ByteCounter,
  CommandResult,
  Node,
  OutcomeRef,
  ScenarioRun,
  SelectionDeclaration,
  Step,
} from './runtime.js';

/** One command's declared response, and the event field each returned member must equal. */
export interface ResponseObservation {
  command: string;
  outcome: OutcomeRef;
  event: string;
  fields: AccessorField[];
  declarations: Record<string, SelectionDeclaration>;
  /** Event field name to the response field it must equal. */
  mappings: Record<string, string>;
  targets: AccessorField[];
}

const BYTE_LIMIT = 1048576;
const DEPTH_LIMIT = 128;
const ENCODER = new TextEncoder();

/** `Object.hasOwn`, because a response field may be named `toString` and Go's map lookup is not. */
function owned<T>(record: Record<string, T> | null | undefined, key: string): T | undefined {
  return record != null && Object.hasOwn(record, key) ? record[key] : undefined;
}

function present<T>(record: Record<string, T> | null | undefined, key: string): boolean {
  return record != null && Object.hasOwn(record, key);
}

function goBytes(value: Node): number {
  return ENCODER.encode(goMarshal(value)).length;
}

// ---- reading the document ---------------------------------------------------------------------

function decodedString(value: unknown, what: string): string {
  if (value === undefined) {
    return '';
  }
  if (typeof value !== 'string') {
    throw new Error(`${what} must be a string`);
  }
  return value;
}

function decodeAccessorField(value: unknown): AccessorField {
  const field = closed(value, '', 'name type');
  return {
    name: decodedString(field.name, 'a field name'),
    type: decodedString(field.type, 'a field type'),
  };
}

function decodeDeclaration(value: unknown): SelectionDeclaration {
  const body = closed(value, '', 'kind of fields variants tag');
  return {
    kind: decodedString(body.kind, 'a declaration kind'),
    of: decodedString(body.of, 'a declaration source'),
    fields:
      body.fields === undefined || body.fields === null
        ? []
        : array(body.fields).map(decodeAccessorField),
    // `json.RawMessage` left nil by an absent key, which every reader of it treats as no document.
    variants: body.variants === undefined ? null : body.variants,
    tag: decodedString(body.tag, 'a declaration tag'),
  };
}

/**
 * Read the response observation the suite carries, refusing an unknown field anywhere in it.
 *
 * Go decodes with `DisallowUnknownFields` and then validates, so an authored document that says
 * something this runtime does not read is refused rather than half-applied. The refusal is the
 * same for a `wire:` alias on a response field, which the accessor admitter permits and the
 * response contract does not.
 */
export function decodeResponseObservation(value: unknown): ResponseObservation {
  const document = closed(value, '', 'command outcome event fields declarations mappings targets');
  const outcomeRaw =
    document.outcome === undefined ? {} : closed(document.outcome, '', 'command outcome');
  const declarationsRaw = document.declarations;
  if (declarationsRaw !== undefined && declarationsRaw !== null) {
    if (typeof declarationsRaw !== 'object' || Array.isArray(declarationsRaw)) {
      throw new Error('response declarations must be an object');
    }
  }
  const mappingsRaw = document.mappings;
  if (mappingsRaw !== undefined && mappingsRaw !== null) {
    if (typeof mappingsRaw !== 'object' || Array.isArray(mappingsRaw)) {
      throw new Error('response mappings must be an object');
    }
  }
  const mappings: Record<string, string> = {};
  for (const [key, mapped] of Object.entries((mappingsRaw ?? {}) as Record<string, unknown>)) {
    if (typeof mapped !== 'string') {
      throw new Error('a response mapping must be a string');
    }
    mappings[key] = mapped;
  }
  const declarations: Record<string, SelectionDeclaration> = {};
  for (const [key, body] of Object.entries((declarationsRaw ?? {}) as Record<string, unknown>)) {
    declarations[key] = decodeDeclaration(body);
  }
  const observation: ResponseObservation = {
    command: decodedString(document.command, 'a command'),
    outcome: {
      command: decodedString(outcomeRaw.command, 'an outcome command'),
      outcome: decodedString(outcomeRaw.outcome, 'an outcome'),
    },
    event: decodedString(document.event, 'an event'),
    fields:
      document.fields === undefined || document.fields === null
        ? []
        : array(document.fields).map(decodeAccessorField),
    declarations,
    mappings,
    targets:
      document.targets === undefined || document.targets === null
        ? []
        : array(document.targets).map(decodeAccessorField),
  };
  validateResponseObservation(observation);
  return observation;
}

/** What Go marshals when it measures the contract: every field of every struct, zero or not. */
function marshalShape(observation: ResponseObservation): Node {
  const fields = (list: AccessorField[]): Node =>
    list.map((field) => ({ name: field.name, type: field.type }));
  const declarations: Record<string, Node> = {};
  for (const [key, body] of Object.entries(observation.declarations)) {
    declarations[key] = {
      kind: body.kind ?? '',
      of: body.of ?? '',
      fields: body.fields === undefined || body.fields.length === 0 ? null : fields(body.fields),
      variants: body.variants ?? null,
      tag: body.tag ?? '',
    };
  }
  return {
    command: observation.command,
    outcome: { command: observation.outcome.command, outcome: observation.outcome.outcome },
    event: observation.event,
    fields: fields(observation.fields),
    declarations,
    mappings: observation.mappings,
    targets: fields(observation.targets),
  };
}

/** Check the contract's bounds, its owner, every type it names, and every mapping it declares. */
export function validateResponseObservation(observation: ResponseObservation): void {
  const declarationCount = Object.keys(observation.declarations).length;
  const mappingCount = Object.keys(observation.mappings).length;
  if (
    observation.command !== observation.outcome.command ||
    observation.outcome.outcome === '' ||
    observation.fields.length === 0 ||
    observation.fields.length > 256 ||
    observation.targets.length > 256 ||
    declarationCount > 4096 ||
    mappingCount === 0 ||
    mappingCount !== observation.targets.length
  ) {
    throw new Error('invalid response contract bounds or owner');
  }
  for (const label of [observation.command, observation.event]) {
    name(label, false);
  }
  const used = new Set<string>();
  const roots = new Map<string, string>();
  for (const list of [observation.fields, observation.targets]) {
    const seen = new Set<string>();
    for (const field of list) {
      if (field.name === '' || seen.has(field.name)) {
        throw new Error('duplicate/empty response field');
      }
      seen.add(field.name);
      checkType(observation, field.type, used, new Set<string>(), 0);
    }
  }
  for (const field of observation.fields) {
    roots.set(field.name, field.type);
  }
  if (used.size !== declarationCount) {
    throw new Error('unrelated response declarations');
  }
  for (const target of observation.targets) {
    const source = owned(observation.mappings, target.name);
    const from = source === undefined ? undefined : roots.get(source);
    if (source === undefined || from === undefined || !responseAssignable(from, target.type)) {
      throw new Error('invalid response mapping');
    }
  }
  if (goBytes(marshalShape(observation)) > BYTE_LIMIT) {
    throw new Error('response contract byte limit');
  }
}

/** A response field's type holds a target's type, widening only into `Optional`. */
export function responseAssignable(from: string, to: string): boolean {
  if (from === to) {
    return true;
  }
  const [inner, optional] = accessorOptional(to);
  if (optional) {
    return responseAssignable(from, inner);
  }
  return false;
}

function checkType(
  observation: ResponseObservation,
  source: string,
  used: Set<string>,
  stack: Set<string>,
  depth: number,
): void {
  if (depth > DEPTH_LIMIT) {
    throw new Error('response type depth limit');
  }
  const [inner, optional] = accessorOptional(source);
  if (optional) {
    checkType(observation, inner, used, stack, depth + 1);
    return;
  }
  if (source.startsWith('Map<') && !source.startsWith('Map<String, ')) {
    throw new Error('response map key must be String');
  }
  const [item, collection] = accessorCollection(source);
  if (collection) {
    checkType(observation, item, used, stack, depth + 1);
    return;
  }
  if (accessorPrimitive(source) && source !== 'Binary64') {
    return;
  }
  const body = owned(observation.declarations, source);
  if (body === undefined || stack.has(source)) {
    throw new Error('missing/recursive response type');
  }
  if (used.has(source)) {
    return;
  }
  used.add(source);
  stack.add(source);
  try {
    const children: string[] = [];
    switch (body.kind) {
      case 'newtype':
        children.push(body.of ?? '');
        break;
      case 'struct': {
        const seen = new Set<string>();
        for (const field of body.fields ?? []) {
          if (field.name === '' || seen.has(field.name)) {
            throw new Error('duplicate/empty response member');
          }
          seen.add(field.name);
          children.push(field.type);
        }
        break;
      }
      case 'enum': {
        const variants = body.variants;
        if (
          !Array.isArray(variants) ||
          variants.length === 0 ||
          !variants.every((label) => typeof label === 'string')
        ) {
          throw new Error('invalid response enum');
        }
        const seen = new Set<string>();
        for (const label of variants as string[]) {
          if (label === '' || seen.has(label)) {
            throw new Error('duplicate response enum label');
          }
          seen.add(label);
        }
        break;
      }
      case 'union': {
        const variants = body.variants;
        if (
          (body.tag ?? '') === '' ||
          typeof variants !== 'object' ||
          variants === null ||
          Array.isArray(variants) ||
          Object.keys(variants).length === 0 ||
          !Object.values(variants).every((child) => typeof child === 'string')
        ) {
          throw new Error('invalid response union');
        }
        children.push(...(Object.values(variants) as string[]));
        break;
      }
      default:
        throw new Error('unknown response declaration');
    }
    for (const child of children) {
      checkType(observation, child, used, stack, depth + 1);
    }
  } finally {
    stack.delete(source);
  }
}

/** The response-mode value check, which is unit A's and reaches back into this file's grammar. */
function responseObserver(
  declarations: Record<string, SelectionDeclaration>,
): SelectionObservation {
  const observer = new SelectionObservation();
  observer.declarations = declarations;
  observer.responseMode = true;
  return observer;
}

/**
 * Compare what the command actually returned against what the event it emitted carried.
 *
 * The contract is validated again here rather than trusted: it travelled through a target callback
 * on the way, and a suite that checked the shape once and asserted against a later one would be
 * asserting against whatever survived.
 */
export function compareResponse(
  observation: ResponseObservation,
  response: Record<string, Node> | null | undefined,
  payload: Record<string, Node>,
): void {
  validateResponseObservation(observation);
  if (response === null || response === undefined) {
    throw new Error('command returned no response');
  }
  const names = new Set(observation.fields.map((field) => field.name));
  for (const field of Object.keys(response)) {
    if (!names.has(field)) {
      throw new Error('response has an undeclared field');
    }
  }
  const observer = responseObserver(observation.declarations);
  const counter: ByteCounter = { bytes: 0 };
  for (const field of observation.fields) {
    try {
      observer.validateValue(
        field.type,
        owned(response, field.name),
        present(response, field.name),
        counter,
        0,
      );
    } catch (error) {
      throw new Error(`response field ${field.name}: ${errorText(error)}`);
    }
  }
  if (counter.bytes > BYTE_LIMIT) {
    throw new Error('response byte limit');
  }
  for (const [target, source] of Object.entries(observation.mappings)) {
    const actual = owned(response, source);
    const exists = present(response, source);
    const emitted = owned(payload, target);
    const emittedPresent = present(payload, target);
    let optional = false;
    for (const field of observation.fields) {
      if (field.name === source) {
        optional = accessorOptional(field.type)[1];
      }
    }
    if (
      optional &&
      (!exists || actual === null || actual === undefined) &&
      (!emittedPresent || emitted === null || emitted === undefined)
    ) {
      continue;
    }
    if (!exists || !emittedPresent || !responseEqual(actual, emitted)) {
      throw new Error(`event field ${target} differs from actual response field ${source}`);
    }
  }
}

/**
 * What `expectResponsePayload` reads of the run it is handed.
 *
 * Named from unit A's own `ScenarioRun`, so a member that moves there is a type error here.
 */
export type ResponseRun = Pick<ScenarioRun, 'lastCommand' | 'last' | 'fail'>;

/** The `expect_response_payload` step: the same invocation returned it and emitted it. */
export function expectResponsePayload(run: ResponseRun, index: number, step: Step): boolean {
  const contract = step.response;
  if (contract === null || contract === undefined) {
    return run.fail(index, 'missing response observation');
  }
  if (run.lastCommand !== contract.command || run.last.outcome !== contract.outcome.outcome) {
    return run.fail(index, 'response observation names a different command/outcome');
  }
  for (const event of run.last.directEvents) {
    if (event.event === contract.event) {
      try {
        compareResponse(contract, run.last.response, event.payload);
      } catch (error) {
        return run.fail(index, errorText(error));
      }
      return true;
    }
  }
  return run.fail(index, 'same invocation did not emit the response-mapped event');
}

/** Admit one authored response observation. */
export function admitResponse(value: unknown): void {
  const root = closed(value, 'command outcome event fields declarations mappings targets', '');
  admitOutcome(root.outcome);
  const declarations = root.declarations;
  if (typeof declarations !== 'object' || declarations === null || Array.isArray(declarations)) {
    throw new Error('response declarations must be object');
  }
  for (const raw of Object.values(declarations as Record<string, unknown>)) {
    if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
      throw new Error('invalid response declaration');
    }
    const body = raw as Record<string, unknown>;
    let required = 'kind';
    switch (body.kind) {
      case 'newtype':
        required += ' of';
        break;
      case 'struct':
        required += ' fields';
        break;
      case 'enum':
        required += ' variants';
        break;
      case 'union':
        required += ' tag variants';
        break;
      default:
        throw new Error('invalid response declaration kind');
    }
    closed(raw, required, '');
    if (Object.hasOwn(body, 'fields')) {
      for (const field of array(body.fields)) {
        admitAccessorField(field);
      }
    }
  }
  for (const key of ['fields', 'targets']) {
    for (const field of array(root[key])) {
      admitAccessorField(field);
    }
  }
  decodeResponseObservation(value);
}

// ---- snapshotting a returned result ---------------------------------------------------------------

/** The Go field names of `CommandResult`, which is what its byte bound is counted over. */
function commandResultShape(result: CommandResult): Node {
  return {
    Response: result.response ?? null,
    Outcome: result.outcome ?? '',
    Error: result.error ?? '',
    Consistency: result.consistency ?? '',
    DirectEvents:
      result.directEvents === undefined || result.directEvents === null
        ? null
        : result.directEvents.map((event) => ({
            Event: event.event,
            Payload: event.payload ?? null,
          })),
  };
}

interface EncodedResult {
  Response: Record<string, Node> | null;
  Outcome: string;
  Error: string;
  Consistency: string;
  DirectEvents: { Event: string; Payload: Record<string, Node> | null }[] | null;
}

/**
 * Snapshot a response-bearing result before any later target callback can reach its maps.
 *
 * The round trip is the copy: the result is written as the bytes Go would write and read back with
 * every number kept as its token, which is what `decoder.UseNumber()` does on the Go side. A value
 * that cannot be written — a NaN, a function — is not a typed wire value and is refused here
 * rather than compared later as whatever it degraded into.
 */
export function snapshotResponseResult(result: CommandResult): CommandResult {
  let raw: string;
  try {
    raw = goMarshal(commandResultShape(result));
  } catch {
    throw new Error('response result is not a typed wire value');
  }
  if (ENCODER.encode(raw).length > BYTE_LIMIT) {
    throw new Error('response result byte limit');
  }
  const decoded = JSON.parse(raw, function reviveTokens(
    _key: string,
    value: unknown,
    context?: { source?: string },
  ): unknown {
    if (typeof value === 'number' && typeof context?.source === 'string') {
      return new JsonNumber(context.source);
    }
    return value;
  } as (key: string, value: unknown) => unknown) as EncodedResult;
  return {
    response: decoded.Response,
    outcome: decoded.Outcome,
    error: decoded.Error,
    consistency: decoded.Consistency,
    directEvents: (decoded.DirectEvents ?? []).map((event) => ({
      event: event.Event,
      payload: event.Payload ?? {},
    })),
  } as CommandResult;
}

// ---- exact numbers -----------------------------------------------------------------------------

/** A number token as an exact value: `units * 10 ** scale`, with no binary64 anywhere in it. */
export interface ExactDecimal {
  readonly units: bigint;
  readonly scale: number;
}

const NUMBER_TOKEN = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?$/;

/**
 * An exact bounded numeric observation; never round a returned Integer through binary64.
 *
 * Bounded twice, as Go bounds it: 512 characters of token, and an exponent within ±512. A token
 * with no finite binary64 image is refused — `1e309` is not a number the specification can carry —
 * while one that merely underflows is not, because `1e-400` is a value and zero is a different one.
 */
export function responseNumber(value: Node): ExactDecimal | null {
  if (!(value instanceof JsonNumber)) {
    return null;
  }
  const raw = value.toString();
  if (raw.length > 512 || !NUMBER_TOKEN.test(raw)) {
    return null;
  }
  let digits = raw;
  let scale = 0;
  const marker = digits.search(/[eE]/);
  if (marker >= 0) {
    const exponent = Number(digits.slice(marker + 1));
    if (!Number.isInteger(exponent) || exponent < -512 || exponent > 512) {
      return null;
    }
    scale = exponent;
    digits = digits.slice(0, marker);
  }
  const parsed = Number(raw);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  const point = digits.indexOf('.');
  if (point >= 0) {
    scale -= digits.length - point - 1;
    digits = digits.slice(0, point) + digits.slice(point + 1);
  }
  return { units: BigInt(digits), scale };
}

function power(exponent: number): bigint {
  return 10n ** BigInt(exponent);
}

/** Order two exact decimals, the way `big.Rat.Cmp` orders the rationals they denote. */
export function compareExact(left: ExactDecimal, right: ExactDecimal): number {
  const shift = left.scale - right.scale;
  const a = shift >= 0 ? left.units * power(shift) : left.units;
  const b = shift >= 0 ? right.units : right.units * power(-shift);
  if (a < b) {
    return -1;
  }
  return a > b ? 1 : 0;
}

/** Whether an exact decimal denotes an integer, and that integer when it does. */
function exactInteger(value: ExactDecimal): bigint | null {
  if (value.scale >= 0) {
    return value.units * power(value.scale);
  }
  const divisor = power(-value.scale);
  return value.units % divisor === 0n ? value.units / divisor : null;
}

const INT64_MIN = -(2n ** 63n);
const INT64_MAX = 2n ** 63n - 1n;

/** Whether a returned value is of the declared primitive kind, as a grammar and not a shape. */
export function responsePrimitiveAdmits(source: string, value: Node): boolean {
  switch (source) {
    case 'Integer': {
      const number = responseNumber(value);
      if (number === null) {
        return false;
      }
      const integer = exactInteger(number);
      return integer !== null && integer >= INT64_MIN && integer <= INT64_MAX;
    }
    case 'Decimal':
      return responseNumber(value) !== null;
    case 'String':
    case 'Timestamp':
    case 'Duration':
      return typeof value === 'string';
    case 'Boolean':
      return typeof value === 'boolean';
    case 'Uuid':
      return typeof value === 'string' && canonicalUUID(value);
    case 'Bytes':
      return typeof value === 'string' && paddedBase64(value);
    default:
      return false;
  }
}

/** Structural comparison in which two number tokens are equal when their exact values are. */
export function responseEqual(left: Node, right: Node): boolean {
  if (left instanceof JsonNumber) {
    const a = responseNumber(left);
    const b = responseNumber(right);
    return a !== null && b !== null && compareExact(a, b) === 0;
  }
  if (Array.isArray(left)) {
    if (!Array.isArray(right) || left.length !== right.length) {
      return false;
    }
    return left.every((child, index) => responseEqual(child, right[index]));
  }
  if (typeof left === 'object' && left !== null) {
    if (typeof right !== 'object' || right === null || Array.isArray(right)) {
      return false;
    }
    const mine = left as Record<string, Node>;
    const other = right as Record<string, Node>;
    if (Object.keys(mine).length !== Object.keys(other).length) {
      return false;
    }
    return Object.keys(mine).every(
      (key) => Object.hasOwn(other, key) && responseEqual(mine[key], other[key]),
    );
  }
  return equal(left, right);
}
