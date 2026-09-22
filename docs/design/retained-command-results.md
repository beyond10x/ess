# Retained command results and effect-free state refusals

Decision date: 2026-09-22. This is the implementation contract for
`story:retained-command-result-replay`, selected under the approved EKR completion
work. It serves O2. The measured adopter is EKR's Commit and document-only Seed:
an exact retry returns the original durable result, even after a later head,
without effects or a caller-supplied retained identity.

## Existing authority and measured gap

ESS0.28.0 supports preservation but requires its identity in command input.
A mixed state command also requires an effect subject on its default branch,
while an error is correctly forbidden from having effects. The retained scoping
report records the released compiler's four named refusals. A compiler-valid
declaration that demands an error on a successful retry is not conformance.

The typed homes being extended are Outcome and its command-local name in
ess-domain, ResolvedOutcome in ess-compiler, and the closed conformance Step and
response value vocabulary. No product entity, generic lookup facility, AEP
dependency, property bag or new IR envelope is introduced.

## Selection does not imply an effect

For ess/7, a command's explicit subject-state outcomes establish one common
existing entity and one real input identity field. An ordinary default named
error may use that selection authority without declaring an effect subject.
It must be the unique complement of the finite state/input partition. Errors
remain forbidden from declaring subject effects, sets or events. External
branches do not fill uncovered cells; conflicting subjects and WrongState
alongside explicit state guards remain refusals.

Conformance arranges the common subject through real commands, calls the command,
and checks the named error, an empty direct-event list and the unchanged subject.
Cover each refused held state, including Proposed, Rejected and Stale in the
five-state adopter. A first-default witness alone does not cover this contract.

On this new source format, ordinary wrong_state refusal witnesses also observe
the complete held subject before and after the real command and require an empty
direct-event list, including undeclared event names. This applies to ess/7 state
refusals generally, not only to entities that have a replay command. If complete
immediate subject observation is unavailable, refuse synthesis by name. Select
the new suite envelope for these assertions. Existing ess/1 through ess/6 witness
bytes remain unchanged; stronger observations must not silently rewrite them.

The compiler carries this concrete obligation as ResolvedOutcome.complete_refusal,
true for an ess/7 ordinary WrongState outcome with a named error and omitted when
false. It selects observation requirements, not additional product-effect authority.
Synthesis consumes it because the current IR otherwise discards the authored format.
Semantic diff/6 records its before/after change as a typed outcome-observation delta;
legacy IR and projection bytes remain unchanged. Do not add a general source-format
field or a new IR envelope for this bounded obligation.

Existing formats keep their meanings and admission. This new admission requires
ess/7 and does not silently relax old declarations.

## Command-local retained-result relation

An outcome may declare `replays: <origin-outcome-name>`. The origin is an earlier
successful outcome of the same command, with a nonempty typed command response
and an observable subject. Initially the origin creates or moves its subject.
The relation means silent successful return of that originating invocation's
complete typed result while preserving its original subject.

A replay declares no independent creates, moves, updates, preserves, instance,
sets, emits or error. Its original subject and identity source are derived from
the origin. Reject a missing or ambiguous origin, self-reference, replay chains,
cycles, cross-command references, error origins, missing typed response or
unsupported observation. A held-state guard requires the original real input
identity, as on Commit. A document-only create can describe replay eligibility
as implementation-owned external context, as on Seed.

An external eligibility explanation on a replay never selects the fault-injection
strategy. The replay strategy exercises a real original success followed by a
retry. No ConfigureExternalOutcome appears in that sequence.

Example shape:

```yaml
- name: already-committed
  when_subject_state: Committed
  replays: committed
```

Seed uses `replays: seeded`, retains its sole document input and describes the
actual retained-input/context match. Raw input/path equality is not runtime
admission authority. The real kernel compares full parsed input, trusted context
and authority anchor. No caller-provided hash or fabricated revision is added.

## Observe before the retry

The generated witness performs these steps through the actual target:

1. Arrange and invoke the originating successful outcome.
2. Capture its actual typed response and original subject identity, from the
   command input or original emitted identity as already declared.
3. Observe the original subject through declared immediate views and snapshot
   every required field and lifecycle state.
4. Invoke the same public command again under unchanged fixture context.
5. Require the replay outcome, no error, no direct events, a type-valid response
   equal to the captured original, and an unchanged original subject.

Add explicit bounded result-capture and replay-comparison steps to the suite's
closed vocabulary. Their references bind the command, origin and snapshot key.
Only a real successful observed result can be captured. Reject missing,
overwritten, wrong-command or aliased snapshots. Deep-copy observed mutable
values. Recursively refuse Decimal and Binary64 in this initial replay response
contract, including through named wrappers, structs and collections; EKR's
retained result types require neither. Do not change existing response-to-event
semantics or silently route these new comparisons through binary64.

After closed typed admission, compare Integer as the exact admitted i64 value,
Boolean exactly, and String/Timestamp/Duration/Uuid/Bytes as exact admitted text.
Compare enums by variant, lists by ordered elements, and structs/maps by identical
key sets and recursively equal values, independent of key insertion order.
Optional presence is significant: absent and explicit null are different retained
results even where both are individually admitted. Retain resource bounds.
Rust and Go parity vectors include adjacent integers beyond 2^53, i64 endpoints,
nested optionals/collections, text differences and recursive decimal/floating
refusals. These rules concern the typed observed result, not raw JSON whitespace.

The adapter must preserve the actual handler's Integer values losslessly before
constructing the observed result, including nested positions. Native Rust targets
can construct Node numbers directly from the actual i64; native Go targets can
construct json.Number from the actual int64's decimal representation. This is an
adapter obligation, not a target-provided certificate or expected-response cache.
A JSON adapter must decode declared Integer values exactly before constructing
Node, or report the observation Unsupported. Generic JSON-to-Node conversion
through binary64 does not establish that obligation. Do not introduce a lexical
integer-token restriction or change legacy Number serialization to compensate.

Certify each supplied adapter using actual handler results at adjacent values
beyond 2^53 and both i64 endpoints, including nested positions. Equal retries pass;
an adjacent changed result fails. A mutation routing the adapter through binary64
must fail these cases. Exercise any serialization round trip that adapter actually
uses. Keep these typed-boundary executions separate from inherited raw JSON decoder
probes; they do not certify arbitrary JSON numeric spellings across runners.

A snapshot is test evidence, never the implementation's retry memory. The target
must not cache expected responses or answer on behalf of the real handler.
Resampled time, equal results with an extra event/error, or changes to the original
subject must fail the generated witness. Returning current head is distinguishable
only when it differs from the original result; the immediate generated witness
cannot establish that case. Authored later-head acceptance must do so.

A complete immediate subject observation is required. Incomplete projections
refuse synthesis rather than silently omit fields. EKR adoption must supply the
real retained-revision observation, including Root0 after a later head, through
existing query handlers without inventing a new CLI verb.

## Native and downstream agreement

Rust and Go outcome values carry the typed response on the origin and replay,
even when no emitted event maps from that response. Existing source keeps
identical generated bytes. Silent response comparison is a separate assertion;
it never claims an emitted-event payload check proved the retained response.

Resolved IR records a concrete optional origin relation and identity authority,
omitted for legacy models, with total handle lookup and canonical provenance.
Docs, native planning and supported API projections must expose the retained
relation or explicitly refuse unsupported projection; never flatten it into a
fresh mutation. Diff and impact analysis report a changed origin as changed
semantics even if all entity and response types are equal.

Rust and Go conformance runners execute the new assertions. Browser/TypeScript
readers must implement them or refuse the new envelope before target callbacks.
A changed envelope is never labeled with an older version.

## Allocated formats and deciding checks

At source16aa8c7, source supports ess/1 through ess/6, suite through11, and
semantic diff through5. Allocate ess/7, ess-conformance/12 and its coverage form13,
and ess-diff/6 for typed replay and refusal-observation deltas. Preserve all previous formats.
No ess-ir/2 is allocated. Reports keep their existing version only if their
serialized meaning is unchanged.

Old readers reject these new envelopes. New steps mislabeled as an old suite
refuse before callbacks. Unchanged old source, IR, schemas and generated native
and suite bytes retain their frozen identities. Regenerate the RawSpecFile
projection with cargo xtask schema; do not hand-edit its generated schema.

Decisive executions include both native runners, all three default refusal states,
document-only replay using the originally emitted id, exact retained result
comparison, missing/wrong snapshot refusal and negative targets for current-head
response, new timestamp, mutation and extra event/error. Apply targeted mutations
and keep their behavioral failures, then restore the source.

The synthesized witness covers an immediate retry. Adopter acceptance separately
covers restart, later head, changed document content at the same path, semantically
equal whitespace, changed host context/anchor, no new occurrence and no physical
write. None is inferred from an ESS-green model alone.

## Delivery

One implementation unit owns the coupled source/compiler/synthesis/runtime/native
and diff change. Root owns planning, this design, review routing and publication.
The recorded path scope is confirmed before edits; request scope additions before
using them. Independent review and the complete repository gate precede main.
Documentation changes also run site-build. Release, verified artifacts and adopter
pinning are a later coordinated step; no local binary is silently installed.

## Complete actual observations and reachable retries — 2026-09-22

The independent source review reproduced two failures of this contract: declared
view coverage could pass with incomplete actual rows, and an immediate retry
could be emitted without establishing its own guard. Preserve those failing
cases. These corrections refine the still-unreleased suite12/13 implementation;
the existing source allocation and legacy contracts remain unchanged.

Add explicit `SnapshotCompleteSubject { view, subject, shape }` and
`ExpectCompleteSubjectUnchanged { view }` steps. Their closed `SubjectShape`
descriptor has required fields, reachable typed declarations and an explicit
`identity_field` discriminator naming a required non-Optional declared field. It carries
observation authority, never expected result data. Use the existing finite
Field/Declaration admission and typed-value machinery; add no JSON Schema engine.
Require this complete pair for retained-result protocol observations. Keep the
legacy snapshot pair unchanged, including in mixed upgraded suites.

Before callbacks, refuse omitted/malformed descriptors, invalid identities,
unknown or unrelated declarations, unsupported recursive observers, mismatched
pairs/views/identity binding and new steps mislabeled as an old suite. The exact
complete-subject profile shares retained-result resource and type bounds;
Decimal/Binary64, invariant and reading observers refuse synthesis recursively.
Observe lifecycle state at its declared enum type where projected.

Validate each selected actual row against the same retained descriptor before
capturing it and before comparing its replacement. Required fields must exist
and match their types through nested collections and declarations. Optional
absence remains legal and differs from present null. Extra view keys are allowed
and are retained and compared exactly along with declared fields; do not infer
closed-view semantics. Multiple complementary views keep the existing coverage
proof, with actual typed observation for each view.

Source7 complete-refusal generation emits the new pair. Generic `ExpectError`
and `ExpectNoEvents` remain independent assertions: neither implies that a
subject exists. Generator regressions and a deliberate replacement of complete
steps with legacy steps must catch omission of the promised source7 observation.
No new suite-authenticity claim or scenario-marker subsystem is introduced.

For immediate replay, prove the replay branch's own condition against the exact
original input, actor and post-origin subject. Account for held state, subject
fact, input predicates and default selection using existing selector semantics.
Unknown or unavailable eligibility produces a named synthesis refusal. Do not
create another subject, change retry input, insert a later mutation or force an
external result. Keep later-state replay declarations model-valid when this
bounded immediate witness cannot arrange them. Where a state/fact guard applies,
observe its actual expected value before retry as well as the complete row shape.

Both runners must reject incomplete or ill-typed original and subsequent rows;
mutations must independently remove each check and fail. Retain positive full-row,
optional, extra-key, mixed-suite and exact native Integer controls. All original
independent regression cases survive unchanged through correction and re-review.
