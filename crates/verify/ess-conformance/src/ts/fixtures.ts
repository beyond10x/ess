// Independently supplied values resolved before scenario activity.
import { array, closed, goMarshal, isObject, SelectionObservation, strictJSON } from './runtime.js';
import type { AccessorField, Node, SelectionDeclaration } from './runtime.js';
import { admitTypedDeclarations, decodeDeclaration, validateTypedFields } from './response.js';

export interface FixtureContract {
  fields: AccessorField[];
  declarations: Record<string, SelectionDeclaration>;
}

export function fixtureName(value: unknown): asserts value is string {
  if (
    typeof value !== 'string' ||
    value.length > 128 ||
    !/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(value)
  ) {
    throw new Error('invalid fixture name');
  }
}

export function admitFixtures(value: unknown): FixtureContract {
  const root = closed(value, 'fields declarations', '');
  admitTypedDeclarations(root.declarations);
  const fields = array(root.fields).map((raw) => {
    const field = closed(raw, 'name type', '');
    fixtureName(field.name);
    if (typeof field.type !== 'string') throw new Error('fixture type must be text');
    return { name: field.name, type: field.type };
  });
  const declarations = Object.fromEntries(
    Object.entries(root.declarations as Record<string, Node>).map(([key, body]) => [
      key,
      decodeDeclaration(body),
    ]),
  );
  if (fields.length === 0 || fields.length > 256 || Object.keys(declarations).length > 4096) {
    throw new Error('fixture field/declaration bound');
  }
  validateTypedFields([fields], declarations);
  if (new TextEncoder().encode(goMarshal(value)).length > 1048576)
    throw new Error('fixture contract byte limit');
  return { fields, declarations };
}

/** Copies all nested values, preserving exact numeric tokens and rejecting non-wire data. */
export function copyFixtureValue(value: Node): Node {
  const raw = goMarshal(value);
  if (new TextEncoder().encode(raw).length > 1048576) throw new Error('fixture value byte limit');
  return strictJSON(raw);
}

export function fixtureValues(contract: FixtureContract, provided: Node): Record<string, Node> {
  // Validate the original result before copying: undefined or functions must not disappear.
  if (!isObject(provided)) throw new Error('fixture values must be an object');
  const expected = contract.fields.map((field) => field.name).sort();
  if (JSON.stringify(Object.keys(provided).sort()) !== JSON.stringify(expected)) {
    throw new Error('fixture result must contain exactly the declared names');
  }
  const observer = SelectionObservation.forResponse(contract.declarations);
  const counter = { bytes: 0 };
  // A round trip normalizes ordinary JS numbers into the runtime's exact numeric tokens.
  const values = copyFixtureValue(provided) as Record<string, Node>;
  if (JSON.stringify(Object.keys(values).sort()) !== JSON.stringify(expected)) {
    throw new Error('fixture result lost a declared value');
  }
  for (const field of contract.fields) {
    observer.validateValue(
      field.type,
      values[field.name],
      Object.hasOwn(values, field.name),
      counter,
      0,
    );
  }
  if (counter.bytes > 1048576) throw new Error('fixture value byte limit');
  return values;
}

/** Inspect only typed scenario-value positions; literal objects cannot declare references. */
export function admitFixtureSteps(steps: Node[]): void {
  const declared = new Set<string>();
  const referenced = new Set<string>();
  for (const [index, step] of steps.entries()) {
    if (step.step === 'resolve_fixtures') {
      if (index !== 0) throw new Error('fixture resolution must be the first and only prelude');
      for (const field of admitFixtures(step.fixtures).fields) declared.add(field.name);
    }
    const groups: Record<string, Node>[] = [];
    switch (step.step) {
      case 'execute_command':
      case 'expect_invocation':
        groups.push(step.input ?? {});
        break;
      case 'snapshot_subject':
        groups.push(step.subject ?? {});
        break;
      case 'query_view':
      case 'eventually_view':
      case 'expect_halt':
      case 'eventually_halt':
        groups.push(step.params ?? {});
        break;
      case 'expect_event_values':
        if (Object.keys(step.payload).length === 0) throw new Error('empty event value assertion');
        groups.push(step.payload);
        break;
    }
    if (step.step === 'expect_view' || step.step === 'eventually_view')
      groups.push(step.expectation.fields ?? {});
    for (const group of groups)
      for (const value of Object.values(group)) {
        if (value.kind === 'fixture') referenced.add(value.fixture);
      }
  }
  if (declared.size !== referenced.size || [...referenced].some((key) => !declared.has(key))) {
    throw new Error('fixture declarations must exactly match the referenced names');
  }
}
