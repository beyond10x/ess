// Suite/28: complete actual returns under independently admitted, finite type authority.
import {
  accessorOptional,
  admitOutcome,
  array,
  closed,
  equal,
  isObject,
  JsonNumber,
  name,
  text,
} from './runtime.js';
import type { AccessorField, Node, OutcomeRef, SelectionDeclaration } from './runtime.js';
import { decodeDeclaration, responsePrimitiveAdmits, validateTypedFields } from './response.js';

interface Field extends AccessorField {
  presence?: string;
}
export interface DirectResponse {
  command: string;
  outcome?: OutcomeRef;
  fields: Field[];
  declarations: Record<string, SelectionDeclaration>;
  expected: Record<string, Node>;
}
const encoder = new TextEncoder();
// Native serde_json size, retaining number tokens and counting payload bytes only.
function bytes(value: Node): number {
  if (value instanceof JsonNumber) return encoder.encode(value.raw).length;
  if (Array.isArray(value))
    return 2 + Math.max(0, value.length - 1) + value.reduce((sum, child) => sum + bytes(child), 0);
  if (isObject(value)) {
    const entries = Object.entries(value);
    return (
      2 +
      Math.max(0, entries.length - 1) +
      entries.reduce((sum, [key, child]) => sum + bytes(key) + 1 + bytes(child), 0)
    );
  }
  return encoder.encode(JSON.stringify(value)).length;
}
export { bytes as nativeJSONBytes };
const own = (value: object, key: string): boolean => Object.hasOwn(value, key);

export function admitDirectResponseField(raw: Node): Field {
  const value = closed(raw, 'name type', 'wire display summary code naming presence');
  const label = text(value.name);
  if (!/^_*[A-Za-z][A-Za-z0-9_]*$/.test(label)) throw new Error('invalid response field');
  const type = text(value.type);
  const naming = value.naming == null ? {} : closed(value.naming, '', 'wire display summary code');
  if (
    value.naming != null &&
    ['wire', 'display', 'summary', 'code'].some((key) => value[key] != null)
  )
    throw new Error('field naming must be flat or nested');
  for (const key of ['wire', 'display', 'summary', 'code']) {
    if (value[key] != null && naming[key] != null) throw new Error('conflicting response naming');
    if (value[key] != null) text(value[key]);
    if (naming[key] != null) text(naming[key]);
  }
  const presence = value.presence;
  if (presence != null && presence !== 'null_when_absent' && presence !== 'omitted_when_absent')
    throw new Error('invalid response presence');
  if (presence != null && !accessorOptional(type)[1])
    throw new Error('response presence requires Optional');
  return { name: label, type, ...(presence == null ? {} : { presence }) };
}

export function admitDirectResponse(raw: Node): DirectResponse {
  const value = closed(raw, 'command fields declarations expected', 'outcome');
  name(value.command, false);
  if (value.outcome != null) {
    admitOutcome(value.outcome);
    if (value.outcome.command !== value.command) throw new Error('direct response outcome command');
  }
  const fields = array(value.fields).map(admitDirectResponseField);
  if (!isObject(value.declarations) || !isObject(value.expected))
    throw new Error('response mappings');
  const declarations: Record<string, SelectionDeclaration> = Object.create(null);
  for (const [key, rawBody] of Object.entries(value.declarations)) {
    name(key, false);
    const body = closed(rawBody, 'kind', 'of fields variants tag');
    const keys: Record<string, string> = {
      newtype: 'of',
      struct: 'fields',
      enum: 'variants',
      union: 'tag variants',
    };
    if (!own(keys, body.kind)) throw new Error('response declaration kind');
    closed(body, `kind ${keys[body.kind]}`, '');
    const normalized =
      body.kind === 'struct'
        ? { ...body, fields: array(body.fields).map(admitDirectResponseField) }
        : body;
    // Existing decoding intentionally drops naming; this profile retains field presence.
    declarations[key] = decodeDeclaration({
      ...normalized,
      ...(body.kind === 'struct'
        ? { fields: normalized.fields.map((f: Field) => ({ name: f.name, type: f.type })) }
        : {}),
    });
    if (body.kind === 'struct') declarations[key].fields = normalized.fields;
  }
  if (fields.length === 0 || fields.length > 256 || Object.keys(declarations).length > 4096)
    throw new Error('direct response declaration bound');
  validateTypedFields([fields], declarations, true);
  const contract: DirectResponse = {
    command: value.command,
    fields,
    declarations,
    expected: value.expected,
  };
  if (value.outcome != null) contract.outcome = value.outcome;
  const counter = { bytes: 0 };
  for (const [key, expected] of Object.entries(contract.expected)) {
    const declaration = fields.find((f) => f.name === key);
    if (declaration === undefined) throw new Error('undeclared response literal');
    validateValue(declaration.type, expected, true, contract, counter, 0);
    checkPresence(declaration, true, expected);
  }
  if (bytes(raw) > 1048576) throw new Error('direct response contract byte limit');
  return contract;
}

function checkPresence(field: Field, present: boolean, value: Node): void {
  if (
    (field.presence === 'null_when_absent' && !present) ||
    (field.presence === 'omitted_when_absent' && present && value === null)
  )
    throw new Error('invalid response presence');
}

function jsonValue(value: Node, counter: { bytes: number }, depth: number): void {
  counter.bytes += 1;
  if (depth > 128 || counter.bytes > 1048576) throw new Error('response resource');
  if (typeof value === 'string') counter.bytes += encoder.encode(value).length;
  else if (Array.isArray(value)) {
    if (value.length > 65536) throw new Error('response collection bound');
    for (const child of value) jsonValue(child, counter, depth + 1);
  } else if (isObject(value)) {
    if (Object.keys(value).length > 65536) throw new Error('response collection bound');
    for (const [key, child] of Object.entries(value)) {
      counter.bytes += encoder.encode(key).length;
      jsonValue(child, counter, depth + 1);
    }
  } else if (
    value !== null &&
    typeof value !== 'boolean' &&
    typeof value !== 'number' &&
    !(value instanceof JsonNumber)
  )
    throw new Error('invalid response Json');
  if (counter.bytes > 1048576) throw new Error('response resource');
}

function validateValue(
  type: string,
  value: Node,
  present: boolean,
  contract: DirectResponse,
  counter: { bytes: number },
  depth: number,
): void {
  if (depth > 128 || counter.bytes > 1048576) throw new Error('response resource');
  const [inner, optional] = accessorOptional(type);
  if (optional) {
    if (!present || value === null) return;
    return validateValue(inner, value, true, contract, counter, depth + 1);
  }
  if (!present) throw new Error('missing response value');
  const declaration = contract.declarations[type];
  if (declaration !== undefined) {
    if (declaration.kind === 'newtype')
      return validateValue(declaration.of, value, true, contract, counter, depth + 1);
    if (declaration.kind === 'enum') {
      if (typeof value !== 'string' || !array(declaration.variants).includes(value))
        throw new Error('response enum');
      counter.bytes += encoder.encode(value).length;
      return;
    }
    if (!isObject(value)) throw new Error('response object required');
    if (declaration.kind === 'struct') {
      if (Object.keys(value).some((key) => !declaration.fields.some((f) => f.name === key)))
        throw new Error('extra response member');
      for (const f of declaration.fields) {
        counter.bytes += encoder.encode(f.name).length;
        checkPresence(f, own(value, f.name), value[f.name]);
        validateValue(f.type, value[f.name], own(value, f.name), contract, counter, depth + 1);
      }
      return;
    }
    const tag = declaration.tag;
    const content = tag === 'value' ? 'content' : 'value';
    if (Object.keys(value).some((key) => key !== tag && key !== content))
      throw new Error('extra response union member');
    if (typeof value[tag] !== 'string' || !own(declaration.variants, value[tag]))
      throw new Error('response union tag');
    // A unit variant (ess/22) is the tag alone.
    if (declaration.variants[value[tag]] === null) {
      if (own(value, content)) throw new Error('response unit variant payload');
      return;
    }
    return validateValue(
      declaration.variants[value[tag]],
      value[content],
      own(value, content),
      contract,
      counter,
      depth + 1,
    );
  }
  if (type.startsWith('List<') && type.endsWith('>')) {
    if (!Array.isArray(value) || value.length > 65536) throw new Error('response list');
    for (const child of value)
      validateValue(type.slice(5, -1), child, true, contract, counter, depth + 1);
    return;
  }
  if (type.startsWith('Map<String, ') && type.endsWith('>')) {
    if (!isObject(value) || Object.keys(value).length > 65536) throw new Error('response map');
    for (const [key, child] of Object.entries(value)) {
      counter.bytes += encoder.encode(key).length;
      validateValue(type.slice(12, -1), child, true, contract, counter, depth + 1);
    }
    return;
  }
  if (type !== 'Json' && !responsePrimitiveAdmits(type, value))
    throw new Error('invalid response primitive');
  if (type === 'Json') jsonValue(value, counter, depth);
  else if (typeof value === 'string') counter.bytes += encoder.encode(value).length;
  if (counter.bytes > 1048576) throw new Error('response resource');
}

export function compareDirectResponse(contract: DirectResponse, actual: Node): void {
  if (!isObject(actual)) throw new Error('command returned no response');
  if (Object.keys(actual).some((key) => !contract.fields.some((f) => f.name === key)))
    throw new Error('extra response field');
  const counter = { bytes: 0 };
  for (const f of contract.fields) {
    validateValue(f.type, actual[f.name], own(actual, f.name), contract, counter, 0);
    checkPresence(f, own(actual, f.name), actual[f.name]);
  }
  if (bytes(actual) > 1048576) throw new Error('direct response byte limit');
  for (const [key, expected] of Object.entries(contract.expected))
    if (!own(actual, key) || !equal(actual[key], expected))
      throw new Error('response differs from literal');
}
