---
format: aep.planning-md/3
id: story:served-store-and-entry
kind: story
status: active
title: A served component gets a generated in-memory store and server entry point
refs:
- provider: github
  reference: beyond10x/ess#318
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:go-generated-behaviour
- informed_by: epic:generated-determined-behaviour
- depends_on: story:served-view-params
scope:
- confidence: cited
  path: .github/workflows/planning.yml
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/context.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/entry.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/store.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/context.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/entry.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/single.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/store.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/served-notes
- confidence: cited
  path: crates/generate/ess-synth/tests/json_primitive.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/served_entry.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/single_crate_layout.rs
- confidence: cited
  path: examples/gatepass-go-realization/go.mod
- confidence: cited
  path: generated
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/guides/synthesize.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:53Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":8}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:53Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":8}}}
---
## Outcome

A component reached by network, synthesized in Go or Rust, runs with no hand-written code when its
behaviours are all generated: ESS generates an in-memory store behind the existing storage port and
a server entry point that links the system, serves it, and can serve a generated web app from the
same origin.

## Fit review

1. **Need.** A served Go or Rust component needs a hand-written store and `main` even when every
   behaviour is generated: `examples/gatepass-go-realization/visit.go:30-59` (map store),
   `cmd/gatepass-server/main.go:34-56` (link, `Serve<Component>`, `authenticate`);
   `examples/gatepass-realization/src/bin/gatepass-server.rs:1-7,41`. A generated web app cannot
   call the generated server from another origin: no CORS headers, and the Go `Serve` builds its own
   handler (`ess-synth/src/go/http.rs:950-956`; story:ui-binding-contract fit review, point 4).
2. **Class.** Convenience by ESS's own record (story:go-generated-behaviour fit review, point 2), and
   a reversed decision: "Storage is a port the implementor provides, never a generated store"
   (`.engineering/planning/epic/generated-determined-behaviour.md:29`; `website/docs/guides/synthesize.md:13-14,202,312`;
   `website/docs/concepts/ess.md:203`; the in-memory-store story archived by operator exclusion
   2026-09-15, `story/a-crud-command-carries-its-own-algorithm.md:39-42,50-53`). The operator reversed
   it on 2026-10-01 for components reached by network: "fully synthesize" the example's server.
3. **Already expressible.** Only by hand (the idioms cited in 1).
4. **Fit.** The store implements the generated `<Entity>Storage` port unchanged (Rust
   `behaviour.rs:23-56`; Go per story:go-generated-behaviour), so durable stores stay a port. Every
   port call runs under the server's `serving` mutex (`server/server.go:49`), so a map is safe.
   Authentication is what ESS cannot determine (`synthesize.md:273-274`): the entry point must not
   decide it silently. D-2 (`docs/design/linker-never-chooses.md:16-22`) forbids the linker selecting
   among alternatives; a run-time flag the operator types is a selection by the operator, and the
   default selects nothing.
5. **Second adopter.** A team prototyping any served component (the gatepass example itself; a
   job scheduler's demo) that wants a runnable server from `synthesize` alone.
6. **Cost.** No format change. New generated files per served component (Go `cmd/<component>-server/main.go`
   and a store package; Rust a `src/bin/<component>-server.rs` and a store module); the docs lines
   that state ESS never generates a store are amended.
7. **Designs.** Change nothing: rejected by the operator. A durable generated store: rejected
   (storage technology is the adopter's; the port stays). An entry point that authenticates
   everybody as one actor: rejected (silently grants). CORS headers: rejected (same origin is
   simpler and needs no policy). **Chosen:** in-memory store plus an entry point whose caller
   resolution and static directory are configuration.

## Decisions

**Accept, redesigned (operator decision, 2026-10-01).** Amends
epic:generated-determined-behaviour line 29 for components reached by network only.

- In-memory context: the generated `Context` assigns `Uuid` identities (random v4: Go `crypto/rand`, Rust the `uuid` crate the generated types use) and timestamps from the system clock. A component whose generated behaviours need any other context answer (caller attributes, an `external:` branch, an assigned value of another type) gets an entry point that refuses to start, naming each one.
- In-memory store: one type per entity implementing `<Entity>Storage`, `List` in identity order;
  Go and Rust; generated only for components reached by network; never durable.
- Entry point: Go `cmd/<component>-server/main.go`, Rust `src/bin/<component>-server.rs`; flags
  (Go `flag` package, Rust clap derive) `--listen <addr>`, `--callers <mode>` and `--static <dir>`.
- `--callers none` (the default) authenticates nobody, so every granted command answers 403 not
  granted. `--callers actor-header` reads `Authorization: Actor <name>` and is printed as a
  demonstration mode in the startup record. Why this is not the machinery choosing (D-2, and
  `synthesize.md:273-274` "the server reads no actor from the request itself"): the generated
  `serve` still reads no actor; the entry point passes it an `authenticate` function the operator
  selected by name at run time, and with no flag none is selected. `synthesize.md:273` is amended to
  say so. The 0.49 `authenticate` seam stays for realizations that write their own.
- `--static <dir>` serves files from `<dir>` for every path the served surface does not answer, so
  a generated web app and its server share one origin. No CORS headers.
- Owed commands keep their seams; a component with an owed command gets an entry point that refuses
  to start, naming the obligations, unless a realization links them.
- Docs: `synthesize.md:13-14,202,273,312` and `concepts/ess.md:203` state the generated store, its
  limits and the caller modes.

## Acceptance

Fixture: `crates/generate/ess-synth/tests/fixtures/served-notes/` (new; one served component, one
entity `Note`, `AddNote` creating it with every field from input, `ArchiveNote` a transition, actor `Writer` that `may` run both, a
view `Notes`), whose plan has no obligation.

`crates/generate/ess-synth/tests/served_entry.rs`:
- `the_served_notes_plan_has_no_obligation`
- `the_go_entry_point_serves_the_fixture_with_no_hand_written_code` (build, start on
  `127.0.0.1:0` with `--callers actor-header` and `Authorization: Actor Writer`: `AddNote` 202, `Notes` lists it; as `Actor Reader` (undeclared): 403 not granted; with no
  `--callers`: `AddNote` 403 not granted)
- `the_rust_entry_point_serves_the_fixture_with_no_hand_written_code` (the same, Rust)
- `the_static_directory_is_served_beside_the_api` (Go and Rust: `GET /` answers `index.html`,
  `GET /notes/views/notes` answers rows)
- `an_entry_point_with_owed_commands_refuses_to_start_naming_them` (gatepass: names
  `gatepass.visit.RegisterVisit`, `generated/go/gatepass/PLAN.md:59`)
- `the_store_lists_in_identity_order`
- `the_generated_context_assigns_distinct_uuids`
- `an_entry_point_needing_caller_attributes_refuses_to_start_naming_them`
- `the_fixture_suite_passes_against_the_generated_go_and_rust_servers` (the synthesized
  conformance suite over HTTP)

Go tests return early without a toolchain (`examples/gatepass-realization/tests/conformance.rs:853-856`);
done needs the PR's CI run, whose `test` job has Go 1.25.10, showing these tests ran.

## Scope

`crates/generate/ess-synth/src/go/{store.rs, context.rs, entry.rs (new), layout.rs, mod.rs, http.rs}`,
`crates/generate/ess-synth/src/rust/{store.rs, context.rs, entry.rs (new), layout.rs, mod.rs, http.rs}` (`http.rs`: `--static` needs a fallback route, since Go `Serve` builds its own handler, `go/http.rs:950-956`, and Rust `serve_function` `rust/http.rs:596`),
`crates/generate/ess-synth/tests/served_entry.rs` (new),
`crates/generate/ess-synth/tests/fixtures/served-notes/` (new), `generated/{go,rust}/{gatepass,billing}`,
`website/docs/guides/synthesize.md`, `website/docs/concepts/ess.md`.

## Sequencing

After story:go-generated-behaviour (the Go store implements its ports) and story:served-view-params (both edit `{go,rust}/http.rs`). Before
story:related-guard-behaviour: both regenerate `generated/` and edit `synthesize.md`.
`CHANGELOG.md` is a merge-time edit (epic).

Recovery scope check (2026-10-02): the implementor read generated/rust/gatepass/crates/gatepass-types/src/primitives.rs:33 and found Uuid is a local string newtype; the earlier Decisions phrase "the uuid crate the generated types use" was an incorrect assumption. The entry runtime may use the uuid crate to mint v4 values and construct the existing wrapper, preserving its public representation. crates/generate/ess-synth/src/go/behaviour.rs:166 and rust/behaviour.rs:177 already collect the actual Context and storage uses privately. Reuse that metadata for store/context generation; both files are now cited typed scope. rust/single.rs is inferred scope until the generated entry's single-crate relayout is verified. No behavioral semantics are delegated to a new traversal. Concurrent PR #387 edits the emitters and generated fixtures; reconcile its eventual merge before publishing this unit.

Runtime dependency correction (2026-10-02): go vet on the generated consumer found os.OpenRoot requires Go 1.24 while the generated go.mod still declared 1.21. Network-server output now declares Go 1.24 so rooted static-file access retains filesystem confinement; output without a served network component stays on Go 1.21. The coordinator accepted this bounded dependency change; documentation and the merge-time changelog name it. Local and CI Go toolchains exceed the minimum. acceptance-2.log preserves the failing stdversion check, and the corrected run must pass go vet as well as build and runtime scenarios.

Confirmed scope corrections (2026-10-02): rust/single.rs is now cited, exercised by the generated single-crate server. Existing tests declared_behaviour.rs, json_primitive.rs and single_crate_layout.rs replace obsolete exact no-store/empty-dependency assertions with positive optional-memory and dependency-boundary checks. Root Cargo.lock records the added optional runtime dependencies; time is pinned to 0.3.41 for generated Rust 1.85 compatibility, requiring its transitive lock entries to move from the prior root time 0.3.55 selection. examples/gatepass-go-realization/go.mod raises its minimum from 1.21 to 1.24 to consume the newly generated served module; offline tidy changes only that directive. Go storage is emitted in types/behaviour/memory.go, using the existing behavior package rather than adding a distinct store package; ports remain unchanged. The coordinator accepted these concrete scope corrections. No plan or capability classification changes in the regenerated fixtures.
