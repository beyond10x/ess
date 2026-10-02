---
format: aep.planning-md/3
id: story:go-generated-behaviour
kind: story
status: implemented
title: The Go target generates determined behaviours, view queries and invariant checks at parity with Rust
refs:
- provider: github
  reference: beyond10x/ess#314
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:feature-request-310
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/entity.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/obligation.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: examples/gatepass-go-realization
- confidence: cited
  path: generated/go
- confidence: cited
  path: website/docs/guides/synthesize.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:52Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:52Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-02T04:46:41Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":14}}}
---
## Outcome

The Go target generates every command behaviour, view query and entity invariant check the plan marks generated, at parity with the Rust target, behind the same storage and context ports.

## Fit review

Read at `origin/main` `f86180f30`. No `ess` or `go` command run; idioms shown from committed files and tests.

1. **Need.** A Go adopter serving a component gets only owed seams for capabilities the plan marks generated; the stubs refuse (`generated/go/gatepass/types/visit/visit.go:668-698`). Reproduction, `examples/gatepass`: the plan marks AdmitVisitor, SignOutVisitor, ExpectedVisits and VisitById generated (`generated/go/gatepass/PLAN.md:28,31,38,40`); Go keeps them owed (`generated/go/gatepass/TARGET.md:22-23`); `examples/gatepass-go-realization/visit.go:116-209` hand-writes them while Rust generates them (`generated/rust/gatepass/crates/gatepass-types/src/behaviour.rs:58-158`). Requester's words: behaviours, view queries including aggregates, invariant checks, "an in-memory store behind the existing storage port, and a `main` for each component reached by network". Corrections: Rust generates no store (`behaviour.rs:9-10`) and no `main` (`examples/gatepass-realization/src/bin/gatepass-server.rs:1-7,41`); the gatepass suite has 17 scenarios (`examples/gatepass-realization/tests/conformance.rs:69`); RegisterVisit stays owed (`PLAN.md:59`). For the requester's model, a `when_related` guard keeps the command a whole obligation in every target (`crates/generate/ess-synth/src/determined.rs:147`, `website/docs/guides/synthesize.md:223`).
2. **Class.** Behaviours, queries, invariant checks: a gap in the Go target, a documented weakening that says "yet" (`src/go/mod.rs:587-610`; `synthesize.md:229-231,260-262`); not a deliberate design (the 2026-09-29 decision scoped implementation to `--target rust`: `.engineering/planning/epic/generated-determined-behaviour.md:17-29`, `story/generated-behaviour-for-declared-commands.md:18`; commit `2066a0649`). Unlisted gap: Go emits no invariant check (`src/go/items.rs:603-604`), Rust emits `broken_invariant()` (`generated/rust/gatepass/crates/gatepass-types/src/visit.rs:124`). Store: local policy, already decided by the operator: "Storage is a port the implementor provides, never a generated store" (`epic/generated-determined-behaviour.md:29`; `synthesize.md:13-14,202,312`; `website/docs/concepts/ess.md:203`; the in-memory-store story archived by operator exclusion 2026-09-15, `story/a-crud-command-carries-its-own-algorithm.md:39-42,50-53`). `main`: convenience (~25 lines, `examples/gatepass-go-realization/cmd/gatepass-server/main.go:34-56`).
3. **Already expressible.** Behaviours only by hand. Store idiom: a map-backed type implementing the port with a stable `List` order (`examples/gatepass-go-realization/visit.go:30-59`, `website/docs/start/runners/go.md:109-116`). `main` idiom: `Link()` then `Serve<Component>(system, "127.0.0.1:"+PORT, authenticate)` (`main.go:42-56`, exercised by `conformance.rs:1023-1048`).
4. **Fit.** Reuse Rust's names: `Generated`, `<Entity>Storage` (get/put/delete/list), `Context` (`behaviour.rs:23-56`; `synthesize.md:197-206`). The plan is unchanged and byte-identical across targets (`crates/generate/ess-synth/tests/go.rs:134`). The bundle plugs into `passservice.New(Behaviors)` unchanged (`generated/go/gatepass/components/passservice/passservice.go:25-31,79`); grants are checked before the port (`server/passservice.go:134`); every port call runs under the `serving` mutex (`server/server.go:49`; `server/passservice.go:117-118`), so a map store is safe (race test `tests/adversary_served_pass1_gatepass.rs:7-8`). Siblings: Web is a bridge over Rust, Clap handlers stay obligations (`synthesize.md:51-52`), TypeScript is not a synthesis target (`synthesize.md:20`), the interpreter is the reference (`determined.rs:1-8`); no authored surface, so verify diff, the entity runtime and authoring are unaffected. The bijection check stays; only its `weakened` carve-out goes (`src/go/mod.rs:432-444`). Hazards: Go maps have no order, so `List` must require a stable order (`behaviour.rs:36-37`); caller attributes never reach `Context` from the served surface (`server/server.go:162-164`), and Go's `authenticate` runs outside the lock (`server/passservice.go:95` vs `:117`).
5. **Second adopter.** billing's Go tree (40 generated capabilities, `synthesize.md:41`); a brand-free Go stock-reservation service whose every line is determined today.
6. **Cost.** No format bump, keyword, diagnostic, migration or diff classification. Go generated API: a new `types/behaviour` package; `BrokenInvariant()` on entity data types; `<ctx>.Unimplemented` stops implementing generated seams (compile error for code passing it as the whole bundle); Go adopts Rust's invariant preflight (`src/rust/invariant.rs:41-60`); the package name `behaviour` is reserved (`src/go/layout.rs:169-181`). `generated/go/{billing,gatepass}` regenerate; gatepass TARGET.md goes from 7 rows to 5. Decimal sum/average need `math/big` (standard library only, `synthesize.md:68`).
7. **Designs.** A change nothing: rejected. B as proposed (store and `main` included): rejected for store and `main` (the store reverses the 2026-09-29 decision and makes Go exceed Rust; a generated `main` must choose an authentication ESS cannot determine, `synthesize.md:273-274`, or import the adopter by convention). **C exact Rust parity, ports only: chosen.** D partial: fine only as a story split. The store and `main` part of B is story:served-store-and-entry.

## Decisions

**Accept, redesigned.** Go emits every CommandBehavior and ViewQuery the plan marks generated; a construct Go cannot spell is a named target refusal. Package `types/behaviour`: `<Entity>Storage` (`Get`, `Put`, `Delete`, `List` in a stable order), `Context` (only the methods the model asks: caller attributes, `Generate<Qualified>()`, `External(command, outcome string) bool`, as Rust's `tests/fixtures/declared-behaviour-harness/main.rs:81-107`), `Ports`, `func New(ports Ports) *Generated` satisfying every component's `Behaviors`; Rust's evaluation order (`behaviour.rs:13-30`). Invariants: `func (d <E>Data) BrokenInvariant() (string, bool)`, refused as `UnmetObligation{Capability: "entity invariant"}`. `Unimplemented` covers owed seams only; the two TARGET.md rows go; server, routes, OpenAPI, startup record and `authenticate` unchanged. **Store and `main`: not in this story; story:served-store-and-entry generates both (operator decision, 2026-10-01).** Still owed: everything `determined.rs` keeps owed (`when_related` incl. its `state`, `{related:}`, existence selection until #310, `instances:`/`affects:`, typed responses), views with parameters or paging, context, authentication; storage and `main` until story:served-store-and-entry. Witness: the generated suite unchanged; parity shown by running it against generated Go through the port and over a socket.

## Acceptance

- `crates/generate/ess-synth/tests/declared_behaviour.rs::the_go_behaviours_build_vet_clean_and_pass_their_own_suite` (new harness `tests/fixtures/declared-behaviour-go-harness/main.go`, in-memory ports, no behaviour; `port` and `served` over real HTTP)
- `declared_behaviour.rs::the_go_target_generates_every_behaviour_the_plan_marks_generated` (replaces `:552`)
- `tests/generated_view_queries.rs::the_go_queries_build_and_pass_their_own_suite` (aggregates included)
- `tests/invariant_check.rs::every_invariant_form_is_checked_by_the_go_data_type`, `::a_go_model_without_invariants_emits_no_check`
- `examples/gatepass-realization/tests/conformance.rs::the_committed_suite_passes_the_linked_go_realization_including_unknown_instances` stays 17/17 with `visit.go` holding only RegisterVisit and the store; new `::the_go_linker_owes_exactly_the_plans_obligations`
- Stay green: `tests/adversary_served_pass1_gatepass.rs::rust_and_go_gatepass_servers_answer_every_command_identically_and_to_their_contract` (race build), `tests/go.rs::the_plan_is_byte_identical_in_both_targets_trees`. Go tests return early without a toolchain (`conformance.rs:853-856`), so done needs the PR's CI run, whose `test` job has Go 1.25.10, showing these tests ran (not skipped).

## Scope

Cited: `src/go/mod.rs` (432-467, 587-610), `src/go/obligation.rs` (252, 296), `src/go/items.rs:595-607`, `src/go/entity.rs:55`, `src/go/layout.rs:161-181`, `tests/declared_behaviour.rs:552`, `examples/gatepass-go-realization/{visit.go,linker.go}`, `generated/go/{billing,gatepass}`, `synthesize.md:56,229-231,260-262`. Inferred: new `src/go/behaviour.rs`, `src/go/behaviour/query.rs`, `src/go/invariant.rs`, the Go harness fixtures. Must not change: `determined.rs`, `view_query.rs`, `existence.rs`, `plan.rs`, `src/rust/*`, `generated/rust/*`, every PLAN.md and plan.json, the Go server contract, `ess-conformance`.

## Sequencing

#310: semantic overlap (reads `determined.rs`, `existence.rs`, ports `rust/behaviour.rs`, all changed by #310): land after #310 and port its existence selection (lift `src/go/mod.rs:292`). #287: no file overlap; re-check suite counts. #306, #272: none. `CHANGELOG.md` (Breaking) is a merge-time edit (epic).

## Issue completion boundary 2026-10-02

PR386 at f0b220099 carries this story's behavior/query/invariant implementation and served-view parameters, with local corrections verified. It does not yet implement generated in-memory stores or entry points. Issue314's full stated acceptance includes those features, now tracked by served-store-and-entry / issue318. The PR body therefore uses Refs #314, not an automatic closure directive. Keep the full consumer request open until both portions are delivered; splitting the plan is not permission to narrow completion.
