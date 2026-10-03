# Mutation scope, single-event alternatives and known failures

Status: proposed for independent review, 2026-10-03. This coordinates issues #212, #236,
#294, #295 and #296 within the accepted bundle. It changes neither specification truth nor
the ordinary conformance verdict. Implementation has not been admitted by this document.

## Existing seams and demonstrated gap

`ess-conformance/src/mutate.rs` mutates authored documents, compiles each through the ordinary
compiler, synthesizes a suite and runs that suite against an unchanged implementation. Its
`Ruler::new` refuses failed/error baselines; `Ruler::judge` excludes unexecuted baseline scenarios
and detects whether their bodies changed. Keep these shared semantics for direct and collected
audits. `report.rs` distinguishes Failed, Error and Unsupported; `counts.rs` additionally carries
producer-qualified Skipped. None of those is a known-failure exemption.

On the retained combined CLI, `verify conform mutate --path examples/billing --target billing
--class emit-drop --format json` executes a green 32-scenario baseline but produces five
stillborn mutants, zero kills and exit 3. Removing the sole emitted event violates
ESS-COMMAND-007. This is a generator coverage gap, not permission to weaken outcome validation.
The current CLI also has no known-failure option. The original issue reproductions request a
declared list of failing scenarios and a valid single-event mutation.

## Single-event mutation: emit-swap

Add a separate class `emit-swap`; preserve the meaning and IDs of `emit-drop`. A swap site is an
outcome with exactly one emitted event and no error. Replace that event with a different existing
event, retaining every other command effect, guard, subject and transition. Do not remove an
expectation from a suite or bypass source compilation.

Choose candidates in qualified-name byte order. A candidate must have exactly the same resolved
field names and types (including named type identity and Optional/container structure) as the
original event. It must be published by every component that accepts this command, and at least
one such component must exist. No inferred publisher is invented for a component-free model:
that site has no admissible alternative under this operator. Rename the outcome's explicit
payload-map event key when present, retaining its field expressions. Recompile the entire mutated
document set; select the first candidate passing ordinary compilation. Thus implicit payload
inference, ownership, binding type checks and naming are still the compiler's responsibility.

The deterministic mutant ID includes class, command, outcome, old event and new event. Its change
description states both events. An event with the wrong field type, merely the same wire label,
or a publisher missing from one accepting component is not an alternative. Candidate enumeration
must not change baseline documents or synthesize a new event declaration.

When no candidate survives, record the source site under `unavailable_sites` with closed reason
`no_compatible_event_alternative`. This is neither a mutant kill nor a stillborn mutant: no
valid altering edit was available. A selected in-scope unavailable site prevents audit success
(exit 3 unless an actual survivor already requires exit 1), including when other mutants were
killed. Text and JSON state that single-event substitution was not audited there. This honestly
limits the operator; it does not claim to detect an implementation that emits nothing merely
because a different-event mutant was killed.

Acceptance uses creating, updating and moving outcomes with compatible alternatives, incompatible
payload alternatives, two accepting components with different publication sets, and no alternative.
Healthy unchanged targets must kill the swaps through actual event observations. A target that
wrongly emits the substituted event must survive that mutant, and a runner that discards event
expectations must fail the audit control. Exercise native and actual generated Go/TypeScript
runners against real generated Rust/Go service targets, plus external emit/collect. Keep multi-event
emit-drop controls. No-alternative output and exit status must be identical through both routes.

## Existing accepted operators and component scope

#212 remains three serial implementation slices: non-creating `sets-drop` with separated prior
and input values; `precedence-swap` on adjacent overlapping input-guard branches; and equality
sub-leaf reversal plus outward integral boundary movement under `guard-boundary`. Do not add
creating sets-drop or stored-guard precedence by inference. Guard arithmetic is checked: an
out-of-range literal produces an explicit unavailable site, never wraparound or a duplicate edit.
Outward movement is +1 for < or <= upper bounds, and -1 for > or >= lower bounds; equality and
inequality use their reversal arm. Existing strictness swaps retain their IDs.

#236 uses the existing `synthesize_for` component admission on both baseline and each mutant.
An emitted mutant is in scope if its component suite differs from the component baseline in a
scenario body or scenario presence. This includes cross-domain commands the component accepts.
Owned-domain filtering is incorrect. `synthesize_for` deliberately preserves whole-system
refusals; comparing that inventory cannot establish component scope. Keep those global facts
visible separately, without adding unrelated mutants to the selected component's denominator.

For scoring an already in-scope mutant, use only refusal keys naming a scenario present in the
union of its baseline and mutant component suites. Other refusals are reported as unscoped facts,
not scored gained refusals. If a selected component has no executable scenario for a command
site it accepts, report that source site separately as unavailable with reason
`selected_command_without_scenario` when a synthesis refusal names that command/outcome. This is
an explicit incomplete audit obligation, not an in-scope mutant or a kill. Match typed semantic
command/outcome references from synthesis, not prefixes parsed from diagnostic prose. A refusal
that cannot be attributed remains a global fact and cannot alone make a mutant in scope. Exercise
both other-component-only refusal changes and a selected command refused before any scenario is
emitted. This preserves #236's changed-scenario mutant selection while making its incompleteness
visible separately.
For unavailable sites, retain the site when its command belongs to the component's accepted
command set; otherwise record it as out of scope. This selection grants no evidence of execution.

`--component` is supported by emit and collect. Collect derives scope from the manifest; a supplied
flag must agree exactly. Built-in `--target` refuses component scope. Record excluded mutant IDs
and unavailable sites with reason `outside_component`, outside every score denominator. Do not
use component scoping to hide a selected component's missing observation or synthesis refusal.

## Known-failure declaration

Use an external, closed JSON document `ess-known-failures/1`, not a marker on authored outcomes.
The intended ESS behavior and ordinary suite stay unchanged. Required fields are:

- `format`, exactly `ess-known-failures/1`;
- `spec_digest`, the admitted baseline specification digest;
- `suite_digest`, the digest of the exact original baseline suite bytes under suite admission;
- `implementation`, the exact nonempty implementation identity used by the observed report;
- `implementation_build`, the SHA-256 identity of the public implementation build, supplied by
  the execution host before target execution and independently of this declaration;
- `failures`, a nonempty list of `{scenario, reason, tracking}` records in scenario ID order.

Scenario IDs are exact, with no wildcard, prefix, command-wide or outcome-wide matching. Reason
and tracking are nonempty user-authored strings; the latter names a repair issue or equivalent
record and is never fetched. Unknown fields, duplicate keys/IDs, invalid digests, unknown scenario
IDs and mismatched spec/suite/implementation identity refuse before scoring. The public build
identity is separate from the report's implementation label. Protected one-time runners keep
their fixed label and empty version: never restore arbitrary target-returned text. Direct built-in
targets use the SHA-256 of the running ESS executable, computed before target execution. External
hosts supply the SHA-256 of their immutable target build through explicit execution-context
configuration, before any target call. Never derive it from a private value, a target response, or
by copying the declaration's value. This is declared host execution provenance, not remote
attestation; the runner remains responsible for truthfully identifying its target just as it is
responsible for its result. A mismatched build refuses even when the safe label matches.

After ordinary report validation, every listed scenario must currently be Failed. A passing entry
is stale and refuses; Error, Unsupported, Skipped or absent entries refuse. Every unlisted failure
still refuses a mutation baseline as ESS-MUTATE-001. An execution error is never exempted.
Declarations exclude entire scenarios from mutation scoring; they do not distinguish two defects
within one scenario. This scope is explicit in reports and documentation, and the unchanged
conformance result continues to fail even for a matching declaration.

No declaration is inferred from previous runs or automatically rewritten when a test changes.
In particular, a still-failing scenario from a changed suite cannot inherit an old exception by
name alone. Original-byte digest admission is performed before JSON normalization. Protected
runtime inputs, expected values and observed values never enter declaration or accounting fields.

## CLI and accounting without changing ordinary truth

Add `--known-failing FILE` and `--accounting-out FILE` to `verify conform run` and `report`.
Accounting output requires a declaration and a distinct output destination. With a declaration,
accounting output is required, avoiding a machine-readable result that silently loses exclusions.
The ordinary report retains its existing format, counts and verdict. `run`, including strict mode,
retains its ordinary failure status/exit behavior. `report` retains its existing exit 0 meaning
that a report was written, regardless of verdict; declaration/admission failure exits 2 and writes
neither result. This is an explicit redesign of #296's suggested strict-mode waiver.

Write the separate closed `ess-known-failure-accounting/1` document with original report-byte,
suite-byte and declaration-byte digests; exact implementation identity and build; spec digest; matched
known-failed IDs/reasons/tracking; unexpected-failed IDs; and the original terminal counts. Its
known-failed count is a subset of Failed, never added to Passed or subtracted from total. It has
no conformance-passed field. Its validator rechecks all identities and the partition against the
original report and declaration. Validate all inputs before writing outputs; create-new outputs
must not overwrite one another or any input. A write failure is failure, never a success receipt.

Generated Go/TypeScript runners keep their ordinary failed reports and process status. Their
results can be accounted through `report --suite --results`; no environment variable makes a
failing runner pass. External `report` additionally requires `--implementation-build <sha256>`
and `--execution-context-out FILE` whenever known-failure accounting is requested. The build value
comes from the host's public build artifact, not from target identity callbacks. Native run and
external report must produce equivalent accounting for the same statuses, labels and build
provenance. Browser conformance truth and one-time identity redaction are unchanged.

Persist host execution provenance in a separate closed `ess-conformance-execution/1` sidecar:
format, exact original report-byte digest, exact suite-byte digest, report implementation label
and implementation_build. There are no values, timestamps, free-text host fields or commands.
The runner freezes the build identity before its first target invocation; it binds the report
digest after producing the ordinary report. Generated Go/TypeScript runners accept an explicit
pre-execution public build digest and execution-context output path, validating both before
execution. The Rust external-report CLI creates the same envelope from its explicit caller claim.
Unknown/duplicate fields and invalid/mismatched digests refuse. A context file is not an attestation
of a remote binary; consumers must trust the result-producing host, as for the ordinary report.
Tests must show protected response/identity sentinels and their hashes cannot enter this envelope.

## Mutation scoring and persistence

`mutate --target` accepts `--known-failing FILE`. `mutate --emit` also accepts it: validate static
suite identity/IDs, copy its original bytes into the new emission directory, and record that
declaration's digest/path, implementation label and implementation_build in the manifest. No matching current failure
is claimed until execution. `--collect` uses that manifest-bound declaration. If supplied a file
again, it must match the recorded original bytes; refusing an unbound late declaration prevents
quietly changing the audit contract after observing mutant reports. Legacy manifest/1–3 refuse
this new option; re-emission under /4 is the migration.

The shared scorer admits only baseline-passing scenarios as eligible witnesses. A declared failed
scenario can never kill a mutant, even if its mutant copy still fails differently. Unsupported and
Skipped retain existing exclusions. Errors remain refusals. A mutant with a changed excluded
scenario and no eligible killer is Inconclusive; a real eligible failure can still kill it. If
every possible witness is excluded, the result is Inconclusive, never Equivalent or Killed. A
mutant that adds a scenario absent from the baseline has no baseline-passing control for that
scenario; it is excluded with reason `no_baseline_control`, and cannot alone kill the mutant.
At least one baseline-passing scenario is required to start scoring. Eligible identities refer to
baseline scenario IDs, while changed-body comparison uses the existing typed suite comparison.

The current valid-suite, gained-refusal and dead-guard rules remain in force after eligibility is
applied. Missing mutant reports remain Inconclusive. Baseline report admission precedes scoring;
all reports must identify the same implementation, and each report must match its own emitted
suite. Manifest/4 collection requires report/2, which admits the exact suite-byte digest; report/1
is accepted only with legacy manifest/1–3 semantics. Test a stale report with unchanged spec,
scenario IDs and counts but different suite bodies. When a declaration is bound, each baseline
and mutant report additionally requires its host-produced execution/1 sidecar, conventionally
`execution.json` beside `report.json`. Admission checks the exact report bytes, that report's
suite, and the same implementation_build as the manifest/declaration. Missing or mismatched
baseline context refuses; a missing mutant context is Inconclusive and can never produce a kill.
Direct execution creates equivalent private context facts. The scorer must not compare a mutant's
spec digest to the unmutated declaration digest.

Coordinate all new persisted mutation fields in manifest/4 and report/4, already needed by #236.
Record scope, sorted out-of-scope IDs/counts, unavailable sites, declaration identity, known-failed
baseline exclusions and per-mutant exclusion reasons. Old manifest/1–3 readers remain supported
with their unchanged meaning; old readers reject /4 before ignoring fields. New writers use /4;
do not relabel changed bytes /3. Ordinary suite, source and conformance report versions do not
change for this design. Canonical JSON, original-byte checks, digest mismatches, closed enums and
strict duplicate/unknown-field admission have native tests. Persisted aliases are not accepted.

Mutation exit 0 means all eligible scored mutants are killed/equivalent, at least one
non-equivalent mutant ran, and there are no survivors, unavailable in-scope sites, inconclusive or
unwitnessed entries. Known failures are displayed prominently even when this mutation-only audit
succeeds; it makes no conformance claim. Survivors retain exit 1; incomplete/refused audits retain
exit 3. Emit success means artifacts were emitted, as today, and is never a mutation score.

## Required proof and rejected alternatives

Direct and emit/collect controls must compare canonical reports for the same target execution:
green baseline; one correctly declared Failed scenario plus independent eligible killers;
unlisted failure; all witnesses excluded; changed excluded scenario; newly added unbaselined
scenario; stale passing declaration; Error/Unsupported/Skipped substitution; duplicate/unknown ID;
changed raw suite bytes/spec/build; missing report; wrong mutant report; tampered declaration;
cross-component commands; no in-scope mutation site; selected and excluded unavailable sites.
Run actual generated Go/TypeScript results through `report` and collect, checking raw reports
remain failed, strict execution still fails and the accounting partition matches native execution.
Plant faults in exclusion and stale-entry handling and show each falsely scored mutant or wrongly
accepted declaration is caught. Unit-only construction of a report is insufficient acceptance.

The second adopters are a partially repaired order service with a known cancellation defect, and
a catalog component audited independently from billing. Both need exact, visible exclusions while
remaining nonconformant to intended behavior. Changing nothing leaves the entire audit blocked;
marking a spec outcome as optional or intended would mix implementation state into domain truth;
dropping event expectations removes the very obligation under audit. These alternatives are
rejected. This proposal instead separates explicit audit eligibility from unchanged conformance
truth, and uses a compiler-valid event substitution with an honest unavailable-site result.
