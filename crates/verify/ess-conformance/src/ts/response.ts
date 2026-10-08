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
  isObject,
  name,
  paddedBase64,
  strictResponseJSON,
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
import { checkResponseConstraints } from './direct_response.js';
import type { StringConstraints } from './one_time_response.js';

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
  /**
   * Response field name to its presence policy (suite/24, beyond10x/ess#139): `null_when_absent`
   * or `omitted_when_absent`. A field with none admits both spellings of an absent value.
   */
  presence?: Record<string, string>;
  nested?: NestedResponseTargets;
  /** String-newtype rules of suite/46 and /47 (beyond10x/ess#499), checked on every actual value. */
  constraints?: Record<string, StringConstraints>;
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

export function decodeDeclaration(value: unknown): SelectionDeclaration {
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
  const document = closed(
    value,
    '',
    'command outcome event fields declarations mappings targets nested constraints',
  );
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
  const mappings: Record<string, string> = Object.hasOwn(document, 'nested')
    ? Object.create(null)
    : {};
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
    fields: [],
    declarations,
    mappings,
    targets:
      document.targets === undefined || document.targets === null
        ? []
        : array(document.targets).map(decodeAccessorField),
  };
  // A response field alone may carry a presence policy; it is read beside the field, not into it,
  // so the contract the byte bound measures stays the one Go measures.
  const presence: Record<string, string> = Object.hasOwn(document, 'nested')
    ? Object.create(null)
    : {};
  for (const raw of document.fields === undefined || document.fields === null
    ? []
    : array(document.fields)) {
    const policy = isObject(raw) ? raw.presence : undefined;
    const field = decodeAccessorField(
      policy === undefined
        ? raw
        : Object.fromEntries(Object.entries(raw).filter(([k]) => k !== 'presence')),
    );
    if (policy !== undefined) {
      if (policy !== 'null_when_absent' && policy !== 'omitted_when_absent') {
        throw new Error('presence must be null_when_absent or omitted_when_absent');
      }
      presence[field.name] = policy;
    }
    observation.fields.push(field);
  }
  if (Object.keys(presence).length > 0) {
    observation.presence = presence;
  }
  if (Object.hasOwn(document, 'nested')) {
    observation.nested = decodeNestedResponse(document.nested);
    normalizeNestedResponse(observation);
  }
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
    (mappingCount === 0 && observation.nested === undefined) ||
    mappingCount + (observation.nested?.mappings.length ?? 0) > 256 ||
    mappingCount !== observation.targets.length
  ) {
    throw new Error('invalid response contract bounds or owner');
  }
  for (const label of [observation.command, observation.event]) {
    name(label, false);
  }
  const roots = new Map<string, string>();
  validateTypedFields([observation.fields, observation.targets], observation.declarations);
  for (const field of observation.fields) {
    roots.set(field.name, field.type);
  }
  for (const target of observation.targets) {
    const source = owned(observation.mappings, target.name);
    const from = source === undefined ? undefined : roots.get(source);
    if (source === undefined || from === undefined || !responseAssignable(from, target.type)) {
      throw new Error('invalid response mapping');
    }
  }
  if (observation.nested !== undefined) {
    validateNestedResponse(observation, observation.nested);
    if (ENCODER.encode(nestedResponseCanonical(observation)).length > BYTE_LIMIT) {
      throw new Error('response contract byte limit');
    }
  } else if (goBytes(marshalShape(observation)) > BYTE_LIMIT) {
    throw new Error('response contract byte limit');
  }
}

// ---- retained results (suite/12, `replays:`) --------------------------------------------------------

/**
 * A retained-result observation: the original command's successful outcome, the retained branch
 * of the same command, the subject both are about, and the complete response schema their results
 * are compared under. `replay::Observation` in Rust.
 */
export interface RetainedCapture {
  snapshot: string;
  origin: OutcomeRef;
  replay: OutcomeRef;
  instance: string;
  /** Where the original subject's identity is read: an input field, or one direct event's field. */
  identity: { kind: 'input'; field: string } | { kind: 'event'; event: string; field: string };
  fields: AccessorField[];
  declarations: Record<string, SelectionDeclaration>;
  /** The capture as written, for the exact comparison of the original and the replay's authority. */
  written: string;
}

/** A retained original result, and what the original was sent with. */
export interface RetainedResult {
  capture: RetainedCapture;
  response: Record<string, Node>;
  identity: Node;
  input: Record<string, Node> | undefined;
  actor: string;
}

/** Read and validate a retained-result observation, refusing an unknown key anywhere in it. */
export function decodeRetainedCapture(value: unknown): RetainedCapture {
  const root = closed(value, 'snapshot origin replay instance identity fields declarations', '');
  const outcome = (raw: unknown): OutcomeRef => {
    const held = closed(raw, 'command outcome', '');
    return {
      command: decodedString(held.command, 'a command'),
      outcome: decodedString(held.outcome, 'an outcome'),
    };
  };
  const identityRaw = root.identity;
  let identity: RetainedCapture['identity'];
  if (isObject(identityRaw) && identityRaw.kind === 'input') {
    const held = closed(identityRaw, 'kind field', '');
    identity = { kind: 'input', field: decodedString(held.field, 'an identity field') };
  } else if (isObject(identityRaw) && identityRaw.kind === 'event') {
    const held = closed(identityRaw, 'kind event field', '');
    identity = {
      kind: 'event',
      event: decodedString(held.event, 'an identity event'),
      field: decodedString(held.field, 'an identity field'),
    };
  } else {
    throw new Error('retained identity must be an input or an event field');
  }
  if (!isObject(root.declarations)) {
    throw new Error('retained declarations must be an object');
  }
  const declarations: Record<string, SelectionDeclaration> = {};
  for (const [key, body] of Object.entries(root.declarations)) {
    declarations[key] = decodeDeclaration(body);
  }
  const capture: RetainedCapture = {
    snapshot: decodedString(root.snapshot, 'a snapshot name'),
    origin: outcome(root.origin),
    replay: outcome(root.replay),
    instance: decodedString(root.instance, 'an instance name'),
    identity,
    fields: array(root.fields).map(decodeAccessorField),
    declarations,
    written: goMarshal(value),
  };
  if (
    capture.origin.command !== capture.replay.command ||
    capture.origin.outcome === capture.replay.outcome
  ) {
    throw new Error('replay must name a distinct origin in the same command');
  }
  if (capture.fields.length === 0 || capture.fields.length > 256) {
    throw new Error('replay schema resource limit');
  }
  validateTypedFields([capture.fields], declarations);
  const floating = (type: string): boolean => /(^|[<,\s])(Decimal|Binary64)([>,\s]|$)/.test(type);
  if (
    capture.fields.some((field) => floating(field.type)) ||
    Object.values(declarations).some(
      (declared) => floating(declared.of) || declared.fields.some((field) => floating(field.type)),
    )
  ) {
    throw new Error('Decimal and Binary64 replay results are unsupported');
  }
  return capture;
}

/** Admit one retained-result observation: its grammar, then the checks decoding makes. */
export function admitRetainedCapture(value: unknown): void {
  const root = closed(value, 'snapshot origin replay instance identity fields declarations', '');
  admitOutcome(root.origin);
  admitOutcome(root.replay);
  name(root.snapshot, true);
  name(root.instance, true);
  for (const field of array(root.fields)) {
    admitAccessorField(field);
  }
  admitTypedDeclarations(root.declarations);
  decodeRetainedCapture(value);
}

/** Every value of an actual response, and no undeclared member, under the capture's schema. */
function admitRetainedResponse(
  capture: RetainedCapture,
  response: Record<string, Node> | null | undefined,
): Record<string, Node> {
  if (response === null || response === undefined) {
    throw new Error('command returned no response');
  }
  const names = new Set(capture.fields.map((field) => field.name));
  if (Object.keys(response).some((key) => !names.has(key))) {
    throw new Error('undeclared response field');
  }
  const observer = responseObserver(capture.declarations);
  const counter: ByteCounter = { bytes: 0 };
  for (const field of capture.fields) {
    observer.validateValue(
      field.type,
      owned(response, field.name),
      present(response, field.name),
      counter,
      0,
    );
  }
  if (counter.bytes > BYTE_LIMIT) {
    throw new Error('replay response byte limit');
  }
  return response;
}

/** What the retained-result steps read of the run they are handed. */
export type RetainedRun = Pick<
  ScenarioRun,
  'lastCommand' | 'last' | 'lastInput' | 'lastActor' | 'instances' | 'retained' | 'fail'
>;

/**
 * `capture_command_result` (original) and `expect_replay_result`: the original's exact typed result
 * is retained, and a retry of the same request by the same actor about the same subject returns
 * exactly it, with no error and no new facts — the Rust runner's `retained_result`.
 */
export function retainedResult(
  run: RetainedRun,
  index: number,
  capture: RetainedCapture | undefined,
  original: boolean,
): boolean {
  if (capture === undefined) {
    return run.fail(index, 'missing retained result observation');
  }
  const refuse = (reason: string): boolean =>
    run.fail(
      index,
      `retained result ${capture.snapshot}: exact retained original result, without error or ` +
        `new facts; ${reason}`,
    );
  if (run.lastCommand === '') {
    return refuse('no preceding command');
  }
  const expected = original ? capture.origin : capture.replay;
  if (run.lastCommand !== expected.command || run.last.outcome !== expected.outcome) {
    return refuse('retained result names a different command or outcome');
  }
  if (run.last.error !== '') {
    return refuse('retained result returned an error');
  }
  if (!Object.prototype.hasOwnProperty.call(run.instances, capture.instance)) {
    return refuse('original subject has no observed identity');
  }
  const identity = run.instances[capture.instance];
  try {
    if (original) {
      let read: Node;
      if (capture.identity.kind === 'input') {
        if (!present(run.lastInput, capture.identity.field)) {
          throw new Error('original input identity is missing');
        }
        read = owned(run.lastInput, capture.identity.field);
      } else {
        const event = capture.identity.event;
        const matching = run.last.directEvents.filter((held) => held.event === event);
        if (matching.length === 0) {
          throw new Error('original identity event is missing');
        }
        if (matching.length > 1) {
          throw new Error('original identity event is ambiguous');
        }
        if (!present(matching[0]!.payload, capture.identity.field)) {
          throw new Error('original event identity is missing');
        }
        read = owned(matching[0]!.payload, capture.identity.field);
      }
      if (!equal(read, identity)) {
        throw new Error('captured subject differs from the actual original identity');
      }
      const response = admitRetainedResponse(capture, run.last.response);
      if (run.retained.has(capture.snapshot)) {
        throw new Error('original result snapshot already exists');
      }
      run.retained.set(capture.snapshot, {
        capture,
        response,
        identity,
        input: run.lastInput,
        actor: run.lastActor,
      });
      return true;
    }
    const saved = run.retained.get(capture.snapshot);
    if (saved === undefined) {
      throw new Error('original result snapshot is missing');
    }
    if (
      saved.capture.written !== capture.written ||
      !equal(saved.identity, identity) ||
      goMarshal(saved.input ?? null) !== goMarshal(run.lastInput ?? null) ||
      saved.actor !== run.lastActor
    ) {
      throw new Error('replay substituted the original authority, subject, input, or actor');
    }
    if (run.last.directEvents.length > 0) {
      throw new Error('replay emitted new facts');
    }
    admitRetainedResponse(capture, saved.response);
    const replayed = admitRetainedResponse(capture, run.last.response);
    if (!exactlyEqual(saved.response, replayed)) {
      throw new Error('retry result differs from retained original response');
    }
    return true;
  } catch (error) {
    return refuse(errorText(error));
  }
}

/** Structural equality with every number compared as the digits it was written with. */
function exactlyEqual(left: Node, right: Node): boolean {
  if (left instanceof JsonNumber || right instanceof JsonNumber) {
    return left instanceof JsonNumber && right instanceof JsonNumber && left.raw === right.raw;
  }
  if (Array.isArray(left)) {
    return (
      Array.isArray(right) &&
      left.length === right.length &&
      left.every((item, position) => exactlyEqual(item, right[position]))
    );
  }
  if (isObject(left)) {
    if (!isObject(right)) {
      return false;
    }
    const keys = Object.keys(left);
    return (
      keys.length === Object.keys(right).length &&
      keys.every((key) => Object.hasOwn(right, key) && exactlyEqual(left[key], right[key]))
    );
  }
  return left === right || (left === null && right === null);
}

// ---- complete subject rows (suite/12 vocabulary, synthesized into every later suite) -------------

/**
 * The complete projected row of one subject: its finite typed schema, never expected data. What a
 * `snapshot_complete_subject` step carries, and `subject::SubjectShape` in Rust.
 */
export interface SubjectShape {
  /** The required, non-optional projected identity field selecting the subject. */
  identityField: string;
  /** Every declared field of the projection. */
  fields: AccessorField[];
  declarations: Record<string, SelectionDeclaration>;
}

/** Read and validate a subject shape, refusing an unknown key anywhere in it. */
export function decodeSubjectShape(value: unknown): SubjectShape {
  const root = closed(value, 'identity_field fields declarations', '');
  const declarationsRaw = root.declarations;
  if (!isObject(declarationsRaw)) {
    throw new Error('subject declarations must be an object');
  }
  const declarations: Record<string, SelectionDeclaration> = {};
  for (const [key, body] of Object.entries(declarationsRaw)) {
    declarations[key] = decodeDeclaration(body);
  }
  const shape: SubjectShape = {
    identityField: decodedString(root.identity_field, 'an identity field'),
    fields: array(root.fields).map(decodeAccessorField),
    declarations,
  };
  validateSubjectShape(shape);
  return shape;
}

/** Admit one authored subject shape: the same checks, with the grammar of each field and type. */
export function admitSubjectShape(value: unknown): void {
  const root = closed(value, 'identity_field fields declarations', '');
  for (const field of array(root.fields)) {
    admitAccessorField(field);
  }
  admitTypedDeclarations(root.declarations);
  decodeSubjectShape(value);
}

function validateSubjectShape(shape: SubjectShape): void {
  validateTypedFields([shape.fields], shape.declarations);
  const identity = shape.fields.find((field) => field.name === shape.identityField);
  if (identity === undefined) {
    throw new Error('complete subject identity is not a declared field');
  }
  const optional = (type: string, depth: number): boolean => {
    if (accessorOptional(type)[1]) {
      return true;
    }
    const declared = owned(shape.declarations, type);
    return depth < 128 && declared?.kind === 'newtype' && optional(declared.of, depth + 1);
  };
  if (optional(identity.type, 0)) {
    throw new Error('complete subject identity cannot be optional');
  }
}

/**
 * Require every declared value of an actual row, each of its declared type. Extra row keys stay
 * part of the exact comparison that follows; they are not refused here.
 */
export function admitSubjectRow(shape: SubjectShape, row: Record<string, Node>): void {
  const observer = responseObserver(shape.declarations);
  const counter: ByteCounter = { bytes: 0 };
  for (const field of shape.fields) {
    observer.validateValue(
      field.type,
      owned(row, field.name),
      present(row, field.name),
      counter,
      0,
    );
  }
  if (counter.bytes > BYTE_LIMIT || goBytes(row) > BYTE_LIMIT) {
    throw new Error('complete subject row byte limit');
  }
}

/** Admit a closed finite type graph for independently owned values. */
export function validateTypedFields(
  groups: AccessorField[][],
  declarations: Record<string, SelectionDeclaration>,
  allowJson = false,
): void {
  const used = new Set<string>();
  for (const list of groups) {
    const seen = new Set<string>();
    for (const field of list) {
      if (field.name === '' || seen.has(field.name)) {
        throw new Error('duplicate/empty response field');
      }
      seen.add(field.name);
      checkType({ declarations }, field.type, used, new Set<string>(), 0, allowJson);
    }
  }
  if (used.size !== Object.keys(declarations).length) {
    throw new Error('unrelated response declarations');
  }
}

/**
 * Admit a fixture contract's closed type graph. A type that reaches itself only behind Optional,
 * List or Map has finite values, which a provider may supply and the value depth guard bounds; one
 * with no such boundary has none (beyond10x/ess#416).
 */
export function validateFixtureTypes(
  fields: AccessorField[],
  declarations: Record<string, SelectionDeclaration>,
): void {
  const used = new Set<string>();
  const seen = new Set<string>();
  for (const field of fields) {
    if (field.name === '' || seen.has(field.name)) {
      throw new Error('duplicate/empty fixture input field');
    }
    seen.add(field.name);
    checkType({ declarations }, field.type, used, new Set<string>(), 0, false, true);
  }
  if (used.size !== Object.keys(declarations).length) {
    throw new Error('unrelated fixture declarations');
  }
  const finite = finiteDeclarations(declarations);
  for (const field of fields) {
    const name = unfiniteFrom(declarations, field.type, finite, new Set<string>());
    if (name !== undefined) {
      throw new Error(
        `fixture input ${field.name} has no finite value: ${name} recurs with no Optional, List or Map boundary`,
      );
    }
  }
}

/** The type references a declaration holds, union variants in label order. */
function declarationChildren(body: SelectionDeclaration): string[] {
  switch (body.kind) {
    case 'newtype':
      return [body.of ?? ''];
    case 'struct':
      return (body.fields ?? []).map((field) => field.type);
    case 'union': {
      const variants = body.variants as Record<string, string>;
      return Object.keys(variants)
        .sort()
        .map((label) => variants[label]!);
    }
    default:
      return [];
  }
}

/**
 * The model's inhabitation rule: the least fixpoint in which Optional, List and Map are base
 * cases, a struct needs every field and a union one variant.
 */
function finiteDeclarations(declarations: Record<string, SelectionDeclaration>): Set<string> {
  const finite = new Set<string>();
  const inhabited = (source: string): boolean =>
    accessorOptional(source)[1] ||
    accessorCollection(source)[1] ||
    owned(declarations, source) === undefined ||
    finite.has(source);
  for (let grew = true; grew; ) {
    grew = false;
    for (const [name, body] of Object.entries(declarations)) {
      if (finite.has(name)) continue;
      const children = declarationChildren(body);
      const admitted = body.kind === 'union' ? children.some(inhabited) : children.every(inhabited);
      if (admitted) {
        finite.add(name);
        grew = true;
      }
    }
  }
  return finite;
}

/** The first declaration with no finite value that `source` reaches. */
function unfiniteFrom(
  declarations: Record<string, SelectionDeclaration>,
  source: string,
  finite: Set<string>,
  seen: Set<string>,
): string | undefined {
  const [inner, optional] = accessorOptional(source);
  if (optional) return unfiniteFrom(declarations, inner, finite, seen);
  const [item, collection] = accessorCollection(source);
  if (collection) return unfiniteFrom(declarations, item, finite, seen);
  const body = owned(declarations, source);
  if (body === undefined || seen.has(source)) return undefined;
  if (!finite.has(source)) return source;
  seen.add(source);
  for (const child of declarationChildren(body)) {
    const name = unfiniteFrom(declarations, child, finite, seen);
    if (name !== undefined) return name;
  }
  return undefined;
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
  observation: Pick<ResponseObservation, 'declarations'>,
  source: string,
  used: Set<string>,
  stack: Set<string>,
  depth: number,
  allowJson = false,
  input = false,
): void {
  if (depth > DEPTH_LIMIT) {
    throw new Error('response type depth limit');
  }
  const [inner, optional] = accessorOptional(source);
  if (optional) {
    checkType(observation, inner, used, stack, depth + 1, allowJson, input);
    return;
  }
  if (source.startsWith('Map<') && !source.startsWith('Map<String, ')) {
    throw new Error('response map key must be String');
  }
  const [item, collection] = accessorCollection(source);
  if (collection) {
    checkType(observation, item, used, stack, depth + 1, allowJson, input);
    return;
  }
  if ((allowJson && source === 'Json') || (accessorPrimitive(source) && source !== 'Binary64')) {
    return;
  }
  const body = owned(observation.declarations, source);
  // A fixture input admits recursion here; `finiteDeclarations` decides whether it has a finite
  // value (beyond10x/ess#416). A response refuses every recursive type.
  if (body !== undefined && stack.has(source) && input) {
    return;
  }
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
          !Object.values(variants).every((child) => child === null || typeof child === 'string')
        ) {
          throw new Error('invalid response union');
        }
        // A unit variant (ess/22) is `null`: it names no type to check.
        children.push(...(Object.values(variants).filter((child) => child !== null) as string[]));
        break;
      }
      default:
        throw new Error('unknown response declaration');
    }
    for (const child of children) {
      checkType(observation, child, used, stack, depth + 1, allowJson, input);
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
  checkResponseConstraints(
    observation.fields,
    observation.declarations,
    observation.constraints,
    response,
  );
  for (const field of observation.fields) {
    const policy = owned(observation.presence, field.name);
    if (policy === 'null_when_absent' && !present(response, field.name)) {
      throw new Error(
        `response field ${field.name} was left out, and it is declared null_when_absent`,
      );
    }
    if (
      policy === 'omitted_when_absent' &&
      present(response, field.name) &&
      owned(response, field.name) === null
    ) {
      throw new Error(
        `response field ${field.name} was sent as null, and it is declared omitted_when_absent`,
      );
    }
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
  if (observation.nested !== undefined) {
    compareNestedResponse(observation, observation.nested, response, payload);
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
export function admitResponse(value: unknown, major = 21): void {
  const root = closed(
    value,
    'command outcome event fields declarations mappings targets',
    'nested constraints',
  );
  if (Object.hasOwn(root, 'nested') && major < 34) {
    throw new Error('nested response observations require suite/34 or /35');
  }
  admitOutcome(root.outcome);
  admitTypedDeclarations(root.declarations);
  for (const key of ['fields', 'targets']) {
    for (const field of array(root[key])) {
      if (key === 'fields' && isObject(field) && Object.hasOwn(field, 'presence')) {
        // A response field's presence policy (beyond10x/ess#139) is suite/24 vocabulary.
        if (major < 24) {
          throw new Error('field presence policies require suite/24 or /25');
        }
        const { presence: _policy, ...rest } = field;
        admitAccessorField(rest);
        continue;
      }
      admitAccessorField(field);
    }
  }
  decodeResponseObservation(value);
}

/** The closed declaration grammar shared by response and fixture authority. */
export function admitTypedDeclarations(declarations: unknown): void {
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
export function snapshotResponseResult(
  result: CommandResult,
  enforceResultBudget = true,
): CommandResult {
  let raw: string;
  try {
    raw = goMarshal(commandResultShape(result));
  } catch {
    throw new Error('response result is not a typed wire value');
  }
  // Direct returns apply their native payload-only budget at the typed observation.
  if (enforceResultBudget && ENCODER.encode(raw).length > BYTE_LIMIT) {
    throw new Error('response result byte limit');
  }
  const decoded = strictResponseJSON(raw) as EncodedResult;
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

interface NestedResponseMapping {
  target: string[];
  source: string;
}
interface NestedResponseTargets {
  roots: AccessorField[];
  declarations: Record<string, SelectionDeclaration>;
  mappings: NestedResponseMapping[];
}
const NESTED_MEMBER = /^_*[A-Za-z][A-Za-z0-9_]*$/;

function nestedMember(value: unknown): string {
  if (typeof value !== 'string' || !NESTED_MEMBER.test(value)) {
    throw new Error('invalid nested response member');
  }
  return value;
}

function nestedType(raw: string, depth = 0): string {
  if (depth > 32) throw new Error('nested response type wrapper depth');
  raw = raw.trim();
  for (const wrapper of ['Optional', 'List', 'Map']) {
    if (!raw.startsWith(`${wrapper}<`) || !raw.endsWith('>')) continue;
    const inner = raw.slice(wrapper.length + 1, -1);
    if (wrapper === 'Map') {
      const comma = inner.indexOf(',');
      if (comma < 0) throw new Error('invalid structural map');
      const key = inner.slice(0, comma).trim();
      if (!accessorPrimitive(key) || key === 'Binary64')
        throw new Error('invalid structural map key');
      return `Map<${key}, ${nestedType(inner.slice(comma + 1), depth + 1)}>`;
    }
    return `${wrapper}<${nestedType(inner, depth + 1)}>`;
  }
  if (!accessorPrimitive(raw) && raw !== 'Json') name(raw, false);
  return raw;
}

function decodeNestedResponse(value: unknown): NestedResponseTargets {
  const raw = closed(value, 'roots declarations mappings', '');
  const field = (value: unknown): AccessorField => {
    const member = closed(value, 'name type', '');
    return {
      name: nestedMember(member.name),
      type: nestedType(decodedString(member.type, 'structural type')),
    };
  };
  if (!isObject(raw.declarations)) throw new Error('invalid structural declarations');
  const declarations: Record<string, SelectionDeclaration> = Object.create(null);
  for (const [key, value] of Object.entries(raw.declarations)) {
    name(key, false);
    if (!isObject(value)) throw new Error('invalid structural declaration');
    let required = 'kind';
    switch (value.kind) {
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
        throw new Error('invalid structural declaration kind');
    }
    const body = closed(value, required, '');
    const decoded = decodeDeclaration(body);
    if (body.kind === 'struct') decoded.fields = array(body.fields).map(field);
    normalizeNestedDeclaration(decoded);
    declarations[key] = decoded;
  }
  return {
    roots: array(raw.roots).map(field),
    declarations,
    mappings: array(raw.mappings).map((value) => {
      const mapping = closed(value, 'target source', '');
      const target = array(mapping.target).map(nestedMember);
      if (target.length < 2 || target.length > 33) throw new Error('nested response path bound');
      return { target, source: nestedMember(mapping.source) };
    }),
  };
}

function normalizeNestedDeclaration(body: SelectionDeclaration): void {
  switch (body.kind) {
    case 'newtype':
      body.of = nestedType(body.of ?? '');
      break;
    case 'struct':
      for (const field of body.fields ?? []) field.type = nestedType(field.type);
      break;
    case 'union':
      if (!isObject(body.variants)) throw new Error('invalid structural union');
      for (const [label, type] of Object.entries(body.variants)) {
        // A unit variant (ess/22) is `null`: it names no type.
        if (type === null) continue;
        if (typeof type !== 'string') throw new Error('invalid structural variant type');
        body.variants[label] = nestedType(type);
      }
      break;
  }
}
function normalizeNestedResponse(observation: ResponseObservation): void {
  for (const field of [...observation.fields, ...observation.targets])
    field.type = nestedType(field.type);
  for (const body of Object.values(observation.declarations)) normalizeNestedDeclaration(body);
}

function nestedReferences(body: SelectionDeclaration): string[] {
  switch (body.kind) {
    case 'newtype':
      return [body.of ?? ''];
    case 'struct': {
      const fields = body.fields ?? [];
      const seen = new Set<string>();
      if (!fields.length) throw new Error('empty structural struct');
      for (const field of fields) {
        nestedMember(field.name);
        if (seen.has(field.name)) throw new Error('duplicate structural member');
        seen.add(field.name);
      }
      return fields.map((field) => field.type);
    }
    case 'enum': {
      const labels = array(body.variants);
      if (!labels.length || labels.some((label) => typeof label !== 'string')) {
        throw new Error('invalid structural enum');
      }
      return [];
    }
    case 'union': {
      if (typeof body.tag !== 'string' || !body.tag)
        throw new Error('invalid structural union tag');
      if (!isObject(body.variants) || !Object.keys(body.variants).length)
        throw new Error('invalid structural union');
      // A unit variant (ess/22) is `null`: it names no type to reach.
      return Object.values(body.variants)
        .filter((type) => type !== null)
        .map((type) => {
          if (typeof type !== 'string') throw new Error('invalid structural union variant');
          return type;
        });
    }
    default:
      throw new Error('invalid structural kind');
  }
}
function nestedNamed(type: string): string | undefined {
  for (;;) {
    const [inner, optional] = accessorOptional(type);
    if (optional) {
      type = inner;
      continue;
    }
    const [item, collection] = accessorCollection(type);
    if (collection) {
      type = item;
      continue;
    }
    return accessorPrimitive(type) || type === 'Json' ? undefined : type;
  }
}
function nestedStruct(
  type: string,
  declarations: Record<string, SelectionDeclaration>,
  memo: Map<string, string>,
): string {
  const walked = new Set<string>();
  for (;;) {
    const known = memo.get(type);
    if (known !== undefined) {
      for (const previous of walked) memo.set(previous, known);
      return known;
    }
    if (walked.has(type)) throw new Error('cyclic nested response ancestor');
    walked.add(type);
    const [inner, optional] = accessorOptional(type);
    if (optional) {
      type = inner;
      continue;
    }
    const body = owned(declarations, type);
    if (body?.kind === 'newtype') {
      type = body.of ?? '';
      continue;
    }
    if (body?.kind !== 'struct') throw new Error('nested response ancestor is not a struct');
    for (const previous of walked) memo.set(previous, type);
    return type;
  }
}
function validateNestedResponse(
  observation: ResponseObservation,
  nested: NestedResponseTargets,
): void {
  if (
    !nested.roots.length ||
    nested.roots.length > 256 ||
    !nested.mappings.length ||
    nested.mappings.length + Object.keys(observation.mappings).length > 256
  ) {
    throw new Error('nested response relationship bound');
  }
  const names = new Set([
    ...Object.keys(observation.declarations),
    ...Object.keys(nested.declarations),
  ]);
  if (names.size > 4096) throw new Error('nested response declaration union limit');
  for (const [name, body] of Object.entries(nested.declarations)) {
    const other = owned(observation.declarations, name);
    if (
      other !== undefined &&
      nestedDeclarationCanonical(other) !== nestedDeclarationCanonical(body)
    ) {
      throw new Error('conflicting nested response declaration');
    }
  }
  const roots = new Map<string, string>();
  for (const root of nested.roots) {
    nestedMember(root.name);
    if (roots.has(root.name) || present(observation.mappings, root.name))
      throw new Error('duplicate or overlapping nested response root');
    roots.set(root.name, root.type);
  }
  const pending = [...roots.values()];
  const visited = new Set<string>();
  while (pending.length) {
    const type = nestedType(pending.pop()!);
    const named = nestedNamed(type);
    if (named === undefined || visited.has(named)) continue;
    const body = owned(nested.declarations, named);
    if (body === undefined) throw new Error('missing structural declaration');
    visited.add(named);
    pending.push(...nestedReferences(body));
  }
  if (visited.size !== Object.keys(nested.declarations).length)
    throw new Error('unrelated structural declarations');
  const used = new Set<string>();
  const paths: string[][] = [];
  const memo = new Map<string, string>();
  for (const mapping of nested.mappings) {
    const path = mapping.target;
    if (path.length < 2 || path.length > 33) throw new Error('nested response path bound');
    path.forEach(nestedMember);
    const root = roots.get(path[0]!);
    if (root === undefined) throw new Error('undeclared nested response root');
    let type: string = root;
    used.add(path[0]!);
    for (const member of path.slice(1)) {
      const body: SelectionDeclaration = owned(
        nested.declarations,
        nestedStruct(type, nested.declarations, memo),
      )!;
      const field: AccessorField | undefined = body.fields?.find((field) => field.name === member);
      if (field === undefined) throw new Error('undeclared nested response member');
      type = field.type;
    }
    const source = observation.fields.find((field) => field.name === mapping.source);
    if (source === undefined || !responseAssignable(source.type, type))
      throw new Error('nested response terminal mismatch');
    if (
      paths.some((other) =>
        other
          .slice(0, Math.min(other.length, path.length))
          .every((member, index) => member === path[index]),
      )
    ) {
      throw new Error('duplicate or overlapping nested response paths');
    }
    paths.push(path);
  }
  if (used.size !== roots.size) throw new Error('unused nested response root');
}
function compareNestedResponse(
  observation: ResponseObservation,
  nested: NestedResponseTargets,
  response: Record<string, Node>,
  payload: Record<string, Node>,
): void {
  for (const mapping of nested.mappings) {
    let object = payload;
    for (const member of mapping.target.slice(0, -1)) {
      const next = owned(object, member);
      if (!isObject(next) || next instanceof JsonNumber)
        throw new Error(`nested response ancestor ${member} is absent or not an object`);
      object = next;
    }
    const actual = owned(response, mapping.source);
    const emitted = owned(object, mapping.target.at(-1)!);
    const field = observation.fields.find((field) => field.name === mapping.source)!;
    if (
      accessorOptional(field.type)[1] &&
      (actual === undefined || actual === null) &&
      (emitted === undefined || emitted === null)
    )
      continue;
    if (actual === undefined || emitted === undefined || !responseEqual(actual, emitted)) {
      throw new Error(
        `event path ${mapping.target.join('.')} differs from actual response field ${mapping.source}`,
      );
    }
  }
}
// Explicit member order and UTF-8 key order match Rust's nested-only accounting profile.
function nestedKeyOrder(left: string, right: string): number {
  const a = ENCODER.encode(left),
    b = ENCODER.encode(right);
  for (let i = 0; i < Math.min(a.length, b.length); i++) if (a[i] !== b[i]) return a[i]! - b[i]!;
  return a.length - b.length;
}
function nestedMapCanonical<T>(values: Record<string, T>, encode: (value: T) => string): string {
  return `{${Object.keys(values)
    .sort(nestedKeyOrder)
    .map((key) => `${JSON.stringify(key)}:${encode(values[key]!)}`)
    .join(',')}}`;
}
function nestedFieldsCanonical(fields: AccessorField[], presence?: Record<string, string>): string {
  return `[${fields.map((field) => `{"name":${JSON.stringify(field.name)},"type":${JSON.stringify(field.type)}${owned(presence, field.name) === undefined ? '' : `,"presence":${JSON.stringify(owned(presence, field.name))}`}}`).join(',')}]`;
}
function nestedDeclarationCanonical(body: SelectionDeclaration): string {
  let result = `{"kind":${JSON.stringify(body.kind)}`;
  switch (body.kind) {
    case 'newtype':
      result += `,"of":${JSON.stringify(body.of)}`;
      break;
    case 'struct':
      result += `,"fields":${nestedFieldsCanonical(body.fields ?? [])}`;
      break;
    case 'enum':
      result += `,"variants":${JSON.stringify(body.variants)}`;
      break;
    case 'union':
      result += `,"tag":${JSON.stringify(body.tag)},"variants":${nestedMapCanonical(body.variants as Record<string, string>, JSON.stringify)}`;
      break;
  }
  return `${result}}`;
}
function nestedResponseCanonical(observation: ResponseObservation): string {
  const nested = observation.nested!;
  return `{"command":${JSON.stringify(observation.command)},"outcome":{"command":${JSON.stringify(observation.outcome.command)},"outcome":${JSON.stringify(observation.outcome.outcome)}},"event":${JSON.stringify(observation.event)},"fields":${nestedFieldsCanonical(observation.fields, observation.presence)},"declarations":${nestedMapCanonical(observation.declarations, nestedDeclarationCanonical)},"mappings":${nestedMapCanonical(observation.mappings, JSON.stringify)},"targets":${nestedFieldsCanonical(observation.targets)},"nested":{"roots":${nestedFieldsCanonical(nested.roots)},"declarations":${nestedMapCanonical(nested.declarations, nestedDeclarationCanonical)},"mappings":${JSON.stringify(nested.mappings)}}}`;
}
