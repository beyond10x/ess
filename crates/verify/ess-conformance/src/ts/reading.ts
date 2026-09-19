// Reading conformance: an already observed event member, and separately observed clock facts.
//
// Ported from `src/go/reading.go`. The accessor never reads a clock and never normalizes anything
// itself — it hands the adapter a request for *facts* and gives those facts, with the contract the
// suite admitted, to the coordinate reader. An authored contract is a claim, never a certificate,
// which is why the origins handed to the target are a copy: a target that rewrote them would be
// deciding the terms of its own assertion.

import {
  ErrUnsupported,
  JsonNumber,
  closed,
  errorText,
  isClockReadingTarget,
  isUnsupported,
  name,
  text,
  unsigned,
  unsupported,
} from './runtime.js';
import type { Node, ScenarioRun, Step } from './runtime.js';
import {
  compareClockReadings,
  resolveClockReading,
  type ClockCoordinate,
  type ClockReadingEvidence,
  type ReadingOriginPair,
} from './coordinate.js';

/** One declared `(role, offset-authority)` alternative of a reading contract. */
export interface ReadingOrigin {
  role: string;
  offset: string;
}

/** The encoding a member is written in, and the authorities that may have produced it. */
export interface ReadingContract {
  encoding: string;
  origins: ReadingOrigin[];
}

/**
 * Names an already observed event occurrence and member, and the contract the adapter must
 * validate against.
 */
export interface ReadingReference {
  event: string;
  occurrence: number;
  field: string;
  /**
   * The member's declared type, under the name the suite document writes it with.
   *
   * Snake case because this *is* the suite document: `decodeStep` casts the parsed JSON, which is
   * the plain `json.Unmarshal` the Go runtime does at the same point, and a camel-cased field
   * would silently be `undefined` in every request a target is handed.
   */
  declared_type: string;
  contract: ReadingContract;
}

/** Asks for facts, never for a normalized coordinate and never for an assertion result. */
export interface ReadingObservationRequest {
  reading: ReadingReference;
  correlation: string;
}

/**
 * What a target offers so a scenario can order two clock readings.
 *
 * Optional, like `Clock` and `OrderedReader`: a target that does not implement it is *skipped* on
 * the scenarios that need it, never failed. Evidence it cannot observe is answered by raising an
 * error carrying `ErrUnsupported`; an authored contract is never a certificate.
 */
export interface ClockReadingTarget {
  observeClockReading(request: ReadingObservationRequest): ClockReadingEvidence;
}

const READING_MEMBER = /^[A-Za-z][A-Za-z0-9_]*$/;
const CLOCK_INTEGER = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?$/;

/** The key one occurrence of one member is known by, in every message and every request. */
export function occurrenceKey(reference: ReadingReference): string {
  return `${reference.event}#${reference.occurrence}.${reference.field}`;
}

/**
 * Admit one authored reading reference.
 *
 * The encoding and the declared authority have to agree: text that carries its own offset cannot
 * be produced by an epoch-defined formatter, and a literal-Z local time cannot carry one at all.
 * A contract that says otherwise describes no implementation and is refused here rather than
 * producing a verdict later.
 */
export function admitReading(value: unknown): void {
  const fields = closed(value, 'event occurrence field declared_type contract', '');
  name(fields.event, false);
  name(fields.declared_type, false);
  let occurrence: bigint | number;
  try {
    occurrence = unsigned(fields.occurrence);
  } catch {
    throw new Error('invalid reading occurrence');
  }
  if (Number(occurrence) > 65535) {
    throw new Error('invalid reading occurrence');
  }
  let field: string;
  try {
    field = text(fields.field);
  } catch {
    throw new Error('invalid reading member');
  }
  if (field.length > 128 || !READING_MEMBER.test(field)) {
    throw new Error('invalid reading member');
  }
  const event = typeof fields.event === 'string' ? fields.event : '';
  if (`${event}#${occurrence}.${field}`.length > 512) {
    throw new Error('reading occurrence exceeds bound');
  }
  const contract = closed(fields.contract, 'encoding origins', '');
  const encoding = text(contract.encoding);
  const origins = contract.origins;
  if (!Array.isArray(origins) || origins.length === 0 || origins.length > 3) {
    throw new Error('invalid reading origins');
  }
  const previous = new Set<string>();
  for (const value of origins) {
    const origin = closed(value, 'role offset', '');
    const role = text(origin.role);
    const offset = text(origin.offset);
    const pair = JSON.stringify([role, offset]);
    if (previous.has(pair)) {
      throw new Error('duplicate reading origin');
    }
    previous.add(pair);
    if (role !== 'producer_process' && role !== 'consumer_process' && role !== 'unknown') {
      throw new Error('invalid reading origin');
    }
    if (
      !(
        (encoding === 'offset_date_time_text' && offset === 'encoded_offset') ||
        (encoding === 'unix_seconds' && offset === 'encoding_defined_epoch') ||
        (encoding === 'local_date_time_millis_literal_z' &&
          (offset === 'requires_observation' || offset === 'unknown'))
      )
    ) {
      throw new Error('contradictory reading encoding and authority');
    }
  }
}

/**
 * What `expectReadingOrder` reads of the run it is handed.
 *
 * Named from unit A's own `ScenarioRun` rather than restated, so a member that moves there is a
 * type error here instead of an `undefined` at the first reading. `skip` is widened to `void`
 * because a stand-in in a case does not have to abort the way the real one does.
 */
export type ReadingRun = Pick<ScenarioRun, 'target' | 'correlation' | 'seen' | 'fail'> & {
  skip(message: string): void;
};

/** Order two observed readings, or say why the claim cannot be decided. */
export function expectReadingOrder(run: ReadingRun, index: number, step: Step): boolean {
  // Go asks with a type assertion; TypeScript has no interface at run time, so the capability is
  // the presence of the method — and `isClockReadingTarget` is the one place that is decided, for
  // this capability and for the three beside it. A second probe here could drift from it.
  if (!isClockReadingTarget(run.target)) {
    run.skip('clock reading evidence unsupported');
    return false;
  }
  const target = run.target as unknown as ClockReadingTarget;
  const resolve = (reference: ReadingReference | null | undefined): ClockCoordinate => {
    if (reference === null || reference === undefined) {
      throw new Error('missing reading reference');
    }
    let occurrence = 0;
    let value: Node;
    let found = false;
    for (const event of run.seen) {
      if (event.event === reference.event) {
        if (occurrence === reference.occurrence) {
          found = Object.hasOwn(event.payload, reference.field);
          value = found ? event.payload[reference.field] : undefined;
          break;
        }
        occurrence += 1;
      }
    }
    if (!found) {
      throw new Error('reading event occurrence/member not observed');
    }
    let scalar = typeof value === 'string' ? value : '';
    let admitted = typeof value === 'string';
    if (reference.contract.encoding === 'unix_seconds') {
      if (value instanceof JsonNumber) {
        const canonical = clockIntegerText(value.toString());
        admitted = canonical !== null;
        scalar = canonical ?? '';
      } else if (typeof value === 'number') {
        // Go's float64 arm: a whole number inside the admitted range, never rounded into one.
        admitted =
          Number.isFinite(value) &&
          value >= 0 &&
          value <= 253402300799 &&
          value === Math.trunc(value);
        scalar = admitted ? String(value) : '';
      } else if (typeof value === 'bigint') {
        // Go's int64 arm, which states the value and leaves the range to the coordinate reader.
        scalar = value.toString();
        admitted = true;
      } else {
        admitted = false;
      }
    }
    if (!admitted) {
      throw new Error('reading scalar does not match encoding');
    }
    // The target owns its request, including the nested origins; it cannot rewrite the admitted
    // contract this or a later assertion is decided against.
    const requested: ReadingReference = {
      ...reference,
      contract: {
        encoding: reference.contract.encoding,
        origins: reference.contract.origins.map((origin) => ({ ...origin })),
      },
    };
    const evidence = target.observeClockReading({
      reading: requested,
      correlation: run.correlation,
    });
    const origins: ReadingOriginPair[] = reference.contract.origins.map((origin) => [
      origin.role,
      origin.offset,
    ]);
    try {
      return resolveClockReading(
        reference.contract.encoding,
        origins,
        scalar,
        evidence,
        run.correlation,
        occurrenceKey(reference),
      );
    } catch (error) {
      if (errorText(error).includes('UnknownEvidence')) {
        // `fmt.Errorf("%w: %v", ErrUnsupported, err)`: the sentinel's own sentence, then the cause.
        throw unsupported(`${ErrUnsupported.message}: ${errorText(error)}`);
      }
      throw error;
    }
  };

  let failure: unknown;
  try {
    const left = resolve(step.left);
    const right = resolve(step.right);
    const order = compareClockReadings(left, right);
    if (
      (step.order === 'before' && order < 0) ||
      (step.order === 'equal' && order === 0) ||
      (step.order === 'after' && order > 0)
    ) {
      return true;
    }
    return run.fail(index, `clock coordinate order differs from ${step.order}`);
  } catch (error) {
    failure = error;
  }
  if (isUnsupported(failure) || errorText(failure).includes('DifferentClock')) {
    run.skip(`clock comparison unsupported: ${errorText(failure)}`);
    return false;
  }
  return run.fail(index, `clock reading: ${errorText(failure)}`);
}

/**
 * Canonicalize an exact mathematical integer token without a floating-point round trip.
 *
 * A reading that arrives as `1.709296245e9` is the same second as `1709296245`, and one that
 * arrives as `1.5` is not a second at all. Answering that by parsing to a binary64 would admit a
 * value the specification says is not an integer, so the digits are moved rather than converted.
 */
export function clockIntegerText(raw: string): string | null {
  if (raw.length > 512 || !CLOCK_INTEGER.test(raw)) {
    return null;
  }
  const negative = raw.startsWith('-');
  let digits = negative ? raw.slice(1) : raw;
  let exponent = 0;
  const marker = digits.search(/[eE]/);
  if (marker >= 0) {
    const parsed = Number(digits.slice(marker + 1));
    if (!Number.isInteger(parsed) || parsed < -512 || parsed > 512) {
      return null;
    }
    exponent = parsed;
    digits = digits.slice(0, marker);
  }
  const point = digits.indexOf('.');
  if (point >= 0) {
    exponent -= digits.length - point - 1;
    digits = digits.slice(0, point) + digits.slice(point + 1);
  }
  digits = digits.replace(/^0+/, '');
  if (digits === '') {
    return '0';
  }
  while (exponent < 0 && digits.endsWith('0')) {
    digits = digits.slice(0, -1);
    exponent += 1;
  }
  if (exponent < 0 || negative || digits.length + exponent > 12) {
    return null;
  }
  return digits + '0'.repeat(exponent);
}
