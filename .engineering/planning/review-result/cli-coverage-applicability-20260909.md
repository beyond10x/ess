---
format: aep.planning-md/1
id: review-result:cli-coverage-applicability-20260909
kind: review-result
status: active
title: Independent review of typed CLI coverage applicability
relations:
- reviews: story:cli-presentation-binding
revision: 1
---
# Independent review: CLI consumer coverage applicability

Date: 2026-09-09. Scope: read-only review of the three new direct-library
consumer profiles and the nine unqualified topology wire IDs. No Cargo, tests,
source changes, coverage-policy changes or AEP mutations were performed. This
report records source-derived conclusions, not newly executed refusals.

Verdict: there is a concrete boundary/accounting mismatch. Keep these nine IDs
unqualified for the current three profiles. The existing 66-ID topology witness
is useful evidence for the particular admitted value and normalization effects
it asserts; it does not make the nine remaining schema rules downstream behavior.
At least the three closed-object rules, required/min membership and the two
minimum=0 nodes give direct counterexamples. The two format nodes need the same
boundary distinction; a valid u32 edge-value test would establish more value
coverage, not automatically qualify the schema format keyword.

## Why this is an accounting requirement, not merely a missing test

The accepted policy deliberately combines two different inventories: typed Rust
declarations and every structural node/value in the authored RawSpecFile schema.
It presumes no complete mapping between them. See
docs/design/review-consumer-coverage.md:29-56 and :58-74. The wire extractor
retains format, required, numeric bounds and boolean schemas, then inventories
each leaf as its own ID:
crates/edge/ess-xtask/src/consumer_coverage/wire.rs:17-40, :54-61, :126-137,
:161-175.

Every discovered model is crossed with every model-consumer profile. This is
explicit policy at docs/design/review-consumer-coverage.md:18-22 and :126-149,
and an unconditional nested loop at
crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:156-162.
There are only Supported, Refused and finite BaselineUnknown dispositions
(enforce.rs:42-73). The reviewed-candidate input also admits only Supported
and Refused (proposal.rs:60-79). No existing non-behavior disposition describes
a wire admission rule that cannot occur at a downstream typed boundary.
The frozen old unknown set cannot cover a new consumer (policy :139-164).

The current new profiles start at CLI binding compile:
crates/edge/ess-xtask/src/consumer_coverage/profiles.json:2-39.
Its actual signature is compile(&EssIr, &Binding), at
crates/specify/ess-cli-contract/src/resolve.rs:335-336.
Emission adds project(&CompiledBinding), and execution adds runtime::run(&Plan,...),
at crates/generate/ess-cli-project/src/lib.rs:10 and src/runtime.rs:544-550.
None takes a RawSpecFile document or a JSON Schema. The separate existing
authored-admission profile explicitly says parsing does not qualify later
consumer behavior (profiles.json:40-46).

RawSpecFile::parse deserializes a YAML Value into closed raw types
(crates/specify/ess-domain/src/spec.rs:141-154). Compilation then produces a
validated resolved IR, with constructor authority kept private
(crates/specify/ess-compiler/src/lib.rs:14-22 and src/ir.rs:1312).
Changing a raw document into something rejected before that stage does not
exercise any of the three selected CLI entrypoints.

## The exact nine IDs and concrete counterexamples

All IDs below are present in the supplied topology handoff and the existing
target/cli-gate-extraction-20260909-02/wire-inventory.json. I did not regenerate
that inventory. Keyword values are corroborated by the owning derive/field
declarations and schemas/generated/ess.schema.json; the committed schema was
not used as discovery authority.

| Exact ID | Distinguishing source/control and actual boundary |
|---|---|
| wire:RawSpecFile#/definitions/RawReplicas/additionalProperties | Add an undeclared sibling such as extra: 1 beside min: 2. The reader rejects it under deny_unknown_fields; no corresponding extra member can reach CLI compile. |
| wire:RawSpecFile#/definitions/RawTopology/additionalProperties | Add extra: 1 beside workloads. The same closed-reader boundary rejects it before CLI compile. |
| wire:RawSpecFile#/definitions/RawWorkload/additionalProperties | Add extra: 1 beside replicas/stateless/requires. The reader rejects it before CLI compile. |
| wire:RawSpecFile#/definitions/RawReplicas/required | With a replicas object present, remove min while retaining max: 3. min is a required u32 field without a default, so the raw reader rejects it. Omitting the entire optional replicas object instead tests a different parent optional/default rule. |
| wire:RawSpecFile#/definitions/RawReplicas/required/0 | The array member is min. The same missing-min distinction is unavailable at the current CLI boundary; asserting that a valid object happens to contain min does not exercise its requiredness. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/min/minimum | The schema minimum is 0.0, but min: 0, although representable by the raw u32 field, is rejected by workload validation. Negative values are rejected by the raw reader even earlier. No valid EssIr workload reaches the exact zero boundary. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/max/minimum | The schema minimum is 0.0. max: 0 with a valid positive floor is rejected because max is below min; choosing min: 0 instead is independently invalid. A negative ceiling is rejected by the raw reader. No valid EssIr workload reaches the exact zero boundary here either. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/min/format | The emitted format spelling is uint32 and the raw field is u32. Comparing admitted 1 and 4294967295 with a stateless workload could usefully witness downstream no-effect over the represented floor values. The overflow/negative alternative distinguishing the unsigned reader domain cannot reach CLI compile, and changing the schema format spelling is not an input to that API. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/max/format | The same distinction applies to a non-null u32 ceiling, with a valid positive floor. A large admitted ceiling can add a real value witness; it does not test how the CLI boundary handles a format keyword, unsupported schema format, or out-of-range document. |

The closed-object/required/u32 declarations are
crates/specify/ess-domain/src/topology.rs:36-69.
The nonzero-floor and ceiling-versus-floor checks are topology.rs:130-161.
Specification assembly converts through Topology::try_from at
crates/specify/ess-domain/src/spec.rs:830; the checks are not owned by the
CLI binding compiler.

The zero-minimum examples are particularly strong: even the schema's admitted
boundary value cannot become a validated CLI input model. This is not a lack of
inventive fixture construction.

I am not claiming that JSON Schema format necessarily enforces a u32 range.
The relevant facts here are narrower: the extractor treats the exact format
spelling as a structural obligation, the raw reader has a u32 field, and the
new CLI APIs receive neither that spelling nor out-of-range raw values.

## Why the available controls do not close those IDs

The topology witness really does compile both admitted models, compare canonical
plans and complete artifact maps, and execute actual source runtime calls with
recorded typed inputs. See
crates/generate/ess-cli-project/tests/consumer_cli_topology.rs:45-90 and :160-212.
Those are substantive observations. Its omission/null controls are also a
legitimate normalization technique where both raw forms are admitted and the
specific raw distinction is actually asserted.

That technique cannot be transferred without attribution:
- Optional replicas omitted versus present is not min omitted from a present
  replicas object.
- Optional max omitted versus null is not the lower-bound rule for a numeric max.
- Workload map membership is not permission for extra keys in the surrounding
  closed object.
- Floor 2 versus 3 is a field-value change, not a change across the minimum
  boundary, requiredness, unknown-field rule or format spelling.
- A schema snapshot assertion proves the provider's current structural spelling,
  not a new CLI consumer's behavior.

The accepted Supported rule requires a concrete behavior or meaningful changed/
control no-effect assertion; Refused requires the named refusal at its boundary
(docs/design/review-consumer-coverage.md:132-135). A green test can mechanically
be attached to these IDs because semantic attribution is reviewed rather than
proven by the execution engine (policy :20-22). That would not make the attribution
truthful. Likewise, a test that first proves raw rejection and then separately
runs a valid CLI call cannot assign the first observation to a CLI-only refusal.

A stronger allowed-range numeric test is worth retaining if needed for the
Rust fields or corresponding represented value nodes. I do not claim no possible
positive numeric test exists. The objection is granting that finite positive
value observation the different meaning of qualifying an exact reader/schema
constraint for the current downstream-only profile.

## Minimum honest resolution

For the current profile contracts, preserve the nine unqualified IDs, their
27 outstanding cells, the existing baseline and the successful 66-ID witness.
Do not treat this report as authority to register a disposition.

There are two materially different decisions available:

1. Keep the intended direct-library boundaries. Then a narrowly scoped coverage
   policy/accounting change is necessary: represent this specific situation as
   an explicit reviewed non-behavior relationship between a finite wire-rule
   obligation and the typed consumer boundary. Its evidence must name the real
   upstream owner and the typed admission barrier; it must not count as Supported
   or Refused, silently drop inventory, or extend the unknown baseline. This
   needs a separate reviewed scope, closed accounting/serialization decision and
   tests. The repository's format rule requires an explicit migration decision
   if persisted meaning/envelopes change (AGENTS.md, Determinism and formats).
   A blanket wire:* exemption or automatic NotApplicable is expressly forbidden
   by the accepted coverage policy.

2. Deliberately change the profiles to authored-source-admission-through-CLI
   pipelines. Include the actual RawSpecFile parsing, assembly and compiler
   entrypoints; add real tests naming each upstream refusal and asserting that
   later stages were not entered, plus successful controls that reach the CLI.
   This can use the current Supported/Refused cell representation, so a new
   format is not inherently required. It is nevertheless an explicit profile
   contract/fingerprint and attribution decision, not a test-only fix or a claim
   that compile(&EssIr,...) rejects a malformed document. It also changes what
   this coverage row proves. Existing direct-library guarantees and application
   runtime limits must stay clear. No parser test should silently widen the
   current three profiles.

I recommend the first decision if root wants to preserve the independently
callable typed-library claim boundaries already approved. The second is the
smallest option inside the existing cell algebra only if root consciously wants
to measure complete authored ingress pipelines instead. Neither is authorized
merely by wanting this CLI feature's gate to pass. Record and review the chosen
scope before implementation.

This diagnosis is bounded to the nine supplied topology IDs. Other raw-only
wire nodes in the remaining partitions may have the same shape, but were not
independently classified by this review. The 66-ID topology attribution as a
whole was not re-reviewed here, and no new execution result is claimed.

## Handoff

Only this ignored report was written. No source/tests/AEP/coverage files changed.
Worktree: cli-binding-ess-20260909.
Lease: cli-coverage-applicability-review-20260909, released after report creation.
Root retains the tree and owns the disposition/scope decision.

