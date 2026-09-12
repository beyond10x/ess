# ESSv2 first half — single language in, single language out: what exists, what does not

Assessed 2026-09-12 against ESS `0.23.0` at `c8023067`, by running the binary and reading the tree.
The subject is the RFC *ESSv2: Multilanguage Synthesis, Semantic Recovery, and Conformance*
(11 September 2026), which is explicitly **proposed architecture, not a statement of shipped ESS
capabilities**. Its own §18 slice table is the sequence this measures against.

*First half* is read here as one turn of the RFC's central pipeline with a single language at each
end: **source → recovered model → admitted model → one target language → build**. Multi-target
mixing (§8), heterogeneous cuts, staged migration and reconciliation (§14) are the second half and
are out of scope below except where a piece of them already exists.

## The verdict in one line

**The out-half is largely built and is honest about its own limits; the in-half does not exist for
any programming language; and the one structural piece missing from both halves is `CodeIr` —
executable meaning.** ESS today is a declaration-driven generator, which is exactly the position
RFC §7 says is insufficient for recovery.

## Out: admitted model → one language → build

| RFC element | State | Evidence |
|---|---|---|
| `EssIr`, system meaning (§3.1) | **present** | `ess specify compile` → canonical typed IR; `source_digest` and `contract_digest` (`crates/generate/ess-gen/src/provenance.rs:23`) |
| One language out | **present, four targets** | `ess generate synthesize --target rust\|go\|web\|clap` (`crates/generate/ess-synth/src/lib.rs:88`) |
| Selected realization planning | **present** | `ess specify realization validate\|compile\|generate`, `ess-realization/1` and `/2` |
| Per-capability planning outcome (§7) | **partial — three, where the RFC wants six** | `SynthesisDisposition::{Generated, Obligation, Refused}` (`crates/generate/ess-synth/src/plan.rs:212`); `ObligationReason::{External, UnspecifiedAlgorithm, ProjectionMaintenance}` |
| Obligations are typed, not holes | **present, and stronger than the RFC assumes** | "Never a hole in a generated file — a typed entry in this document, with the reason it cannot be generated" (`plan.rs:216`) |
| Target cannot carry it → said out loud | **present** | `TargetReport{weakenings, refusals}`; `TargetFailureCode`, ten of them (`crates/generate/ess-synth/src/failure.rs:12`) |
| Plan is language-neutral, refusal is per target | **present** | `RefusalStage::{Planning, Target}`; 16 `CapabilityKind`s |
| Contracts independent of target (§10) | **present** | `--kind docs\|site\|docs-ir\|schema\|openapi\|asyncapi` |
| Conformance derived from intent (§13 row 4) | **present** | `ess verify conform synthesize` → `ess-conformance/4`…`/9`; `run --target` → report `/1`,`/2` |
| Reference interpretation independent of an emitter (§13 row 2) | **present, scenario-scoped** | built-in reference targets `billing`, `oracle-fixture`; the Go target emits runner + predicate evaluator |
| Target compiler check (§13 row 3) | **partial** | emitted Rust/Go compile inside tests; a CI lane for the Go and TypeScript compilers is `story:native-realization-ci`, still `draft` |
| Build, release, deployment (§17) | **present and the strongest area** | `build compile\|graph\|execute`, `component compile`, `release verify\|bundle\|publish\|fetch`, `stack resolve`, `deployment compile\|diff\|reconcile`, `project helm\|kubernetes` |
| Precise numeric semantics (§1 slice 1) | **mostly present** | `ess-primitives` Integer/Decimal/Binary64; `ess/2` authored Binary64. Open: `story:primitive-canonical-serialization` (`draft`) |
| Differential checks between revisions | **present** | `ess verify diff`, `ess verify impact` |
| Generated-output reconciliation (§14, file level) | **present** | output ownership `adopt\|recover`, `--check` drift, transaction decisions modelled in `models/output-ownership/` |
| Effect plans: transaction, retry, exception profile (§11) | **absent for generated systems** | ESS models its *own* execution recovery (`models/execution-recovery/`); a generated system's effect boundary is `external:` plus a binding's `delivery`/`on_failure` |
| `CodeIr` — executable meaning (§3.2) | **absent** | no `CodeIr`/`code_ir` anywhere under `crates/`. `SynthesisPlan` carries capabilities and dispositions, not statements or expressions |
| Language-specific IRs, `RustCodeIr`/`GoCodeIr` (§3.3) | **absent** | emitters render from the plan directly to source |

## In: source → recovered model → admitted model

| RFC element | State | Evidence |
|---|---|---|
| Source-language frontend for any programming language | **absent** | `ess infra import` has exactly two adapters: `kubernetes`, `openapi`. No occurrence of `java` under `crates/` |
| Framework and persistence interpretation (§4.3) | **absent** | — |
| Recovered executable bodies (§4.2) | **absent** | follows from the absence of `CodeIr` |
| Import carries provenance, coverage and gaps | **present in shape — for contracts, not code** | `ess-openapi-import/1` retains exact source with SHA-256 and durable accounting; the adapter contract requires coverage, obligations, unresolved references and refusals |
| Observed and desired kept separate (§4.5) | **present, infrastructure only** | `infra-observation/1` → `infra-ir/1`; `ess verify bindings` compares declared selection against observation and answers `satisfied\|violated\|unknown` |
| Lift recovered facts into `EssIr` (§4.4) | **absent from ESS; a draft-generator exists in AEP** | `ess infra import openapi` produces an import envelope and never an `EssIr`; `aep plan reverse openapi` drafts an `ess/1` domain from an OpenAPI document and "names every decision it could not take rather than omitting it" |
| Characterization vs normative conformance (§13) | **absent** | no separation exists because no source observation is imported |

## What this means for the RFC's slice table

| Slice | Position today |
|---|---|
| 0. Re-establish the baseline | **effectively done** — the workspace, its IRs, emitters, importers and generated fixtures are under gate; `models/toolchain/` now states the toolchain's own surface as an ESS model |
| 1. Shared semantic code kernel | **partly** — precise types, outcomes and a reference evaluator exist; arithmetic, control flow and effect boundaries as an executable kernel do not |
| 2. One source language recovered plus two backends | **half** — two backends exist and are rigorous; no source language is recovered |
| 3. Admitted semantic lifting | **not started** for code; the contract-level analogue exists in `aep plan reverse openapi` |
| 4–8 | not started |

## The finding worth acting on

The out-half's limit is not a missing feature. It is that **behaviour has exactly one source —
declarations** — and the planner is honest about it: what the specification does not determine
becomes an `Obligation` with a reason, never a silent hole. RFC §7 argues that for migration this is
insufficient, because a dropped method body presented as generated scaffolding is not a migration.
Both statements are true at once. The bridge between them is `CodeIr`, and it is the single item
whose absence blocks the whole in-half and the "portable body" and "compatibility runtime"
dispositions of the out-half.

Two smaller items are ready to close without it:

- `SynthesisDisposition` has three cases; §7 names six planning outcomes. The missing four —
  portable body, compatibility runtime, target adapter, retained foreign unit — are meaningful only
  once a body can come from somewhere other than a declaration, so this follows `CodeIr` rather than
  preceding it.
- `story:native-realization-ci` (`draft`) is §13 row 3 for Go and TypeScript, and needs nothing new.

## What was not established

- The RFC was read from a shared conversation rendered client-side; the section headings and the
  §18 slice table were recovered from the page's embedded payload, not from a file in this
  repository. Quotations above are from that recovery.
- No claim is made here about the second half — multi-target plans, heterogeneous realization,
  staged migration, incremental reconciliation.
- `task check` was not re-run for this document; it changes no code.
