# Pre-execution fixture values

A deployment assigns identities and canonical input values that a deterministic conformance generator cannot invent. A target must not repair a scenario by copying observed post-state into its expected values. Resolve explicitly declared fixture values before any target scenario or command starts, validate their types, and freeze the values used by both requests and assertions.

## Source declarations

A command may declare `fixture_inputs`, mapping an existing input field to a lower-kebab fixture name. This is a conformance adapter control, like the existing external-outcome control; it does not add a production request field. The input's declared type is the fixture's type authority. The first version refuses such a binding when the field participates in any outcome input predicate, rather than silently changing a generated negative witness. Unknown fields, empty names and incompatible uses of the same fixture name are rejected.

Authored scenarios gain an explicit `$fixture` reference and typed fixture declarations for values that are not command-wide inputs. Literal authored inputs remain literals: no adapter or compiler silently replaces an author's requested value. References can be used in inputs, view expectations and event payload expectations. Compatibility is checked against the actual target field type.

## Suite and execution

A scenario that uses fixtures starts with a typed fixture-resolution declaration. The runtime validates and resolves this prelude before BeginScenario. A provider receives the finite set of fixture names and source-owned type declarations, and returns values only; it cannot change types, mappings or expected outcomes. Missing provider capability is an explicit skip. A malformed, incomplete or wrongly typed provider result is an execution error before target activity. Values are copied into scenario-local immutable storage. They are not re-read later, shared between scenarios, or coerced.

The value vocabulary gains Fixture. A resolved event assertion combines literal values, fixture references and the existing shape assertions on the first matching direct event of the last command, selected by event name before comparing values. Separate occurrences cannot satisfy different halves of one claim, and a later correct occurrence cannot hide an earlier wrong value. Existing instance and observed-event references retain their meanings and cannot become forward references.

Nested records, lists, maps, enums and nominal types reuse the independently admitted declaration/value checker used by typed response observations. Unsupported invariants or reading semantics are refused explicitly; a primitive-only shape check must not certify a constrained type.

## Compatibility

New specification meaning, authored syntax and suite vocabulary require new format admission. Unused fixture support must leave old canonical suite bytes unchanged. Old readers reject the new format before target activity. Both ordinary and inventory-bearing suite formats require equivalent admission. Rust, emitted Go and emitted TypeScript must agree on valid values, malformed input, missing capabilities, immutability and failures. No report is promoted into a baseline until every previous named passing and answered scenario remains so.

The candidate allocates source `ess/13`, authored `ess-scenario/3`, ordinary suite `ess-conformance/18` and coverage suite `ess-conformance/19`. These are unreleased additions. Each suite major implies every one below it; the TypeScript runtime, which refuses suites 12–17 whole, admits 18 and 19 and still refuses their retained-result steps, string operators and aggregate scenarios by name. Browser replay refuses independently provisioned fixtures before emitting an artifact.

## Boundaries

A fixture provider owns provisioning and teardown outside this runtime. It may read its own independently provisioned resources before execution; it must not observe the tested result to choose an expectation. This feature neither synthesizes upstream stimuli nor publishes events, starts phones, authenticates callers, or changes application code. Generated output values such as negotiated media descriptions are a separate modelling question.

## Required evidence

- A non-literal provisioned identity reaches both the command and its later equality assertion.
- A target returning a different identity or field value fails.
- Wrong type, missing field, unknown name, unsupported constraint and incompatible fixture reuse are refused before target activity.
- A provider or target mutating a retained object cannot change a later expected value.
- Fixture values are resolved once per scenario and cannot leak into the next.
- Missing capability skips explicitly; it never passes or quietly uses a generator's placeholder.
- An ordinary input predicate and its negative witnesses cannot be overridden by fixture metadata.
- Old-format readers reject the new vocabulary, and fixture-free suites keep their original bytes.
