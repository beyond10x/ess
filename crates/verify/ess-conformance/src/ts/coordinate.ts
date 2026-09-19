// Evidence checks shared verbatim with the Go and Rust emitters.
//
// Ported from `crates/specify/ess-domain/src/reading/coordinate.go`, which the Go target pulls into
// its emitted runtime from outside its own directory (`src/go/mod.rs:202`). Every refusal below is
// the one the Go file makes, with the same message, because a coordinate the two runtimes read
// differently is a scenario that passes in one language and fails in the other.
//
// Nothing here reads a live clock. A coordinate is a bounded position within an *observed* process
// epoch, and two of them are comparable only when the adapter observed the same epoch for both.

/** One declared `(role, offset-authority)` alternative from a reading contract. */
export type ReadingOriginPair = readonly [string, string];

/**
 * Adapter-observed facts for one reading occurrence. An empty identity refuses.
 *
 * `offsetMinutes` is minutes east of UTC and is observed, never declared: it is present only for
 * the `fixed_offset` formatter, which is the one encoding whose text does not carry its own offset.
 * Absent and `null` are the same answer, both standing for the Go `*int16` nil.
 */
export interface ClockReadingEvidence {
  /** Scenario correlation supplied by the caller. */
  correlation: string;
  /** Requested event occurrence and member. */
  occurrence: string;
  /** Actual process instance; empty means unknown. */
  processInstance: string;
  /** Actual clock epoch; empty means unknown. */
  epoch: string;
  /** Observed `producer_process` or `consumer_process` role; unknown refuses. */
  origin: string;
  /** Observed `encoded_offset`, `unix_seconds` or `fixed_offset` mode. */
  formatter: string;
  /** Independently observed minutes east of UTC, for `fixed_offset` only. */
  offsetMinutes?: number | null;
}

/** A bounded coordinate within an observed process epoch, not physical elapsed time. */
export interface ClockCoordinate {
  /** Exact normalized milliseconds. */
  unixMillis: number;
  /** Actual observed process instance. */
  processInstance: string;
  /** Actual observed epoch. */
  epoch: string;
}

/** The closed set of reasons a reading cannot establish the requested coordinate comparison. */
export type ClockReadingReason =
  | 'InvalidContract'
  | 'InvalidValue'
  | 'OutOfRange'
  | 'UnknownEvidence'
  | 'MismatchedOccurrence'
  | 'IncompatibleOrigin'
  | 'IncompatibleFormatter'
  | 'DifferentClock';

/**
 * A refusal carrying the reason in its message, exactly as the Go file writes it.
 *
 * The message is load-bearing: the reading accessor decides *skip* rather than *fail* by looking
 * for `UnknownEvidence` and `DifferentClock` in it, which is what the Go runtime does. `name` is
 * left inherited so `String(error)` reads `Error: clock reading: <reason>`, as the Go
 * `fmt.Errorf("clock reading: %s", …)` renders.
 */
export class ClockReadingError extends Error {
  readonly reason: ClockReadingReason;

  constructor(reason: ClockReadingReason) {
    super(`clock reading: ${reason}`);
    this.reason = reason;
  }
}

const MAX_UNIX_SECONDS = 253402300799;
const MAX_UNIX_MILLIS = 253402300799999;
const IDENTITY_LIMIT = 512;

function fail(reason: ClockReadingReason): never {
  throw new ClockReadingError(reason);
}

/**
 * Resolve a declared reading using occurrence-scoped facts, without consulting a live clock.
 *
 * Checks occurrence, source, declared origin and observed formatter before it normalizes anything:
 * an authored contract is a claim about the implementation, never a certificate that the claim
 * holds.
 */
export function resolveClockReading(
  encoding: string,
  origins: readonly ReadingOriginPair[],
  value: string,
  evidence: ClockReadingEvidence,
  correlation: string,
  occurrence: string,
): ClockCoordinate {
  if (evidence.correlation !== correlation || evidence.occurrence !== occurrence) {
    fail('MismatchedOccurrence');
  }
  for (const identity of [
    evidence.correlation,
    evidence.occurrence,
    evidence.processInstance,
    evidence.epoch,
  ]) {
    if (identity.length === 0 || identity.length > IDENTITY_LIMIT) {
      fail('UnknownEvidence');
    }
  }
  if (evidence.origin !== 'producer_process' && evidence.origin !== 'consumer_process') {
    fail('UnknownEvidence');
  }
  if (origins.length === 0 || origins.length > 3) {
    fail('InvalidContract');
  }
  for (const [index, origin] of origins.entries()) {
    let valid =
      origin[0] === 'producer_process' ||
      origin[0] === 'consumer_process' ||
      origin[0] === 'unknown';
    valid &&= authorityMatches(encoding, origin[1]);
    for (const previous of origins.slice(0, index)) {
      if (previous[0] === origin[0] && previous[1] === origin[1]) {
        valid = false;
      }
    }
    if (!valid) {
      fail('InvalidContract');
    }
  }
  const observedOffset = evidence.offsetMinutes ?? null;
  let authority: string;
  if (
    encoding === 'offset_date_time_text' &&
    evidence.formatter === 'encoded_offset' &&
    observedOffset === null
  ) {
    authority = 'encoded_offset';
  } else if (
    encoding === 'unix_seconds' &&
    evidence.formatter === 'unix_seconds' &&
    observedOffset === null
  ) {
    authority = 'encoding_defined_epoch';
  } else if (
    encoding === 'local_date_time_millis_literal_z' &&
    evidence.formatter === 'fixed_offset' &&
    observedOffset !== null
  ) {
    authority = 'requires_observation';
  } else if (evidence.formatter === 'unknown') {
    return fail('UnknownEvidence');
  } else {
    return fail('IncompatibleFormatter');
  }
  const admitted = origins.some(
    (origin) => origin[0] === evidence.origin && origin[1] === authority,
  );
  if (!admitted) {
    fail('IncompatibleOrigin');
  }
  let millis: number;
  if (encoding === 'unix_seconds') {
    const seconds = parseSignedInteger(value);
    if (seconds === null) {
      fail('InvalidValue');
    }
    if (seconds < 0 || seconds > MAX_UNIX_SECONDS) {
      fail('OutOfRange');
    }
    millis = seconds * 1000;
  } else {
    millis = normalizeClockText(value, observedOffset);
  }
  return { unixMillis: millis, processInstance: evidence.processInstance, epoch: evidence.epoch };
}

/** Whether a declared offset authority is the one its encoding can carry. */
function authorityMatches(encoding: string, authority: string): boolean {
  if (encoding === 'offset_date_time_text') {
    return authority === 'encoded_offset';
  }
  if (encoding === 'unix_seconds') {
    return authority === 'encoding_defined_epoch';
  }
  if (encoding === 'local_date_time_millis_literal_z') {
    return authority === 'requires_observation' || authority === 'unknown';
  }
  return false;
}

/**
 * The Go `strconv.ParseInt(value, 10, 64)` plus its round-trip check, in one answer.
 *
 * Go refuses a token whose canonical rendering differs — `01`, `+1`, `-0` — and so does this. The
 * admitted range is bounded well inside the exact integer range of a binary64, so no reading this
 * function admits can lose a digit.
 */
function parseSignedInteger(value: string): number | null {
  if (!/^-?(0|[1-9][0-9]*)$/.test(value)) {
    return null;
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || String(parsed) !== value) {
    return null;
  }
  return parsed;
}

/**
 * Parse offset text, or literal-Z millisecond text with an independently observed offset.
 *
 * Exported for the reading accessor's own cases; the grammar is fixed and the range is the same
 * 1970–9999 the encodings above admit.
 */
export function normalizeClockText(text: string, localOffsetMinutes: number | null): number {
  const invalid = (): never => fail('InvalidValue');
  // Go scans bytes: a non-ASCII character makes the UTF-8 length disagree with the character
  // count, and every such reading is refused either way, so the ASCII check comes first here.
  for (let index = 0; index < text.length; index += 1) {
    if (text.charCodeAt(index) > 127) {
      invalid();
    }
  }
  if (text.length < 20 || text.length > 29) {
    invalid();
  }
  if (
    text[4] !== '-' ||
    text[7] !== '-' ||
    text[10] !== 'T' ||
    text[13] !== ':' ||
    text[16] !== ':'
  ) {
    invalid();
  }
  const year = digits(text.slice(0, 4));
  const month = digits(text.slice(5, 7));
  const day = digits(text.slice(8, 10));
  const hour = digits(text.slice(11, 13));
  const minute = digits(text.slice(14, 16));
  const second = digits(text.slice(17, 19));
  if (year < 1970 || year > 9999) {
    fail('OutOfRange');
  }
  if (
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > 31 ||
    hour < 0 ||
    hour > 23 ||
    minute < 0 ||
    minute > 59 ||
    second < 0 ||
    second > 59
  ) {
    invalid();
  }
  let millis = 0;
  let position = 19;
  if (text[position] === '.') {
    if (text.length < 24) {
      invalid();
    }
    millis = digits(text.slice(20, 23));
    if (millis < 0) {
      invalid();
    }
    position = 23;
  }
  const suffix = text.slice(position);
  let offset = 0;
  if (localOffsetMinutes !== null) {
    if (position !== 23 || suffix !== 'Z') {
      invalid();
    }
    offset = localOffsetMinutes;
  } else if (suffix !== 'Z') {
    if (suffix.length !== 6 || (suffix[0] !== '+' && suffix[0] !== '-') || suffix[3] !== ':') {
      invalid();
    }
    const offsetHours = digits(suffix.slice(1, 3));
    const offsetMinutes = digits(suffix.slice(4, 6));
    if (offsetHours < 0 || offsetMinutes < 0) {
      invalid();
    }
    if (offsetHours > 14 || offsetMinutes > 59 || (offsetHours === 14 && offsetMinutes !== 0)) {
      fail('OutOfRange');
    }
    offset = offsetHours * 60 + offsetMinutes;
    if (suffix[0] === '-') {
      offset = -offset;
    }
  }
  if (offset < -840 || offset > 840) {
    fail('OutOfRange');
  }
  // Date.UTC normalizes an impossible day the way time.Date does, and reading the parts back is
  // how the Go file refuses February the thirtieth.
  const instant = Date.UTC(year, month - 1, day, hour, minute, second, millis);
  const read = new Date(instant);
  if (
    read.getUTCFullYear() !== year ||
    read.getUTCMonth() + 1 !== month ||
    read.getUTCDate() !== day
  ) {
    invalid();
  }
  const result = instant - offset * 60000;
  if (result < 0 || result > MAX_UNIX_MILLIS) {
    fail('OutOfRange');
  }
  return result;
}

/** The value of a run of ASCII digits, or -1 when any character is not one. */
function digits(part: string): number {
  let value = 0;
  for (let index = 0; index < part.length; index += 1) {
    const code = part.charCodeAt(index);
    if (code < 0x30 || code > 0x39) {
      return -1;
    }
    value = value * 10 + (code - 0x30);
  }
  return value;
}

/** Compare coordinates only within one actually observed source epoch. */
export function compareClockReadings(left: ClockCoordinate, right: ClockCoordinate): number {
  if (
    left.processInstance === '' ||
    left.epoch === '' ||
    right.processInstance === '' ||
    right.epoch === ''
  ) {
    fail('UnknownEvidence');
  }
  if (left.processInstance !== right.processInstance || left.epoch !== right.epoch) {
    fail('DifferentClock');
  }
  if (left.unixMillis < right.unixMillis) {
    return -1;
  }
  if (left.unixMillis > right.unixMillis) {
    return 1;
  }
  return 0;
}
