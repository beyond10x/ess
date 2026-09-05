# Rust target feasibility before emission

Status: binding implementation decision, 2026-09-05, for story:review-rust-target-feasibility.
Source basis: ESS6616b26fe41548af9cb7ff9cf833ae977883f625; implementation opening4b66aac7b608b1deee9de88942390d4a6c5ec745. This document defines required behavior and verification; it does not claim the checks have run.

The coordinator adopted the independently scoped proposal under the operator's standing remediation authorization. Existing Synthesis, TargetReport, TargetRefusal, Capability and resolved types retain their meaning. A separately versioned TargetFailure describes complete emission failure, including causes without a planned capability; no new ESS domain entity is introduced. Atlas story:ess-rust-target-refusal-migration and ADR0037 coordinate the public Rust API and known SDK reader, whose separate story:reject-ess-rust-target-refusals preserves its current ESS pin.

## Decision

Retain the neutral plan and successful generated rendering. Check actual emitted scopes and representations before rendering; a fatal target failure returns no Synthesis or code artifacts.

The required measurement established that an empty compiler-admitted domain demo.lib has plan.capabilities=[] and panics at duplicate crates/demo-types/src/lib.rs on the unchanged producer. Exactly1 regression executed and failed (exit101) at opening4b66aac. Its source digest is717485cb4659225ae39ac57e64510a3209acefffc09220c77bbafbe14359de5f. A per-capability TargetRefusal cannot encode this failure honestly, so complete emission failure has a checked result independent of capability accounting.

Required signatures (module-qualified notation):

~~~rust
pub fn rust::workspace(ir: &EssIr, plan: &SynthesisPlan)
    -> Result<Vec<Artifact>, TargetFailure>;
pub fn synthesize(ir: &EssIr) -> Result<Synthesis, TargetFailure>;
pub fn synthesize_for(ir: &EssIr, target: Target)
    -> Result<Synthesis, TargetFailure>;
pub fn web::workspace(ir: &EssIr, plan: &SynthesisPlan)
    -> Result<web::Emission, TargetFailure>;
~~~

No public unchecked wrapper retains the source-driven panic path. Synthesis, TargetReport and web::Emission successful values retain their fields and meanings. Rust success is Ok with target=None and no TARGET metadata. Go/Clap and historical partial Web reports remain existing successful results inside Ok. A whole Rust/Web failure is Err, never a successful diagnostic-only artifact tree.

This is an explicit source API migration. The source scan finds only the assigned CLI outside ess-synth, plus the known external SDK facade reader. SDK admits old direct and new Result forms through a private typed conversion before pin adoption; no dependency/source version sniffing is needed. Unknown external callers must handle the checked call on upgrade.

Preserve the existing successful allocation/naming algorithm and validate its final results. Do not rename valid APIs, box values or restrict source-language validity. A fabricated mismatched caller-supplied plan remains outside the compiler-admitted-input guarantee.

## Whole-workspace refusal accounting

A fatal workspace failure is represented by an invariant-preserving TargetFailure with private fields/read-only accessors, Display and std::error::Error. It is Send+Sync and carries original source/cause information through SDK errors. Internal construction guarantees nonempty deterministically ordered causes.

Its new Serialize-only envelope has exactly these fields:

~~~text
format: ess-target-failure/1
target: rust | web
plan: the unchanged SynthesisPlan for the input
causes:
  - code: a closed TargetFailureCode
    sources: nonempty actual source identities
    detail: actionable human-readable cause
~~~

The closed initial codes are invalid-identifier, symbol-collision, path-collision, recursive-layout, binding-assignment, missing-type-owner, wire-collision and missing-representation. Use actual qualified source identities, including system/domain owners for global layout, not invented capabilities. Sort/deduplicate causes and source identities deterministically. The nested plan preserves provenance, scope, complete capabilities and dispositions as the exact unchanged typed value, including an empty capability list.

The envelope uses explicit Rust data, not a generic JSON bag. Canonical JSON is typed pretty JSON plus one newline; YAML presents the same fields. No persisted reader/admission API is introduced or implied. Callers consume the typed error or documented versioned output. Err with nonempty causes establishes complete failure even with zero capabilities.

No code or manifest vector is returned on Err and no existing partial TargetReport is rewritten. The nested plan explains the intended neutral contract. Do not invent TargetRefusal capabilities or reclassify neutral obligations/refusals. Successful coverage/stub assertions remain. Source-driven causes may accumulate; internal programming defects are not fabricated source refusals.

## One allocation inventory, actual scopes

Build a checked allocation value containing the existing names, paths, scope identities and source owners. Reuse the current normalization and suffix/fallback algorithms verbatim for admitted inputs; check their final results, not just initial candidates. Pass that same value into rendering so checking and emission cannot allocate different names. The scope key must represent Rust's applicable namespace and owner, rather than a global blacklist. Independent modules may retain identical names.

| Scope | Minimum inventory and checks | Current owner |
| --- | --- | --- |
| Workspace packages and Rust crate identifiers | Types/system/server and all component packages; existing suffix repair; package-to-crate hyphen replacement; legal final identifiers and collisions in dependency use scopes. Reserve names that the current allocator already reserves, even if that package is not emitted. | `rust/layout.rs:43–50,114–117,256–275` |
| Artifact paths and module declarations | Root manifest, each package manifest/lib, domain modules, primitives/obligation helpers, server lib/http/json/wire and per-component HTTP modules; exact duplicate/file-directory conflicts; final module identifier versus the actual filename rustc resolves. A legal filesystem filename is not proof of a legal Rust module mapping. | `rust/layout.rs:145–147,218–253`; `rust/mod.rs:198–220,239–283`; `rust/http.rs` |
| Domain type/value/module namespaces | All named types, events/errors/views, command input and generated Outcome names, entity plus Data/Snapshot/Any forms and state module/enum, existing obligations module. Include implicit tuple/unit struct constructors in the appropriate value namespace. | `rust/items.rs:24–32,53–55,108–112,143–151,160–190`; `rust/entity.rs:46–50,58–144,164–211,270–333` |
| Fields, variants and inherent methods | Struct/command/event/error/view fields; enum/union/state/outcome variants; each actual typestate impl's transition methods beside new/state/data/into_data/refine/snapshot. Methods on disjoint state instantiations are not automatically a collision. | `rust/items.rs`; `rust/entity.rs:185–239,304–333` |
| Port scopes | Port type and generic parameter, PublishedEvent, constructor/outbox helpers, command handlers and queries; normalized accepted commands from different domains can share one method scope. Preserve legal same names in different components. | `rust/port.rs:116–172,214–327` |
| System and obligation scopes | System/SystemEvent/BindingInvocation, binding functions/variants, transformation/escalation traits and stubs, component generic names and fields beside obligations/invocations/published/cursor/retries; conversion/behavior/query trait fragments and helper UnmetObligation/Unimplemented. | `rust/system.rs:187–239,271–340,368–499,505–674`; `rust/obligation.rs:34–136,151–275` |
| Post-repair event names | Repeated outcome event fields after numbering, including collision with the fixed error field; event variant fallback to full qualified names must itself be unique. | `rust/items.rs:208–259`; `rust/mod.rs:158–192` |
| Name resolution of generated helpers | Bare String/Option/Vec/Result and primitives, core/std paths and imported helper modules must resolve to what the renderer means in that concrete scope. Refuse a shadow only where it changes emitted meaning or makes resolution fail; do not reject every source declaration named String or Result everywhere. | `rust/layout.rs:170–203,287–297`; obligation/system/wire renderers |
| Wire scopes where codecs are emitted | Full-name-derived codec functions after normalization; effective JSON field names and the adjacent union tag/content keys; fixed outcome envelope keys in their own nested objects. Rust member identity and wire identity are separate checks. | `rust/wire.rs:49–88,124–161,319–453`; `ess-gen/src/schema.rs:64–81` |

Examples that must be measured include FooBar/Foo_Bar; self/self_ fields; domain primitives/primitives_domain after reserved suffix repair; domain lib colliding with lib.rs; a keyword module whose token is `r#type` but whose current path includes that token; a type named Self; helper shadowing only in scopes that use that helper; generated FooOutcome versus a declared type of that name; repeated event fields ending in an already-used `_2`; a refusal outcome whose emitted event field becomes `error`; binding/component normalized names; and global wire fragments that flatten distinct qualified names identically. Neither a source name being legal nor one helper promising collision freedom settles the final scope.

Reuse `ess_gen::schema::wire_field_name` and `union_content_key` as existing authorities. Do not rename source/wire fields or edit ess-gen to accommodate Rust. Refuse ambiguous effective wire identity when a generated Rust HTTP/Web codec would carry it. A duplicate source wire annotation in a model emitted as pure semantic Rust types without codecs is not itself a Rust compile failure; the target check must state which promised wire surface makes it a refusal. This avoids an unrelated universal source restriction.

There is no `ResolvedBody::Alias` in the current emitter: the variants are Newtype, Struct, Enum and Union (`rust/items.rs:24–32`). Do not claim an alias renderer exists or add a new source construct. Newtype chains and generated entity/state wrappers still belong to the representation/name inventory.

## Minimum representation refusal policy

**Recursive layout.** Refuse any cycle in the graph of actual by-value generated representation dependencies. A named reference, struct field, tuple-newtype payload, or union payload contributes an edge; Optional preserves that size edge. List/Vec and Map/BTreeMap break it. Include generated representation wrappers where they contain source values; keep the graph keyed by resolved identity, not normalized text. Report a deterministic self or mutual cycle path, including the relevant field/variant. Do not treat all semantic reference cycles as size cycles, and do not reject collection-mediated recursion or acyclic nested optionals. Current `rust/layout.rs:181–187,196–202` and `rust/items.rs:53–55,143–151` show why Optional is not sufficient indirection.

Choose refusal, not automatic `Box`, for this unit. Boxing would alter constructors, field types, pattern matching, conversions and codecs together; that is a separate representation/API migration. The acceptance expressly permits explicit refusal for valid but infeasible source models. A domain/compiler prohibition would erase source-language freedom and is excluded.

**Optional binding assignments.** The compiler intentionally admits a required source into one or more optional target wrappers (`ess-domain/src/types.rs:886–900`; `ess-compiler/src/resolve.rs:2630–2644`). The neutral planner treats a no-conversion event-field mapping as Copy (`plan.rs:984–988`), and the Rust transformation emits only `event.field.clone()` (`rust/system.rs:309–317`). That expression is not the required `Some(...)` lift when source and target types differ. For the minimum refusal implementation, refuse a planned-generated binding transformation with a non-identity no-conversion assignment before rendering; identify the event field and both resolved type spellings. Preserve identical Optional-to-Optional copies and omitted optional inputs (`plan.rs:976–982`; `rust/system.rs:331`). Do not reject every optional input. Do not change neutral planning to turn the mismatch into an obligation. Literal-to-optional shapes already classified as obligations by `determined_input` stay obligations; do not claim this unit implements them.

This conservative refusal can later be replaced by a separately accepted exact lifting renderer, but no new Some/boxing policy is needed to satisfy this story. The proposed initial implementation must not accidentally keep emitting the current plain clone merely because the source compiler accepted assignability.

**System-level types.** These are legal and have no domain owner: `ess-domain/src/system.rs:1390–1401` explicitly tests that fact. Rust `Layout::of` only records domain rosters (`layout.rs:50–70`), while rendering calls the total/panicking owner accessor (`rust/mod.rs:381–390,484–485`; `layout.rs:126–134`). Refuse every planned-generated type without an allocated domain owner **before** any `owner`/`type_name` call. This includes unreferenced system-level types, because the neutral plan still promises them. The reason is the current Rust emitter has no representation location for this construct; it is not invalid ESS. Adding an invented system module or changing all absolute paths is a separate naming migration, not the minimum fix.

**Other source-driven missing layout/support cases.** Any emitted reference must resolve in the checked allocation. Refuse an unsupported representation required by emitted code with its actual capability owner. Do not use render-then-catch-panic or invoke rustc in production as the feasibility algorithm. Retain post-render assertions for implementor defects, not as the response to the enumerated valid-source classes.

## CLI refusal and output policy

The CLI currently synthesizes then writes at main.rs:2394–2415. Match the checked result before write_artifacts or destination creation. On Err, text names target and every source/cause; JSON/YAML renders ess-target-failure/1, then exits1. This applies with or without --out, with no generated-success summary.

Create no directory and change no byte in an existing destination, including metadata/sentinels. Typed Err is the fatal predicate; do not infer failure from artifact filenames, capability counts or free-form text. Ok preserves existing partial Go/Clap/Web reports, success output bytes and containment checks. This story does not add I/O rollback, ownership tracking or directory retirement.

## Web implications, kept within existing result conventions

Web manifests name Rust crates by path and share Rust allocation/names/codecs (web/mod.rs:14–19,294–305; web/layout.rs:18–21,67–75). Direct web::workspace returns Result<Emission, TargetFailure>. Check its Rust prerequisite and actual Web codec/bridge scopes before rendering. A fatal failure returns Err with target web, unchanged neutral plan and source-addressed dependency causes. Preserve Ok Emission fields and historical partial reports. The facade propagates Err; CLI handles it before writes without metadata-shape heuristics.

`web::browser_catalog -> BrowserCatalog` remains an independent semantic catalog API with unchanged `ess-browser-catalog/1` bytes. It renders source identities, wire metadata and neutral dispositions, not a compilable Rust dependency tree (`web/catalog.rs:40–77,365–425`). Do not import the code-workspace fatal gate into this API or narrow the catalog's source-model coverage. Its documented exact equality with an emitted Web catalog applies when that Web workspace is emitted; preserve that equality on admitted fixtures and state clearly that catalog availability is not code feasibility. This keeps the existing SDK catalog consumer on its declared semantic contract.

**Zero generated deliveries is not a Rust refusal.** The system crate is emitted for any component/binding, even with no generated delivery (`rust/system.rs:52–66`); BindingInvocation/invocations and redeliver are conditional (`:208–219,567–569,757–795`). Web `replay_method` currently calls `self.redeliver` unconditionally whenever an interaction layer exists (`web/bridge.rs:329–338`). The minimum Web correction is a conditional body using the same generated-delivery set: retain current bytes when nonempty; when empty, validate the requested published occurrence, perform no binding delivery, then pump and return success. Missing occurrences retain the existing NoSuchOccurrence error. Preserve the existing public Bound replay signature and catalog schema. A model with components but no deliveries must be an admitted positive control, not a blanket rejection inserted to hide this renderer mismatch. A model with neither components nor bindings retains the existing catalog-only bridge branch.

That zero-delivery body correction changes only output previously referring to an unavailable Rust method. It needs a fresh generated Rust-plus-Web compile proof and focused replay behavior proof before being described as fixed. No such proof was executed here.

The implementation's fresh catalog-only wasm32 compiler lane also measured an existing failure in `web/bridge.rs::module`: its no-system branch calls the shared exports but omits the `INPUT` and `OUTPUT` thread-local buffers those exports use. The retained `expanded-isolated-24.log` records E0425 for both buffers after the paired pure Rust workspace compiled. The coordinator binds a minimal declaration of those same buffers in that branch, with no system installation or dispatch invented. This is an additional correction to previously compiler-invalid Web bytes; catalog identity, neutral plan and all previously compilable outputs remain frozen. Verify catalog serving and the existing no-interaction response to replay through local wasm32/Node execution.

## Frozen compatibility surface

- The neutral plan, its JSON/Markdown, source/model/contract provenance, capability names and dispositions are unchanged for every input.
- Every previously valid and compilable Rust artifact path and content is immutable, including manifests, comments, public signatures and helper spellings. Admitted Rust retains target=None and no target metadata. Do not run rustfmt over generated fixtures.
- Admitted Go/Clap bytes and existing Web reports/catalogs remain unchanged. Web bytes change only for the specifically broken zero-delivery code branch and missing catalog-only export buffers; full prerequisite refusal affects newly detected undeliverable workspaces, not a renamed valid API.
- Existing deterministic name fallback/repair rules remain. A newly discovered collision receives a refusal instead of a newly allocated name. No silent namespace migration, import qualification rewrite or broadened helper blacklist.
- Complete Rust/Web failures use ess-target-failure/1 and checked API; existing successful/partial TargetReport bytes are frozen. No new capability kind or neutral disposition is permitted.

Whole-workspace refusal intentionally withholds otherwise individually feasible modules alongside a failing one. This is the chosen target policy, and the report must state it. The current report convention has no partial-Rust-workspace dependency contract to preserve.

## Generated compilation validation to require at dispatch

Fresh generated compilation is essential: the current tests mostly compare source, and comments referring to `cargo xtask synth` are stale. `ess-xtask` currently has Generate/Schema/Release; the normal workspace compiles committed billing/gatepass crates through realization path dependencies. That does not compile arbitrary fresh emission. Add a Rust integration lane under `crates/generate/ess-synth/tests`, using new synthetic inputs and only assigned target scratch. Missing Cargo/toolchain/offline prerequisites fail setup explicitly, never skip the lane.

For each admitted generated workspace, write its exact artifacts to a unique isolated fixture; preserve its generated root `[workspace]` and all manifests. Resolve a fixture-local lock offline, then compile using a fixture-owned target directory and remove an inherited CARGO_TARGET_DIR from the child process. No third-party dependency is required by the generated Rust workspace (`rust/mod.rs:21–29`). The future commands are:

```sh
cargo generate-lockfile --offline --manifest-path <fixture>/Cargo.toml
cargo check --locked --offline --workspace --all-targets --manifest-path <fixture>/Cargo.toml --target-dir <fixture>/target
```

These are required validation commands, not executed results. The test runner should use the Cargo executable supplied to the integration test environment, capture full stdout/stderr and exit, preserve generated hashes before/after, and keep every fixture isolated. Construct the expected sibling `generated/rust/<system>` and `generated/web/<system>` layout for Web checks; do not hand-edit generated path dependencies. Generated Web intentionally refuses a host target. Its bridge requires cargo check/build with --target wasm32-unknown-unknown; use the installed target and a local Node/WASM replay harness for actual behavior. Preserve the host-target refusal as setup evidence, not a generator defect. The existing browser-lab gate remains the committed valid control.

Before a fix, reproduce the known FooBar/Foo_Bar and Optional self-recursion failures with the real old emitter/compiler, and measure mutual recursion, optional binding lift, missing system owner and zero-delivery Web. After implementing each lane, run it alone before package suites. Refused cases validate source compilation to EssIr, neutral-plan parity, deterministic source/cause refusal and no returned code; they cannot honestly claim newly emitted Rust compiled because no Rust code is returned.

| Lane | Required admitted/refused outcomes |
| --- | --- |
| Core names and layout | FooBar/Foo_Bar; final suffix/fallback collisions; generated wrappers/outcomes/helpers; fields/methods/variants; raw-keyword module filenames; package/crate identifiers and duplicate paths. Positive same names in independent modules and existing reserved-package repairs compile. |
| Size graph | Optional self and mutual struct/newtype/union cycles refuse with a cycle; legal List/Map-mediated recursion and acyclic nested Optional compile. No indiscriminate reference-cycle refusal. |
| Bindings | Required-to-optional and added-optional-wrapper no-conversion assignments refuse; identical Optional copies and omitted Optional inputs compile; existing nonmechanical/literal obligations retain neutral disposition and valid stubs. |
| System types | Domainless system type, with and without a referencing domain, refuses before panic; domain-owned counterpart compiles. |
| Wire/shared scopes | Ambiguous effective wire fields/codec names refuse on actual HTTP/Web surfaces; ordinary distinct wire overrides and a union whose tag is value compile using the shared content-key rule. |
| Zero-delivery Web | Components with no bindings, bindings whose deliveries are neutral-refused, and catalog-only no-interaction model; compile paired Rust/Web as applicable. Existing occurrence replay performs no delivery, missing occurrence retains error; a nonempty-delivery historical control is byte-identical. |
| Facades and direct APIs | Direct Rust/Web and facade Err, including zero capabilities; typed nonempty causes and exact unchanged plan; historical partial Web Ok and valid Rust Ok target=None. No direct API bypass. |
| CLI | Text/JSON/YAML with/without --out, new destination and existing sentinel tree; fatal Rust/Web refuses before any writes, partial Web and valid Rust retain established behavior. |
| Historical outputs | Compare complete committed valid artifact maps and neutral plans, not merely two calls to the new implementation. Compile the fresh admitted billing/gatepass/shape controls. Keep all old assertions. |

Run package-scoped `cargo test -p ess-synth -p ess-cli --locked`, `cargo fmt -p ess-synth -p ess-cli --check`, and strict package Clippy after isolated lanes. Coordinator retains each `task check` exit and performs `task site-build` for the binding documentation/validation change as ESS AGENTS requires. No root Taskfile, root manifest or ess-xtask change is established as necessary by this proposal.

## Actual external reliance and Atlas boundary

Service SDK advertised main was refreshed read-only and remains **`48833c6d14ec37cb3b614fca05cf7dd78f63b743`**; its exact local object was inspected. Its `Cargo.toml:56` pins ess-synth to ESS **`d1a66772a91b5411d942d7a45bbf08dfc5de4651`**. `crates/service-builder/src/lib.rs:86–96` calls `ess_synth::synthesize`, then copies only plan and artifacts into EssBuild; it drops target. `:189–191` independently obtains `web::browser_catalog` and parses its canonical JSON. `crates/service-runtime-ir/src/lib.rs:12` consumes SynthesisPlan/CapabilityKind/SynthesisDisposition. No direct `rust::workspace` caller was found in these exact SDK sources.

The selected Result API requires source handling. SDK's separate reader uses a private conversion trait for historical Synthesis and for Result<Synthesis,E> where E is Error + Send + Sync + 'static. It propagates the original typed error before its borrowed TargetReport guard. The guard admits None or an empty Rust report, rejecting mismatched targets, refusals and weakenings. This compiles readers-first against the current pin and accepts the future checked result without version sniffing, dependency changes or public injection APIs.

Current-pin tests distinguish direct Synthesis and synthetic fallible-outcome checks. Once ESS is frozen, a real refused source must propagate through SDK generate/check without destination mutation, alongside valid artifact controls. SDK semantic catalog/planner consumption remain separate. External callers and AgentIDE's exact old builder are not silently upgraded.

Within ESS, the generated billing/gatepass realization code and browser bridge rely on generated APIs/paths; the impact reader checks workspace plan provenance (`ess-diff/src/impact.rs:1023`). Keeping admitted bytes/neutral plans frozen avoids a migration of those persisted surfaces. Outside the exact known SDK object, direct public-emitter users, installed versions and deployment pins remain unestablished.

ESS AGENTS requires canonical bytes preserved unless a coordinated migration changes them, and a new format version if meaning/identity/references/names or an envelope changes; internal Rust capabilities alone do not require one. This decision keeps persisted valid surfaces fixed and introduces a new versioned whole-target failure envelope with checked public API results. **Use coordinator-owned Atlas coordination for that relying-reader/API rollout**, including the exact SDK old pin, new report-handling commit, candidate ESS commit and compile/generation controls. The new ess-target-failure/1 and Atlas ADR0037 are explicit coordinated decisions. If later implementation renames a valid generated API, boxes a representation, changes catalog/plan/report schema or alters canonical valid bytes, that crosses the frozen decision and requires a separately accepted migration before proceeding.


## Implementation constraints and unresolved measurement

Assigned source scopes remain ess-synth and ess-cli plus this design. Qualified ess_synth source scans find only the assigned CLI production caller outside the package; ess-conformance's similarly named functions are separate. No compiler/domain/ess-gen, root manifest/lock/task or generated product edit is required. The new error type/serialization and tests belong inside ess-synth.

The actual zero-capability collision must return non-panicking typed Err through direct Rust, direct Web where applicable, facades and CLI. Keep its plan empty and source-addressed causes nonempty. Also cover nonempty generated plans and obligation-bearing controls so capability count never determines failure.

Previously valid/compilable output and controls are frozen. Previously emitted compiler-invalid or ambiguous wire output may be refused. Zero-delivery Web needs the actual wasm32 compiler and replay evidence. Remaining finite-rule completeness and SDK compatibility are established by implementation/review/gates.

Atlas ADR0037 and the ESS catalog record this API/new format. No old report is relabeled. The initial metadata-only proposal was superseded before its production implementation because the required zero-capability measurement disproved its accounting premise.
