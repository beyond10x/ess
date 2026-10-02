# One-time response values

Design for [issue 389](https://github.com/beyond10x/ess/issues/389), 2026-10-02.
Status: coordinator-approved contract, awaiting independent design review and AEP
implementation acceptance. This page is binding once accepted; it does not claim
that the current executable implements the feature. The accompanying
[illustration](one-time-response-values.example.yaml) is proposed source/21 input.

## Authority and source syntax

A successful command outcome with `returns: true` may declare
`one_time_response: [secret]`. Each distinct list member resolves to a response
field of that command. It identifies the exact outcome that originates the
value; this is not shared `Field` metadata and does not annotate entity fields.
An absent list preserves existing semantics and bytes. An explicitly empty list,
duplicate name, unknown field, non-returning outcome, error outcome, or
`accepts: nothing` combination is refused with its source location.

The initial value profile is required String, optionally through finite,
transparent named newtypes. The actual string must be nonempty. Optional,
collections, structs, unions, enums, numeric and recursive shapes are explicitly
refused for marked fields; other unmarked response fields retain their types.
Existing newtype constraints still apply. No randomness, entropy, hashing or
secure storage property follows from the declaration or `{generated: true}`.

The plaintext may appear only in that invocation's selected marked response
field. It must not appear in another response position, a declared event, an
error, a view, or a declared persistent field. Multiple marked fields originate
separate values and cannot copy one another. The same value cannot be originated
again, including by another command, another actor, or a later rotation.
Comparison is exact, case-sensitive UTF-8 text; observed strings and object keys
are checked for the plaintext as a substring. Thus a prefixed plaintext leak is
also forbidden. No normalization, decoding, cryptographic inference or arbitrary
transformation inversion is claimed. A separately computed verifier is permitted
provided it does not contain the plaintext. ESS does not prove it is a verifier.

## Invocation, retry and lifecycle meaning

This is at-most-once disclosure per originated value, not guaranteed receipt by
the client. A successful invocation consumes its disclosure opportunity even
when its reply is lost. An implementation must not recover that plaintext by
returning a retained reply. A retry follows the command's ordinary declared
outcomes: it may refuse, return a different declared response, or originate a
fresh value through an origin outcome. An outcome with a required `secret`
response cannot arbitrarily omit it to satisfy this rule.

Reject every `replays:` edge whose origin marks a one-time field. Existing
retained-result replay compares the complete original typed result; do not
redact its comparison or redefine old replay semantics. A rotation is another
successful origin invocation, whose value must differ from all previously
captured values in the observed trace. No new rotation command, retry identity,
storage engine, actor policy or concurrency ordering is invented.

The semantic prohibition applies across restarts and concurrent requests too.
This release's serial finite witnesses do not establish either property: report
separate restart and concurrency coverage as unsupported, with their required
implementation-owned checks. There is no target restart or response-loss receipt
primitive today. Serial repeated invocation establishes a retry observation,
not a transport-level lost-response experiment. The implementation owns durable
consumption, atomic issuance and its crash/lost-reply behavior.

## Static flow checking

Resolve the marked names into a concrete outcome policy in typed IR. Flow
ownership is the selected outcome, not the shared command response shape: an
unmarked alternative may use its own response-source mappings without acquiring
another outcome's policy. Reject a
declared response-source mapping from a marked field into an event payload or
any other supported declared persistent/output destination. Walk nested declared
mapping expressions rather than checking spelling alone. Existing unsupported
response-to-state assignments remain refusals; do not make them legal here.
Reject retained replay references as above. A name match with a separately
generated verifier is not itself a plaintext flow.

Where a declared conversion, accessor or other opaque mapping would require
analysis ESS cannot perform, produce a located unsupported-flow-analysis refusal;
never certify it safe by absence of a known direct edge. Distinguish this from
implementation code, which is not a declared flow and is outside static proof.
Entity fields or views with unrelated values are not forbidden merely because
their names resemble a secret. Black-box checks below detect actual plaintext.

## Finite conformance contract

Every generated one-time witness is isolated using the existing scenario
lifecycle. It arranges a genuine successful origin through existing public
commands, captures the actual response, and never supplies an expected secret
to the target. Capture is runner-only state, not an input or target certificate.
It is deep-owned, bounded, scoped to the scenario, and discarded when it ends.

Generate a deterministic inventory for every marked command/outcome/field:

1. Origin: validate the complete declared response, capture each marked nonempty
   value, and scan its entire response, direct events and error. Only the exact
   designated field value is exempt for its newly originated value. Scan that
   field against all older captured values, and all other marked values, too.
2. Retry: execute the exact same command input and actor again. Check ordinary
   declared behavior as well as absence of every prior plaintext. If it takes an
   origin outcome, capture its new value only after checking against prior ones.
3. Reads: after origin and after rotation, exercise each declared view with the
   finite parameter witnesses the current synthesizer can arrange; scan rows,
   declared paging output and all other declared returned string positions.
4. Commands: fork an origin-prefix witness for each declared command/outcome the
   existing arrangement machinery can reach. Observe every operation performed
   after capture, including arrangement operations, not only the final command.
   Check each actual response, event and error against all captured values.
5. Actors: repeat applicable follow-up witnesses for every declared actor,
   including distinct actors with identical permission sets and a denied actor
   when declared. Permission equivalence does not imply identity equivalence.
   The actor switch uses the actual target caller seam; no fabricated authority.
6. Rotation: invoke a reachable origin again and then perform the follow-up reads
   and retry. Observe at least two successful origin invocations; freshness is
   checked against every value captured so far in that trace.

At origin and after each subsequent operation, also query the independent
`observe_events` publication log for every declared event in the scenario's
correlation context. A response's direct-event list is not that log. Scan every
returned payload and string key, including delayed publication and publications
from a command that reports refusal. Repeated observation of one occurrence is
idempotent scanning, not a second issuance or a duplicate-count failure; no log
observation is exempt, even when the same occurrence was returned directly.
Never capture an originating value from a log. Check the whole observed history
against all currently captured values; do not discard older occurrences based
on guessed identity or a missing sequence number.

Event observations carry existing explicit deadlines. For each declared delayed
delivery/timing obligation, use its arranged finite deadline and repeatedly scan
through the observation window, including its closing boundary; also perform a
final scan before scenario teardown. `observe_events` may return early when a
harmless occurrence exists: one request with a future deadline does not prove
the remaining window was observed. Use the existing target clock/wait seams to
establish window completion; if the adapter cannot establish that completion,
report the cell unsupported rather than busy-loop or infer elapsed time. A
clean early event followed by a delayed leak must fail. Rescan retained histories
without deduplicating by equal payload; optional sequence fields do not become
mandatory event identity or correlation guarantees.
An unspecified unbounded future publication is outside the finite claim. A
required delay that cannot be arranged or observed has its own unsupported cell,
not an immediate empty-log pass. Log-only and delayed-log redisclosure mutants
must fail; a duplicate clean occurrence in direct and independent observations
must remain healthy.

“Each” refers to declared finite inventory cells, not every possible input or
history. Use existing supported witness construction and outcome selection, with
stable source references and deterministic IDs. No fixture or outcome is invented
to make a cell pass. Unreachable, unarrangeable, unsupported paging/actor/target
observation and resource-bound cells are explicitly refused or inventoried as
unsupported with their reason, never silently omitted or reported passed. An
ordinary suite request that cannot carry the required inventory refuses; the
coverage suite retains cells and statuses. Empty/vacuous success is not coverage.
Where a model has no applicable view or additional actor, record the finite
inventory fact; do not invent an endpoint or label nonexistent work passed.

The observer runs after every actual query/command in these traces before any
ordinary mismatch diagnostic is rendered. It also integrates with authored
scenario compilation: a selected marked origin arms the same observer for the
remaining timeline, including subsequent rotations. No authored capture syntax
or new scenario format is necessary. Suite admission validates the capture
authority, command/outcome/field/type binding, and observation lifetime before
target callbacks. A forged exemption, reference to an uncaptured value, duplicate
capture identity or missing observation authority is refused.

Use closed, typed suite capture/observation policy data. Select ordinary suite/34
and coverage suite/35 whenever it occurs. Do not create one generic step whose
arbitrary property map claims to describe every security policy.

## Bounds and diagnostics

Reuse the direct-response profile per observed payload: at most 1 MiB serialized
bytes, depth 128 and 65,536 members per collection. Add explicit per-trace bounds
of 256 captured values and 1 MiB total captured UTF-8 bytes. Crossing a bound is
an explicit unsupported observation, never truncation, eviction, partial scanning
or a pass. The generator records unsupported cells when a required trace exceeds
these bounds. Generated work must be bounded before starting target execution.

Reports, stdout/stderr produced by the runner, persisted evidence and generated
failure text must not include captured plaintext, excerpts, object keys containing
it, hashes of it, or target-provided diagnostics that may carry it. A failure
names only a stable scenario/source reference, operation ordinal, fixed channel
category, marked field identifier and rule code. Do not print a data-derived
JSON pointer. One-time traces use value-free diagnostics even for ordinary shape,
type, target-error and mismatch failures, including malformed origin replies
before capture. Do not serialize the capture store or raw target observations.
This does not promise memory zeroization or control an external target's own logs.

Finite black-box runs do not prove absence in implementation storage, logs,
undeclared endpoints, encoded/transformed values or every future execution.
Record these implementation-owned obligations beside the explicit unsupported
restart/concurrency cells; a green serial witness is only that witness's result.

## Format allocation and compatibility

Inventory at base `1ff305685`: source admission supports ess/1–20
(`ess-domain/src/system.rs`); suites support 1–33
(`ess-conformance/src/scenario.rs`), with coverage through33 (`coverage.rs`).
The chosen fast lane is source **ess/21**, ordinary **ess-conformance/34** and
coverage **ess-conformance/35**. Old source tags refuse the new key. Explicit
older suite pins refuse the new observation. Old readers reject the new envelope
before interpreting new vocabulary. Existing inputs without the declaration,
their IR, suites and projections retain their exact canonical bytes.

This explicitly supersedes the unreleased scheduling decision in
`.engineering/waves/downstream-gaps.md`: its previously coordinated ess/21 syntax
bundle moves **as a whole to ess/22**. The coordinator records that reallocation
and preserved history; this feature does not claim any of that bundle delivered.

`EssIr` has no serialized format envelope; its `format()` is source authority and
is skipped in serialization (`ess-compiler/src/ir.rs`). Add a concrete resolved
policy omitted when empty, not an invented ess-ir/2 envelope. Source21 governs its
new meaning. Semantic delta admission currently supports ess-diff/1–12; allocate
**ess-diff/13** for typed add/remove/change of outcome disclosure policy and
reject older explicit pins that would lose it. Adding the restriction narrows
allowed implementation behavior; removing it relaxes the guarantee. Ordinary
unchanged deltas preserve their older format and bytes. No report schema change
is needed: existing report/2 carries fixed-code checks and exact suite association.

## Projection and implementation scope

Rust domain/compiler/schema, semantic diff, conformance generation/admission and
native execution implement the policy. The source schema is regenerated from the
Rust authority. All projections must either preserve the policy and state its
implementation/verification obligations, or refuse it explicitly by name. A
structural type library alone cannot enforce temporal disclosure: it must not
claim to. Generated server/library code must not silently present a policy-free
implementation as complete.

The operator requires complete runtime parity in this delivery. Rust/native,
generated Go, generated TypeScript and every applicable browser/WASM/reference
execution path execute the complete admitted one-time vocabulary and its
prerequisites. This explicitly closes existing Go/TypeScript suite28/29 direct
response, suite30/31 delivery-context and suite32/33 structured-value gaps before
adding suite34/35. Both current emitters' three refusal helpers and Go's newest
admitted major27 are implementation gaps to close, not acceptable final gates.
A closed admitted-major set is only an intermediate safety check. A language-specific refusal,
skip, dropped step or weaker check does not satisfy acceptance. No partial-runtime
release is permitted. Intrinsic finite restart/concurrency limits and genuinely
unarrangeable model cells remain explicit and equal across runtimes; they cannot
be used to hide a missing port.

Use the same healthy/adversarial fixture manifest and exact admitted suite bytes
across runtimes. Assert identical scenario identity, executed/pass/fail/error/
unsupported counts, fixed rule codes, disclosure results and value-free diagnostic
semantics. Generated target drivers exercise real runtime callbacks; a report
translator or precomputed answer is not execution parity. Preserve legacy suite
bytes and existing unrelated semantics while implementing prerequisites.

The path inventory and ownership plan below distinguish an execution runner from
a model visualization. A browser player that currently emits no execution report
must not be mislabeled a conformance runtime; it still preserves and displays the
policy without exposing captured values. Where an actual target runs through a
browser or WASM adapter, run the shared healthy/mutant checks through that adapter.
Other generator surfaces are tested for policy preservation plus obligations or
explicit refusal, including docs, schema, OpenAPI, AsyncAPI and synthesized
implementations. This projection boundary is not permission for a conformance
runner to refuse the feature.

## Typed-contract-first implementation ownership

Freeze the closed suite DTOs, admission rules, format allocation, fixed diagnostic
codes and shared fixture manifest before ports begin. One contract owner changes
domain/compiler/schema/diff and conformance synthesis/admission/coverage/authored
modules. It publishes exact fixture bytes and expected result counts for each
healthy and mutant trace. Ports consume these bytes; they do not change policy
or choose their own coverage inventory. Shared-module changes route through the
contract owner to avoid incompatible local interpretations.

The frozen authority must carry and validate the accepted marked newtype's
constraints. The existing `typed_fields.rs` helper rejects declarations with
`reading` or constrained bodies and cannot silently stand in for this profile.
Define concrete constraint authority in the source21/suite34 observer profile,
or reuse a typed profile only after proving it carries those rules. Go/TypeScript
declaration decoders must read the same authority and execute the same constraints.
An opaque/custom reading outside that closed profile is an explicitly rejected
source shape across all runtimes, not a per-language skip. Establish valid and
invalid constrained-newtype vectors before the contract freeze.

| Path | Source seam and required work | Ownership after contract freeze |
| --- | --- | --- |
| Rust library/native CLI | `ess-conformance/src/runner.rs`, `target.rs`, `direct_response.rs`, new bounded observer; CLI conform entry and report rendering | Native owner, complete execution and diagnostic tests |
| Interpreted/reference targets | `ess-conformance/src/interpret.rs`, `interpret/execute.rs`, `reference.rs`, `faulty.rs` | Native owner; interpreted source21 origins produce deterministic fresh test values, and generic healthy/faulty targets exercise all observer channels; fixed-domain billing/oracle targets remain scoped to their own models |
| Generated Go | `ess-conformance/src/go/mod.rs`, embedded `go/runtime.go`, `go/reading.go`, `go/response.go` | Go port owner; close all direct-return, delivery-context and structured-value prerequisite refusals with complete validation/execution and one-time parity |
| Generated TypeScript | `ess-conformance/src/ts/mod.rs`, embedded `ts/runtime.ts`, `ts/response.ts` and admission modules | TypeScript port owner; same three prerequisites and observer semantics, asynchronous callback ordering and error redaction |
| Browser suite player | `ess-conformance/src/web.rs`, `web_replay.rs`, `assets/player.js`, coverage player/admission assets | Browser owner; admit/preserve/render source policy and suite vocabulary without presenting visualization as a successful execution or disclosing captured values |
| Browser/WASM target boundary | `ess-synth` web/Rust dispatch emitters; `ess-synth/tests/json_web.rs`, `upsert_by_existence.rs`; `examples/billing-web`, `website/src/pages/lab/_bridge.mjs`, `_run.ts`, `Taskfile.yml` site-lab route | Browser owner; test actual target adapters through the existing execution harnesses with shared vectors; do not claim the fixed billing lab independently executes arbitrary suites |
| Reports/recorded execution | `ess-conformance/src/report.rs`, `record.rs`, `recorded.rs`, `evidence.rs`, CLI rendering | Native owner audits all persisted/rendered observations; ports match value-free report semantics |

All paths above are inventoried, not all necessarily need edits. The exact
implementation manifest is settled from the frozen DTO and existing call graph;
any uncovered execution entry blocks completion until accounted for. Existing
browser coverage replay explicitly emits no execution report (`web.rs`), and the
WASM lab bridge dispatches a fixed generated model, so neither is falsely counted
as another passing suite runner. A Rust integration harness generates the shared
fixtures and drives language/browser processes under the existing template
conventions; add no standalone shell/Python or handwritten foreign-language
test executable. Executable repository additions are Rust; existing generated
foreign-runtime template conventions remain the port seam.

Ports can proceed independently after the contract freezes, then one parity
integration owner runs the full manifest across all executable paths and checks
counts, redaction, unsupported categories and legacy controls before the combined
candidate's gate. A missing runtime is a release blocker, not deferred delivery.

## Acceptance and implementation order

1. Domain/compiler/schema regression tests: valid required String/newtype;
   every invalid shape/name/combination; old-format refusal; forbidden nested
   direct flow, opaque flow refusal, replay contradiction; unchanged legacy bytes.
2. Typed suite/admission/coverage/diff: exact format selection, old-reader and
   explicit-pin refusals, forged capture/exemption refusal, deterministic IDs,
   zero dropped unsupported cells and disclosure-policy delta coverage.
3. Native execution: healthy issuer and fresh rotation pass. Independent faulty
   targets redisclosing through retry, another response field, event, error,
   view row, object key, embedded string, same-permission actor switch,
   independent event log, delayed publication and old-value rotation
   fail generated witnesses. Multiple marked fields and several rotations guard
   against checking only the latest value or exempting an entire reply.
4. Redaction and bounds: malformed origin, target error, ordinary assertion
   failure, malicious data-derived keys, old-value leak and limit exhaustion
   leave no plaintext in serialized report, rendered diagnostic or captured
   runner output; unsupported does not count as pass. Authored traces share the
   same observer. Unsupported restart/concurrency are visible.
5. Shared full-runtime parity controls, projection refusal/preservation controls
   and finite inventory completeness;
   then affected package tests, strict Clippy, supported fmt check, ci-lint and
   docs checks appropriate to changed published sources. Preserve same-test
   red/green evidence and independent adversarial review before publication.

No implementation is authorized merely by creating this page. The coordinator
accepts the reviewed design and records acceptance before source code begins.
