---
format: aep.planning-md/3
id: story:feature-request-293
kind: story
status: draft
title: Explorer excludes every command with an Optional input (and every command with an unknown_instance branch)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#293
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore_optional_unknown.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_explore_optional_unknown
- confidence: inferred
  path: docs/design/explorer-optional-unknown.md
revision: 5
---
## Outcome

Resolve beyond10x/ess#293: Explorer excludes every command with an Optional input (and every command with an unknown_instance branch).

## Origin

beyond10x/ess#293, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 5, `~/.cache/ess-gaps/triage-cb/`).

## Fit review


The accepted need is full exploration of existing Optional inputs and explicit unknown-instance outcomes. The seven-question fit review below retains the actual source, scope, cost, alternatives and execution evidence. The immutable private source report has SHA256 86bf244ceb13f4211ad083fe156d29f23993665719756cb2e200ccf57eb5a7af.
# #293: Optional inputs and unknown-instance explorer coverage

Intake recommendation: accept the need as an existing-source explorer capability gap. Implement against a written draw/decision/replay contract; removing the two planner exclusions alone is insufficient. No production implementation, new format, or release claim is authorized by this report.

## Executed evidence

Source carrier was `5c5aeaf795a46aacfd3709e04f630d83d8a6a837`, with runtime import `046db8a6805154aa5cafc63b0a4b741bc26e954d`. HEAD was unchanged through the probe and `git diff HEAD -- crates` was empty. Exact source objects are retained in `dependency-source-git-objects.txt`. All authored source is brand-free. The Rust driver and temporary Go/JavaScript diagnostic targets live outside repositories; no production file or AEP artifact was changed.

The final `corrected/model.yaml` declares four commands: Create, Close with an ordinary move plus wrong-state and unknown-instance refusals, PlainNote with a String note, and OptionalNote differing in that note's type is Optional<String>. The note is not used by a guard. A granting actor covers all commands. An identity/state view makes the existing wrong-state obligation observable. Specification assembly and compilation succeed; ordinary synthesis produces **7 scenarios, 0 refusals**, and one existing `GrantEnforcedByCaller` note. Both Go and TypeScript packages are emitted by the real `emit_with_model` functions from that same IR and suite.

Actual generated runtime results, with four sequences seeded 1–4 and 32 commands per sequence:

| Observation | Go | TypeScript |
| --- | ---: | ---: |
| Executed commands | 128 | 128 |
| Create calls | 56 | 56 |
| PlainNote calls | 72 | 72 |
| OptionalNote calls | 0 | 0 |
| Close calls | 0 | 0 |
| Healthy-target disagreements | 0 | 0 |
| Wrong PlainNote event detected | seed 1, step 1 | seed 1, step 1 |

Both report exactly these exclusions:

```
demo.items.Close: outcome `not-found` has a `unknown_instance` condition
demo.items.OptionalNote: input `note` is a `optional`
```

Both list all three Close outcomes and OptionalNote/noted as excluded outcomes. Included outcomes are Create/created and PlainNote/noted, with no unreached, ambiguous or undetermined entries. The Go diagnostic also calls the actual internal `explorePlanOf` and records exactly Create and PlainNote in its drawn-command plan. TypeScript evidence uses the public `explore` result and actual target callback counters, not an export or rewrite of the private planner.

The independent targets own their records, IDs, command answers and queried view rows. They do not read suite assertions. The negative control changes only the returned PlainNote event flag: both explorers report that false differs from the input-derived true value and shrink to the one-command trace. This proves an active explorer/target comparison; it does not claim the excluded commands were tested.

The Rust acceptance check first requires healthy execution and detection of the plain-input fault, then requires callbacks for both affected commands in both runtimes. It exits **101** at `EXPLORER_293_EXCLUSION_GAP`, listing all four missing runtime/command pairs. Repeating that Rust check over the retained runtime results gives the same named gap and exit. The generated runtimes were executed once for the initial source and once for the final clean source; the final acceptance-check repeat is not a second execution of the target.

The original source/package/result set remains intact at the probe root. It had one unrelated fixed-suite synthesis refusal because no identity/state view exposed the closed row. The final source adds only `Items`, a read-your-writes view projecting `id: ItemId` and `state: Item.State`; private targets add QueryView over their own rows. No Optional or unknown-instance semantics changed. The first attempted view incorrectly typed state as String and was source-refused; that diagnostic is retained as `corrected/view-type-refusal.log`. An earlier source draft omitted required explicit event payload ownership and was likewise source-refused, retained as `emit-source-refusal.log`. Neither authoring refusal is counted as the product defect.

Important evidence SHA256 values:

| Artifact | SHA256 |
| --- | --- |
| Rust driver `src/main.rs` | `6a8aa198442b9b61b8069091913be7789a3df14a71429952219473a6dd2a0685` |
| Final source `corrected/model.yaml` | `2818436126068e19fe737df0735ad176b17587208f179015c3a4f8a29585db8a` |
| Final suite | `f078caf8f1c903d7d1048df6cb1ca3022ad8ae0dfcddd46a2aeb675c80ace706` |
| Final source/synthesis result | `a2e30e89d245ff9e4ba3fd0c149be2dc2f5c9cfaca4c9ebed93a614de968db32` |
| Final Go result | `1fbb51c562a930f73f77989f0dfdd7efe08f965da49bba5e77eac546d6b1c8b3` |
| Final TypeScript result | `ecc4d8a3d031aa293d887b33e07d7f2f145aeb052c02be904945be3dd042a35e` |
| Final Go log | `6821365d614a797b1c08003d4416ef0bf843a8e8b1729e7f69d0e1394190a095` |
| Final TypeScript log | `2069967d666e543f98d1b2cbb77c8292e6d5e49a9cc6eb2138613c7dbbfa0773` |
| Final acceptance red | `520a22b5311de2b9a7ca9d588d2caaac50860b408115a60f8ac3caaab7e96983` |
| Acceptance-check repeat | `9434749e79af5d09f25a156e38ee02d73a02de91a95b58f43a4376f7d19c9da7` |
| 136-file pre-report evidence manifest | `cf1bc4b17841955c5a06592bb18de043873cc73994f6cb78f23caac5a27d7be7` |

The manifest includes original and final source, generated packages, temporary targets, compiled TypeScript, lockfile, logs and exit receipts. Local-path dependency manifests remain private; this report uses publication-safe relative paths.

## Seven-question fit review

### 1. Need, separated from the proposal

The consumer needs random command sequences to include commands whose inputs may be absent and commands whose specified answer for an unknown identity is not-found. Otherwise successful exploration leaves entire command families unexamined. The request proposes drawing Optional<T> as absent or a supported T and choosing the unknown-instance branch for an identity absent from model state. The measured evidence confirms both exclusions today, even with unused optional input and complete ordinary synthesis.

### 2. Classification

This is an implementation capability gap over admitted ESS syntax, not a syntax request. Optional input already compiles and the emitted IR explicitly describes its wrapper. The unknown-instance branch already compiles and synthesizes an executable fixed-suite obligation. Generated explorers explicitly refuse these constructs in their plans. Their reported exclusions are honest; the defect is missing exploration coverage, not a falsely reported execution count.

### 3. Can it already be expressed?

Yes. The final source uses existing ess/15, Optional<String>, ordinary move/wrong_state/unknown_instance, actor grants and a view. No new keyword or type is needed. Fixed conformance synthesis and manually supplied precondition inputs are separate routes; neither causes these excluded commands to be randomly drawn. Changing an optional field to required or removing the not-found branch changes the contract and is not a solution.

### 4. Fit and composition

Source inspection after the actual red identifies more than admission switches:

- Go `src/go/explore.go::exploreResolveAs` and TypeScript `src/ts/explore.ts::resolveKind` support primitive/declared branches and reject Optional wrappers. Add a bounded Optional draw representation around every inner kind already drawable in the selected exploration mode, including supported nominal/struct nesting. Preserve the actual unsupported reason for an unsupported inner type. Absence, explicit null, no-record redraw and unknown model knowledge must not share an accidental sentinel. Apply existing input presence semantics and predicate `facts`/defined/missing behavior rather than inventing a second truth rule.
- `explorePlanAs` / `plan` reject `unknown_instance` before any target call. Allowing that condition must accompany a real decision rule: `exploreDecide` / `decide` currently return an ambiguous draw when a supplied identity has no record, after input-guarded refusals. Select the source-declared unknown-instance outcome at the correct point without disturbing input-refusal, accepting-guard, external, wrong-state or default precedence.
- `exploreDrawStep` / `draw` force the supplied subject input to `mustExist`. An empty model therefore skips the draw, and a populated model always chooses a known row. Unknown-instance support requires explicit known/unknown subject draws, not merely lifting the planner rejection. Unknown candidates must be proved absent from the model, with a bounded collision strategy; finite identity domains cannot guarantee a fresh value once exhausted.
- Existing-record references are stored as entity/creation-index pairs for shrink/replay. Optional present references must retain that provenance, while absence must survive cloning/serialization and fresh unknown identities must remain intentionally unknown after shrinking. Do not relabel an absent record caused by removing an arranger as the same original known-record witness without checking the reproduced failure.
- Common planner/type paths serve both serial and concurrent exploration. The concurrent recorder delegates history verdicts to Rust; it must preserve admitted inputs and history semantics, not acquire a new local model oracle. The current probe executes the serial public explorer only. Concurrent draw/history/shrink compatibility is a required follow-up test, not measured support.
- Other condition/effect exclusions, including the #221 family, are adjacent work and must not be silently admitted by widening one predicate. Distinguish source-admitted unknown-instance refusals from any broader create/update-on-existence combination and cover each intended admitted shape with a compiled fixture. This report establishes the refusal case, not every possible upsert form.

Existing native conformance, generated conformance runners and source projections need no feature rewrite for this issue. Browser declaration playback is not an explorer. The all-features goal still requires their separately tracked gaps; passing a Go/TS explorer probe cannot discharge them.

### 5. Second unrelated adopter

An inventory editor accepts an optional adjustment explanation and rejects updates to a missing stock record. A reservation service accepts an optional cancellation comment and reports a missing reservation ID. Both need omission/present draws and known/missing identity sequences through the same generic type and outcome semantics; neither depends on consumer-specific command names or naming policy.

### 6. Cost

The concrete production surface is the existing embedded `crates/verify/ess-conformance/src/go/explore.go` and `src/ts/explore.ts`: kind resolution, input drawing, branch decisions, reference replay and shrinking. Add focused Rust-driven generated-runtime tests beside `tests/explore.rs` or in a dedicated `tests/explore_optional_unknown.rs`, with brand-free shared fixtures. Existing `tests/explore_preconditions.rs` and concurrent explorer tests are neighbors because they share plan/sendable/draw behavior. No source/IR/suite/report format bump, public target callback, native interpreter change or new independent checker is established by this need.

Preserve deterministic seed behavior where practical, but do not promise identical historical traces after adding previously excluded commands; changed reachability changes the draw population. Specify seed reproducibility and cross-language agreement for the same final model/runtime, and update pins only with explained coverage changes. Newly committed harness/implementation code must be Rust; the existing generated Go/TypeScript runtime assets follow their existing repository convention.

### 7. Alternatives

1. Leave explicit exclusions in place: truthful but does not meet the reported need or the all-features goal.
2. Delete exclusion checks only: reject. It still cannot draw absent optional values or unknown subjects, and missing records remain ambiguous.
3. Add bounded Optional draw semantics and source-selected unknown-instance handling, with replay/shrink parity: recommended. Require absence/present and known/unknown outcome witnesses, not just smaller exclusion lists.
4. Force specific commands with authored scenarios/preconditions: useful independent controls, but not replacement random-sequence coverage.
5. Replace the explorer with the native interpreter or broaden all type/condition support at once: unnecessary for the established defect and creates a much larger authority/parity scope. Keep other admitted feature gaps visible and separately designed.

Proposed decision: accept the need with an explicit design for Optional absence representation, unknown-identity draws and precedence, deterministic replay/shrink, and the shared concurrent draw seam. Preserve exact existing unsupported classifications outside that accepted scope.

## Required implementation red/green matrix

Keep this final seven-scenario source as the regression seed. Add actual Go/TS healthy targets and independent mutants for ignored present/absent Optional input, wrong defined/missing branch, wrong not-found outcome/error, mutation on not-found, and wrong ordinary/missing/wrong-state precedence. Require affected commands to be present in the plan and observed at callbacks. Cover supported inner primitive/newtype/enum/struct kinds, Optional members inside structs, explicitly unsupported inner types, initial empty model, known live/closed rows, fresh-ID collisions, input-guarded refusal before not-found, and omitted/null policies where source-admitted. Exercise shortened traces and fresh target replay so a change in identity existence cannot fabricate the same failure. Add a concurrent recording control at its existing Rust history-check boundary; do not claim it passed from serial runs.

## Reproduction and resource handoff

The retained Rust driver emits packages from `model.yaml` in its working directory. Build/run it with `cargo run --offline` from the probe root using the agreed idle target cache, one job, no debug/incremental output, and a private external TMPDIR. Invoke the built executable from `corrected/` to emit the final source. Private target sources are retained in each generated package; a fresh emit would need those copied back from the retained files.

For the retained final packages, run `GOWORK=off GOMAXPROCS=1 go test ./essconform -run '^TestProbe$' -count=1 -v` from `corrected/go`; run `tsc --project probe.tsconfig.json` then `node probe.mjs` from `corrected/ts/essconform`; run the built Rust probe with argument `check` from `corrected/`. The first two harnesses exit 0 after validating healthy/mutant controls; the Rust acceptance check exits 101 because the affected commands remain excluded.

Toolchain: rustc 1.98.1, Go 1.27.0-X:nodwarf5, Node 22.23.2, TypeScript 6.0.3. TypeScript used the existing noCheck runtime-test convention; no strict typecheck or full repository gate is claimed. Only the small Rust probe compiled against the warm servers cache; no cleanup occurred. The assigned cache's lease was acquired, heartbeated and released. Free space stayed above the 8 GiB floor and was 18,705,657,856 bytes at handoff. No running probe/build remains. Root owns AEP/design decisions and any next implementation authorization.

## Decisions

Accept, redesigned: lift the two explicit exclusions only with recursive typed Optional absence/presence, deliberate known/fresh subject draws, source-selected unknown-instance behavior, and preserved shrink/replay/concurrent semantics. No new syntax, format or target callback authority. This accepts the need; exact production authorization follows independent review of the design supplement.

The proposed supplement implementation-design.md has SHA256 50f0e84dfde4f5ba2060646d5cb2d4c632070c17a6dcf41bc872cf8c6a9f21ad. Root has read it and assigned independent review. It scopes two existing Go/TypeScript explorer assets plus focused Rust-driven tests; unresolved details must be settled with compiled source fixtures, particularly subjectless branch precedence and absence propagation. Story remains draft while design review runs; no worker is implementing this unit yet.
