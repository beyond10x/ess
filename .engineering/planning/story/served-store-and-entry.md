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
  path: Cargo.lock
- confidence: cited
  path: crates/generate/ess-synth/Cargo.toml
- confidence: cited
  path: crates/generate/ess-synth/src/feasibility.rs
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
  path: crates/generate/ess-synth/src/go/store/identity.rs
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
  path: crates/generate/ess-synth/src/rust/store/identity.rs
- confidence: cited
  path: crates/generate/ess-synth/src/served.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/served-notes
- confidence: cited
  path: crates/generate/ess-synth/tests/served_entry.rs
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

# #318: read-only implementation outline

Authority: canonical `story:served-store-and-entry`, revision 8, complete accepted decisions and all nine named scenarios. Source inspected at server candidate `4826099161bec53d2da65996958b97cb0c96165a`. No implementation, fixture edit or planning mutation in this pass. The follow-on implementation needs its own managed tree and lease.

## Work sequence

1. Add the accepted served-notes specification and a Rust integration harness. Generate both language trees into test scratch and build/start their emitted binaries without injecting business code or a handwritten store. Capture the startup record to discover a port bound at `127.0.0.1:0`. Establish meaningful red for the no-code server, default denial, explicit actor-header mode, static/API coexistence and conformance run before changing emission.
2. Derive generated-store and context requirements from the existing behavior/storage port seam. Emit deterministic in-memory storage implementations and a composed ports value that delegates those exact traits/interfaces. Use typed model/layout information; avoid parsing generated source text or inventing a second definition of which behavior is generated. Context supplies random v4 UUIDs and system-clock timestamps only; other assignments, caller attributes and external answers remain named startup refusals. Emit this convenience only for network-reached components.
3. Emit each component's executable and connect it to the existing generated system and HTTP surface. Rust command lines use clap derive; generated Go uses the story's flag-based output, emitted and exercised by Rust repository code. Keep the existing public `serve`/`Serve*` authentication callback usable by handwritten realizations. Add explicit entry-point configuration for listen address, caller mode and static root. Startup must reject unmet behavior/context obligations before announcing readiness. API routes, grant failures and unsupported operations must keep their existing behavior when static serving is enabled.
4. Prove every accepted case in both languages, then run package and projection verification. Refresh committed trees through the now-documented settled-reference/adoption workflow. Amend the stated documentation claims about generated stores, caller resolution and nondurability. Regeneration and any scope extension are reviewed before commit; no reduced version of the acceptance is proposed.

## Acceptance coverage

| Named acceptance | Decisive observation |
|---|---|
| `the_served_notes_plan_has_no_obligation` | Actual compiled fixture plan has zero owed capabilities; target-independent plan stays consistent. |
| `the_go_entry_point_serves_the_fixture_with_no_hand_written_code` | Build untouched Go emission, start it, AddNote returns 202, Notes includes the created row, undeclared Reader gets 403, omitted caller mode gets 403. |
| `the_rust_entry_point_serves_the_fixture_with_no_hand_written_code` | Same process-level behavior for untouched Rust emission. |
| `the_static_directory_is_served_beside_the_api` | Both targets return index.html at `/` and correct view rows at the declared API route; API refusal statuses do not turn into static responses. |
| `an_entry_point_with_owed_commands_refuses_to_start_naming_them` | Gatepass fails startup naming RegisterVisit and all other relevant owed capabilities; handwritten linkage remains possible through the existing public seams. |
| `the_store_lists_in_identity_order` | Insert out of order, replace/delete rows, and observe deterministic identity order in both generated stores. |
| `the_generated_context_assigns_distinct_uuids` | Actual emitted context produces distinct valid v4 UUIDs in both targets. System-clock timestamp behavior also needs a real bounded observation because it is an explicit decision. |
| `an_entry_point_needing_caller_attributes_refuses_to_start_naming_them` | Before ready/listen, deterministic diagnostics identify the actual unsupported attribute; external and unsupported assigned types receive equivalent explicit evidence. |
| `the_fixture_suite_passes_against_the_generated_go_and_rust_servers` | Synthesized, admitted suite executes through HTTP with strict Passed statuses and nonzero expected case counts; no injected handwritten behavior. |

Toolchains are present locally, so required Go evidence must execute rather than early-return. The canonical CI requirement must also be verified when the follow-on group is published.

## Scope and acceptance risks to resolve during implementation

- **Rust dependency premise is stale.** `rust/mod.rs:402/410` emits Timestamp/Uuid as String wrappers; generated manifests currently have zero external dependencies (`rust/http.rs:247`, `rust/single.rs:228`). The story's claim that the generated types already use the uuid crate is inaccurate. Its explicit UUID-v4 and clap decisions still stand, requiring deliberate server/runtime dependencies and offline build evidence. Do not replace random UUIDs with sample/counter strings to preserve the old dependency claim.
- **Single-crate layout must be handled.** `rust/single.rs:140–202` drops workspace manifests and places every server file below `src/server/`; its manifest at :228 has no dependencies. New executable paths and dependencies need folding/feature treatment here, although single.rs is absent from the current cited scope. This is a necessary scope refinement rather than permission to break an admitted layout.
- **Context requirements are currently global.** `rust/behaviour.rs:59–130` collects `Uses` over all model commands/views; `Uses` at :177 includes caller, assigned and external requirements. A component's startup cannot silently invent answers to unrelated global methods or falsely refuse a runnable component because of an unrelated sibling. Requirement accounting must include behavior reachable through bindings, not only its direct HTTP routes.
- **Nominal identity wrappers lack Ord.** Generated Uuid primitives derive Ord, but nominal newtypes such as `EmployeeId` derive only equality. A store cannot blindly use the public newtype as a BTreeMap key or impose new derives on every domain type. Preserve the port's types and choose an internal typed ordering representation; integer identity order must not become lexicographic numeric-string order.
- **Authentication spelling needs an explicit choice consistent with acceptance.** The accepted fixture sends `Authorization: Actor Writer`; existing realization examples compare qualified declared names (`gatepass-server.rs:authenticate`). Do not accidentally make the required short-name case fail, accept arbitrary undeclared names, or choose between ambiguous aliases. Default `none` must never read a granting header.
- **Startup is part of the contract.** Existing bound-port/startup facts are shared by Rust and Go (`rust/http.rs:460–551`, Go :957–984). Include the explicitly selected demonstration caller mode without emitting ready on a refused entry point. Preserve genuine unmet obligations rather than satisfying them with placeholder return values.
- **Static routing belongs after route selection.** Existing Go Serve creates its own handler; Rust serve owns its listener loop. Add fallback without changing current callback signatures/behavior for realizations that do not opt in. Preserve known-route method, auth and decoding errors. Confine file resolution to the selected static root, including encoded paths and filesystem aliases.
- **Generated names and package coverage matter.** New store/context/binary paths need the existing feasibility allocation checks, including collisions with authored component/module names. Existing no-server specifications should not gain server stores/binaries; generated HTTP/WASM consumers and no-feature single-crate builds must still compile.
- **Generated claims also need amendment.** Storage trait prose currently says ESS never emits an implementation (`rust/behaviour.rs:250`), beyond the website lines named in the story. Update only claims the accepted network-served exception changes. Keep durability, authentication and unresolved behavior responsibility explicit.

No caller policy, durable persistence, CORS policy, broader related-guard generation or general symbolic/context solver is added by this story.

## Identity ordering implementation refinement

The compiler does not restrict entity identity to scalar types. The generated store must not silently fall back to debug/text sorting for admitted structural identities or round numeric identities through binary floats. The worker is implementing typed structural keys with matching Rust/Go meaning and scalar/structural regressions, preserving port signatures and existing semantic equality. Map/set keys must be canonical independently of insertion order. Exact numeric exponent/value comparison uses existing locked num-bigint0.4.8 in generated Rust (optional with the single-crate server feature) and an ess-synth dev-dependency edge; Go uses math/big. No new dependency version is authorized. End-to-end HTTP tests must cover value-equivalent numeric spellings, negative zero and exponents; document bounded parsing and ordering. This is an explicit store scope refinement, not new planning/proof or authored language support.

## Identity contract correction after source inspection

The earlier proposed numeric-key canonicalization was an unverified coordinator inference and is withdrawn. Existing generated Rust explicitly defines Decimal equality AND order over rendering (rust/mod.rs:395-399) and Json number spelling/object order as representation identity (rust/json.rs:793-798); Go preserves the same comparability (go/mod.rs:694-703, go/json.rs:5-15). Existing store harnesses use typed equality, with no overriding canonical identity rule found.

Preserve those contracts in the new store: Integer numeric order; Decimal and textual scalar rendering order; transparent wrappers; records by declared field structure; ordered lists; map/set canonical ordering only where existing generated equality ignores insertion order; Json number spelling and object member order retained. Numeric-equivalent Decimal spellings such as1 and1.0 remain distinct identities as the generated type documents. No num-bigint dependency is needed for this design; remove only newly added direct edges from the abandoned proposal. Add exact representation-distinction and lookup/order controls through the real HTTP path. This corrects the earlier assumption before publication rather than silently changing row-address equality.

## MemoryPorts identity boundary after preparatory review

The generated MemoryPorts identity contract is equality of decoded wire values, preserving Decimal rendering and Json numeric spelling and object-member order. It does not promise universal Go native == preservation: pointer-bearing Optional values and collections already make that promise incoherent across targets. Existing primitive types, constructors and equality remain unchanged.

In particular Go Json retains raw constructor document text, whereas Rust Json parses a tree. Direct Go constructors for [1] and [ 1 ], or zero Json{} and NewJson("null"), can differ under native equality but normalize to the same MemoryPorts key. Generated HTTP cannot produce the whitespace distinction after decoding. Document and test this direct-port boundary explicitly, alongside HTTP numeric/member-order controls; do not silently claim the representations are identical. This narrows the revision 13 preservation statement for the new store only. Preparatory reviewer server_corrections performed source inspection, zero test/build executions; frozen implementation still needs independent review.

## Preserve the no-panic guarantee for generated context

The model-global Context port requires infallible methods even for context used only by unrelated components. MemoryPorts cannot truthfully implement unsupported answers, and a panic or fabricated neutral value violates the existing generated-stub contract. The initial local prototype is not accepted for publication on that basis.

Accept an additive fallible companion seam: Rust retains the existing Context and supplies a blanket adapter into TryContext; existing realizations continue to compile unchanged. MemoryPorts implements the fallible seam with named UnmetObligation errors. Go retains its existing Context surface and adds a fallible companion path with explicit precedence; legacy implementations continue to work. Generated behavior propagates the existing typed obligation error rather than panicking. Do not replace existing method signatures or weaken the no-panic tests. Delegate unsupported command/view obligations to existing generated stubs instead of duplicating them.

Regression controls must establish legacy Context source compatibility, direct MemoryPorts refusal for unsupported caller/assignment/external answers, unchanged component-specific startup refusal/reachability, and error propagation without partial storage mutation or emitted success events. Audit each context call relative to writes before choosing placement: a fallible value must be obtained before committing effects that would otherwise survive its error. No new syntax or format is authorized. New generated names must use existing collision allocation, with a collision fixture. Record exact touched files and changes before the independent frozen review.

## Total identity for direct invalid Go Json constructors

Go NewJson can retain malformed document text even though generated HTTP rejects it. Storage ports have no error return, so a generated identity helper must remain total without panicking. Accept a separate InvalidRaw identity variant, ordered deterministically by original text and distinct from every valid decoded Json key. Use it when decoding or internal decoded-value conversion cannot produce a supported valid key. It must never silently become null, an empty key or a Debug/map rendering. Identical invalid raw text compares equal; different raw text remains distinct. Existing Json constructors, native equality and HTTP validation stay unchanged. Add direct-store controls for replacement, distinction and no-panic behavior; decoded-wire identity rules still govern valid inputs.
