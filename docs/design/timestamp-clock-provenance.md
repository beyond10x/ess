# Timestamp clock provenance

A named newtype may attach a closed reading contract to its direct String or Integer
representation. This uses conditional source ess/3; absence retains legacy bytes.
Existing Timestamp and elapsed-window semantics remain unchanged. The companion typed
model uses existing ESS syntax to name the value and observation concepts before this
attachment is implemented.

Supported encodings are offset-date-time text, local millisecond text with a literal Z,
and integer Unix seconds. Declared producer/consumer alternatives are requirements,
not evidence that a particular branch, process or formatter produced a value.
Literal Z on a local reading never supplies UTC authority.

Normalization is deterministic and bounded: years 1970 through 9999, valid Gregorian
dates, hours 00–23, minutes and seconds 00–59, and fixed offsets within ±14:00.
Offset text uses YYYY-MM-DDTHH:MM:SS with optional exactly three millisecond digits and
Z or ±HH:MM. Local literal-Z text requires exactly three millisecond digits and an
independently observed fixed offset. Unix seconds are exact signed integers in
0–253402300799; normalized milliseconds must remain in 0–253402300799999.
No leap seconds, timezone database, host timezone, current-time fallback or calendar
arithmetic is inferred. Malformed, out-of-range and unobserved inputs refuse.

The adapter supplies observed source process instance and epoch, actual origin and
formatter mode, scoped to the current scenario correlation and an existing event
occurrence/member. It supplies facts rather than normalized expected values or verdicts.
Unknown source, epoch, origin or required offset remains explicit and cannot prove
comparability. Authored names/reasons do not authenticate a clock. The adapter owns
truthful observation and validation against its actual model; a hand-written suite is
not a certificate. Unsupported instrumentation remains a non-passing capability result.

Comparison normalizes two observed readings and orders coordinates only when their
observed process instance and epoch agree. Different encodings of that same source
can compare, including known fixed-offset local text. This is not a proof of physical
elapsed time or monotonicity. Cross-source calibration/subtraction remains unsupported.

The new conformance operation belongs to coordinated suite6/7 and requires report2;
complete native target failures use coordinated target-failure3. Native Rust/Go helpers
retain scalar wire bytes and execute the same bounded normalization contract. Schema
annotations retain the attachment without promoting local literal-Z text to date-time.
Generic predicate comparison must not silently erase an attached clock contract.

Deciding witnesses compare known-source offset text and Unix seconds, normalize a
known fixed-offset local reading, and refuse wrong correlation, mismatched occurrence,
source or epoch, unknown pre-initialization formatter offsets, erased origin and
cross-producer comparison. Old readers must refuse new vocabulary. Production consumer
instrumentation and calibration are not claimed by controlled fixture evidence.

## Executable seams

The native Rust newtype method `resolve_reading` and Go method `ResolveReading` retain
the declared contract. Their `ClockReadingEvidence` argument supplies an occurrence key,
scenario correlation, observed process instance/epoch, observed producer/consumer origin,
and formatter mode. A fixed-offset formatter additionally supplies minutes east of UTC.
An empty source identity or `unknown` origin/mode is unresolved. The returned coordinate
can be compared with `compare_clock_readings` / `CompareClockReadings`; callers must keep
the observation adapter honest. Public coordinate DTOs are not authentication tokens.

Conformance uses `ReadingReference::from_event` to retain the nominal attachment for an
existing event member, including transparent newtype wrappers. A zero-based occurrence
indexes events already observed in the isolated scenario. `ExpectReadingOrder` supports
`before`, `equal`, and `after`; it does not infer an ordering claim from the attachment.
The target observation seam validates the actual declared event/member/type contract
before supplying source and formatter facts. Persisted references are structurally
validated but cannot certify their own truth. Direct persisted readers before suite/6
refuse this operation. Authored YAML convenience syntax for this comparison is not added;
callers construct the typed suite operation and persist it through the admitted writer.

Rust conformance and generated Rust use the same source for normalization and evidence
checks. Native Go and the generated Go conformance runner share one Go normalization and
evidence implementation. Generated Go conformance runtime bytes change to carry these
new optional operations even for old suites; legacy source/IR attachments remain omitted.

Semantic diff records attachment addition, removal, or editing as the typed
`ReadingContractChanged` delta, even if the newtype representation is identical. Its
before/after contracts include declared requirements only. This uses coordinated
ess-diff/3; requesting /1 or /2 refuses the new vocabulary. Unchanged attachments
produce no reading delta, and legacy differences retain their previous format.
