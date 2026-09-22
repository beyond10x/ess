---
format: aep.planning-md/1
id: story:retained-command-result-replay
kind: story
status: active
title: Express and verify retained command results on exact retry
relations:
- serves: vision:O2
- decomposes: epic:model-driven-interpretation
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: WHATS-CHANGED.md
- confidence: cited
  path: changes
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/generate/ess-cli-project/tests/consumer_cli_model_ingress.rs
- confidence: cited
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-diff
- confidence: cited
  path: docs/design
- confidence: cited
  path: fuzz/Cargo.lock
- confidence: cited
  path: models/toolchain
- confidence: cited
  path: schemas/generated
- confidence: cited
  path: website/docs/getting-started.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/reference
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 11
---
## Outcome

Represent and execute truthful exact-result retries for EKR Commit and its
document-only Seed, plus effect-free default refusals on mixed subject-state
commands. The implementation contract is docs/design/retained-command-results.md.
The read-only source and released-compiler evidence is retained in
.engineering/waves/ekr-retained-replay-scope.md.

Existing typed Outcome/ResolvedOutcome, command response and conformance Step
vocabularies are extended. This creates no product domain entity or generic
lookup facility. The adopting EKR project still owns persistent receipt
authority and real restart/head-advance acceptance.

## Acceptance

On ess/7, a named error can be the effect-free default complement selected
against one common existing subject. All finite held states are covered.
Errors still cannot mutate, set fields or emit events. Prior formats retain
their old admission and bytes.

A command-local replays relation names a prior originating creates/moves
success with its typed response. Strict reference and exclusivity validation
refuse cycles, absent/wrong origins and duplicate identity/effect authority.
The subject is observed independently before retry from the original input
or emitted identity, never from the retry response.

Synthesis invokes real original success and retry, captures their actual
results and verifies exact declared response equivalence, no error/events and
full subject preservation. It never configures the expected external outcome.
Correct Rust and Go targets pass; targets returning current head, new time,
an extra event/error or changed subject fail. Snapshot references and aliases
are admitted before callbacks and compared with existing exact typed values.

Native Rust/Go origin and replay outcomes carry the typed result even without
event mapping. All projections, semantic diff and impact account for the
relation. Allocate source7, suite12/coverage13 and diff6; older-envelope
mislabeling refuses. Browser/TypeScript implements or explicitly refuses the
new envelope before callbacks. No ess-ir/2 or open property bag.

Regenerate the source schema, preserve legacy golden bytes, execute targeted
mutations, obtain independent adversarial review and run the complete gate.
Documentation changes also require site-build. Release and adopter pinning
follow verified source integration; local-only success is not delivery.

## Scope

One coupled implementation unit covers domain/IR, conformance/synthesis/native
runners, generated APIs and semantic diff. The detailed read-only report names
current files and proposed new fixtures. Machine-readable scope records the
inspected crate areas, generated schema, design and relevant documentation.
Before editing a new area, confirm it or request an explicit scope addition.
Root alone owns planning, normative design, adoption and publication.

## Coordination

Authorized by the user's instruction to implement and finish EKR, including
its measured upstream dependencies. A four-critic decomposition panel is not
opened: this adds one indivisible implementation unit, not a new multi-unit
decomposition. An independent contract critique and independent source
adversary remain required. No deployment, operator store mutation or source
format activation in EKR is part of this source unit.


## Integer observation boundary

The actual target adapter preserves the handler's exact Integer values before
constructing the typed response, recursively through nested response positions.
Native adapters construct from actual i64/int64 values; JSON adapters must decode
declared Integer exactly or report Unsupported. A generic JSON-to-Node floating
conversion does not establish this contract. Do not add lexical token restrictions
or change legacy numeric serialization.

Certify the supplied adapters with adjacent integers beyond binary64 precision
and both i64 endpoints, nested positions, equal/changed retries and a mutation
that routes the adapter through binary64. Exercise any real adapter serialization
round trip. Keep typed native parity evidence separate from inherited raw JSON
transport probes. The coordinator's initial strict-lexical hypothesis was refuted
by the measured Rust probe; its red Go experiment is not a product fix.

The independent report is retained with this decision; the adopted implementation
contract is docs/design/retained-command-results.md. These new adapter executions
remain required, not claimed complete by this planning revision.

## Release scope

Root coordinates the additive source release after implementation handback and
source review. Cited owners are Cargo.toml (workspace version/internal path
requirements), Cargo.lock and fuzz/Cargo.lock (both derived locks), CHANGELOG.md,
changes/ and its WHATS-CHANGED.md projection, the existing format-release registry
in crates/edge/ess-xtask/src/docs.rs, public reference pages, getting-started.md
and the source support matrix at website/docs/status/where-this-stands.md.
AGENTS.md and Taskfile.yml supply the exact full gate, documentation gate and
release-verification obligations. No task-gate weakening or shared documentation
deployment is included. EKR adoption waits for verified release artifacts.

## Complete source7 refusal observations

The real EKR-shaped external Stale route exposed an inherited weak witness:
ordinary Validate/wrong_state checked a named error and named event absence but
could pass an undeclared direct event or subject mutation. The Go worker retains
its added red controls. For ess/7 generally, ordinary WrongState named-error
witnesses now require complete independently observed held-subject preservation
and no direct events of any name. Missing complete observations refuse synthesis.
Keep ess/1 through ess/6 canonical IR/projection/suite bytes unchanged.

ResolvedOutcome.complete_refusal is compiler-minted for this precise source7
obligation, omitted when false. Synthesis consumes it because the existing IR
does not retain source format. Diff6 records a typed outcome-observation change.
This is no new effect authority, generic metadata bag or IR envelope. The new
semantics are in docs/design/retained-command-results.md and the public format
references, copied into the unit before implementation continues. Existing red
controls must survive unchanged; no filtering or waiver closes them.
