---
format: aep.planning-md/1
id: review-result:fuzz-specification-binding-pass1
kind: review-result
status: active
title: Specification fuzzing candidate binding review
relations:
- reviews: story:fuzz-the-specification-surface
revision: 1
---
unit: candidate fuzz binding v2, SHA256 8af1c91c3c592defd07c242de28eb1b3b92db94aebfd086a59ae49dd29e412df
verdict: NEEDS-CHANGE
cases: executed not run→not run, red not measured
origin: introduced 0 / pre-existing 0 / undecided 3
wrote-outside-worktree: none
needs-coordinator: Amend the three binding gaps below before acceptance; retain candidate-only status and the six reservations.

Scope/diff proof: this is the explicitly assigned document-only adaptation of aep-drive:adversary 0.8.0. No Git command or diff, test, build, gate, CLI probe, integration, active-writer inspection, implementation or store mutation was allowed or performed. The only writes are this report and its source-manifest.json under the assigned coordinator scratch directory. Input hashes, rather than a claimed Git diff, establish the inspected byte boundary. Nothing here is an executed source attack or approval of wave 11.

The coordinator identity supplied by the parent is d2057ffb944455d0ef3a90ab7c5043ae70027289 with published production 239996d846460aee342ce42514378c25b2be5152. The retained baseline names those same commits; all 16 inspected production inputs present in its source map match its hashes. This is not a new full-source or gate measurement. Draft v1 was not compared, so all finding origins remain undecided rather than being attributed to v2's author.

## Findings

| ID | File:line | Category | Severity | Verdict | Origin | Finding |
| --- | --- | --- | --- | --- | --- | --- |
| F1 | `target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md:67` | boundary | blocker | NEEDS-CHANGE | undecided | Bind the five generator calls to ess_gen::artifact::run; registry enumeration alone permits bypassing the production provenance and duplicate-path boundary. |
| F2 | `target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md:62` | acceptance | blocker | NEEDS-CHANGE | undecided | Define live-campaign qualification so zero compiled callbacks or incomplete downstream execution cannot pass merely by recording counters and a successful fuzzer exit. |
| F3 | `target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md:79` | boundary | blocker | NEEDS-CHANGE | undecided | Close the observation and termination rules, including combined assembly/validation, stage-start attribution, complete-stream admission and watchdog kill/reap behavior. |

### F1 — call the checked production generator boundary

**Established by static source inspection.** The draft at :67–77 enumerates the registry and speaks of dispatch, but never identifies artifact::run. The refreshed scope, section 4, does: it requires this wrapper rather than only Generator::generate. The latter is a public, callable method at crates/generate/ess-gen/src/artifact.rs:145, while artifact::run at :160 constructs the production mint, checks readable and matching provenance at :167–183, and returns the duplicate-path refusal at :185–189. The CLI uses the wrapper at crates/edge/ess-cli/src/main.rs:2327–2329.

**Reachable counterimplementation, not an executed probe.** A harness can enumerate the exact five entries, invoke each generate method, count five completed stages and continue to docs-ir and four syntheses. It satisfies the draft's nominal dispatch wording but can survive a bad provenance stamp that the actual production wrapper would panic on. It can also omit the wrapper's ordinary duplicate-path result. That loses the story's downstream survival boundary while keeping apparently complete stage counts.

**Smallest correction.** Name RawSpecFile::parse, Specification::assemble, ess_compiler::compile, ess_gen::artifact::run for every registry entry, the actual docs-ir construction plus pretty serialization, and synthesize_for for each public Target as the shared pipeline. The latter calls are already supported by source and scope; naming them removes ambiguity. Treat artifact::run's Err as an ordinary observed result, continue later calls and let its genuine assertions escape. Keep filesystem-authored site inputs and generated-program compilation outside this property. No production API edit or new reservation is required.

### F2 — make the live campaign's acceptance executable

**Established by static document inspection.** Draft :59–63 specifies stable diversity and asks the instrumented lane to record accepted/compiled counts. It calls a parse-only campaign insufficient, but :83–89 and :155–163 supply neither an explicit qualifying predicate nor a required failure when those counts are zero. Only the three stable mandatory fixtures get an explicit all-ten-call assertion at :81–82. The fixed run/time limits correctly permit fewer than 2,048 executions and cannot supply the missing semantic predicate.

**Reachable counterimplementation, not an executed probe.** The stable vectors can use the correct shared entry while the engine adapter passes a wrongly decoded carrier or always returns before that entry. The fuzzer then completes its bounded campaign normally; every actual callback reports a refusal or no downstream work. A report can truthfully print zero compiled callbacks and retain exit zero, leaving the unit's success decision unspecified. Likewise, one successful live target invocation is insufficient if other accepted callbacks silently omit it. The stable replay's 16 accepted bundles cannot establish what the instrumented callbacks did.

**Smallest correction.** Define qualification separately for each of the two live entries: at least one actual engine callback must produce a compiled in-domain model, every such noncrashing callback must invoke each required downstream stage exactly once, and normal Result refusal still counts as a completed invocation. A crashed/timed-out campaign is failed evidence even when its earlier counters are nonzero. Zero accepted work is insufficient evidence and must prevent claiming that lane green. Select and retain finite starting inputs that actually exercise both entries; distinguish live callback counts from any separately executed seed/replay cases. Report source diversity and startup versus mutated work honestly without inventing a required random acceptance rate.

Bind aggregate identities such as completed generator/target invocations versus completed compiled callbacks, admitting partial counts only on failed attempts. A control that removes an engine's shared-pipeline call or forces all its inputs to decode-refuse must be rejected even while stable replay stays green. These are bounded controls, not a demand for whole-grammar completeness.

### F3 — make partial evidence distinguishable from success

**Established by static source and document inspection.** Draft :79–92 promises a closed observation, start identity, stage counters and a ten-second child watchdog without closing their state transitions or terminal admission rules. Specification::assemble already runs validate and roster validation internally at crates/specify/ess-domain/src/spec.rs:260–262. On an Err, its public result does not expose a separate observed validation invocation or prove exactly where the internal execution stopped. CLI load uses one assembly call at crates/edge/ess-cli/src/load.rs:105 and compile at :117. Splitting that into separately measured assembly and validation calls would either invent observations or add another public validate call that changes the exercised sequence.

A stream containing only attempt start plus completed counters cannot unambiguously identify the currently running stage when that stage crashes. A buffered final-only write can lose all stage observations when the watchdog terminates the child. An exit-zero child with a truncated stream is not successful evidence simply because the retained prefix looks valid. The draft's general transport-failure rule points in the right direction but does not bind how these states are distinguished.

**Smallest correction.** Specify a compact typed contract before implementation; this need not become a product format or general event system:

- Record one monotonically identified attempt with entry identity and exact original input identity. For structured inputs, preserve the deterministic rendered document identities/order too. For the raw entry, clarify whether “decodes and mutates” at :50–51 means libFuzzer mutates the carrier externally or an additional deterministic target translation; identical original bytes must replay the same actual source collection.
- Observe Decode, per-document Parse, combined AssembleValidate, Compile, the distinct generator calls, DocsIr and individual synthesis targets. A separate successful-validation fact may be derived from successful assembly, but label it as derived; do not report another actual call. Ordinary refusal, not reached and completed successful invocation must remain distinct.
- Persist a stage-start identity before invoking production, then its actual returned result. Close each normal attempt exactly once and reject duplicate, missing, out-of-order or truncated records on an otherwise successful run. Preserve a terminal unfinished stage as partial failed evidence after a crash or timeout; never synthesize a completion. Specify the bounded framing/flush policy and make observation-I/O failure operationally nonzero.
- For stable replay, define the ten seconds using a monotonic parent deadline, with child termination followed by reaping before the parent proceeds. Preserve the original input and last observed stage; classify timeout, crash/nonzero exit, launch failure and observation failure separately from ordinary source/target refusal. A child ending successfully without a complete admitted terminal observation is a harness failure. The controlled stalled-child case must prove the child is stopped and reaped, not only that the parent prints “timeout.”

The exact transport and Rust type names can remain implementation choices after those meanings are fixed. For the live engine, reconcile the typed callback stream with raw libFuzzer termination evidence, but do not assume its summary counter is already identical to callback/startup counts without measuring that relationship. No compiler/domain instrumentation or broad source reservation is needed.

## Checks that fit the chosen scope

These are static feasibility conclusions, not executed successes.

**Go prerequisite.** Public EssIr::types and domains at crates/specify/ess-compiler/src/ir.rs:1350–1355 and ResolvedDomain.types at :1214 provide the required typed values. The roster includes compiler-created entity lifecycle enums (:1213); resolve.rs:2736–2741 derives it from resolved type names. The guard should compare actual handles' names against all ir.types keys, not infer ownership from which input document happened to contain a type. Existing Go Layout builds owners from those same roster names at go/layout.rs:187–191 and allocates every ir.types declaration at :457–458 before returning from Layout::of. The proposed guard timing before go/mod.rs:279 is correct, after the existing Binary64 check at :277. The current public facade also checks Binary64 before dispatch at ess-synth/src/lib.rs:285. No contradiction was found with the chosen narrow repair.

The existing TargetFailure constructors in failure.rs:53–62 and :79–95 already enforce nonempty cause/source/detail, sorted unique sources and sorted/deduplicated causes; Go receives the existing /2 envelope and the unchanged plan. All missing type names must remain source-addressed. A small source-grounded binding choice is one MissingTypeOwner cause per missing qualified type, with exactly that name as its source and a Go-specific explanation; Rust's analogous existing construction is at rust/feasibility.rs:177–184. Alternatively bind an explicit grouping rule. Do not infer either new Go detail bytes or grouped-cause multiplicity from the old Rust result. This is a remaining narrow diagnostic choice, not an API gap or a demand for another format.

**CLI regression.** The reserved file already invokes CARGO_BIN_EXE_ess, tests exact exit 1 and empty stderr, switches text versus typed JSON/YAML, and checks both spellings and destination preservation at target_failure.rs:29–75. Retain that Rust/Web empty-domain case verbatim and add the separate Go fixture. The existing Result edge in main.rs:2362–2370 returns exactly 1 on every complete target failure and writes the failure to stdout before the artifact writer at :2373. Tighten the draft's “nonzero checked exit” to that exact existing status. Typed JSON/YAML should check /2, target go, exact MissingTypeOwner source/cause expectations and neutral plan; text should check its actual target/source/explanation. Text Display at failure.rs:119–125 does not print a format discriminator or machine code. No --out, an absent output tree with an absent parent, and a pre-existing sentinel tree can all be covered without CLI production edits.

**CLI drift correspondence.** The six projection choices are private ValueEnum alternatives at main.rs:347–358 and are exposed by the value_enum argument at :162–163. The four synthesis alternatives are at :375–379 and :181–182. The reserved integration test already has the actual binary; ess-cli/Cargo.toml:28–29 already supplies ess-gen and ess-synth. Therefore an exact complete offered-choice comparison is source-feasible without expanding reservations. Use exhaustive public Target matching without a wildcard to catch a newly added enum variant, alongside comparison with actual offered CLI values. Exact help layout, parsing and error handling remain unexecuted; refusal to extract a complete list must fail the check, as the draft already says. Do not quietly replace it with a static array or filter away unfamiliar choices.

**Input boundaries and version.** Duplicate fields/labels, unknown shapes, fixed shallow carrier, label/text bounds, byte cap before decoding, original multi-file retention and accepted structured diversity are explicitly selected. Four actual type families and ownership variants are meaningful bounded coverage; they are not a whole-language generator. One implementation detail needs care: production defaults an omitted system format to V1 at system.rs:1057, while V2 is supported at :53. Classify the actual admitted Specification.system().format (:357), rather than guessing the effective version from a textual header search. An ess/2 source may be valid production input but outside this campaign; distinguish that harness-domain fact from a production validation refusal. Do not rewrite headers or impose production limits.

**Gate and delivery.** The stable/engine split fits the fuzz reservation and avoids root manifests. Taskfile's existing fmt list and workspace checks do not cover those standalone manifests; the explicit new stable task is necessary. The draft correctly leaves actual offline lock resolution and instrumented build feasibility to setup. Their success remains unmeasured here. The public formats row at website/docs/reference/formats.md:51 currently names the finite Binary64 failure; the proposed narrow update fits the exact reservation and must preserve separate normalization and Binary64 statements. All final CLI/formats/production bytes must be refreshed after coverage integration before later selection.

## Evidence and limits

The entire v2, complete existing refreshed scope, full draft revision 8 story, repository AGENTS, installed adversary charter and both baseline report/companion were read. Relevant production and test excerpts are cited above. The historical 33 CLI subprocesses and observed Go exit 101 remain the original record; this review executed none. The companion explicitly identifies the four Rust/Web diagnostics as retained text stdout, not new JSON receipts.

No mutation was executed to demonstrate the counterimplementations. They describe concrete paths admitted by missing binding rules, with source boundaries establishing their relevance. Offline dependency resolution, actual CLI help extraction, instrumented throughput, watchdog behavior, accepted structured/live counts, Go family/reference permutations and canonical before/after byte comparisons remain future measured work. No universal termination, generated-program compilability or fresh source approval is claimed.

One read-only rg search returned exit 2 because a guessed go/refusals.rs file does not exist; the relevant actual layout and module source were read instead. This is retained as a source-search incident in the manifest, not classified as a product/test failure. An initial optional baseline metadata key was absent; the actual source_equivalent_to_published field was then read. No input or result was silently substituted.

The manifest contains 27 inspected files, all unchanged at sealing, including the immutable scope and its earlier manifest. All 16 inspected production files represented in the retained 504-file baseline map match it. The subject remains exactly 14,097 bytes with the assigned SHA256. No original draft, earlier report, baseline or active implementation file was changed.

Source-manifest.json: 15,133 bytes, SHA256 **8b8a2222a83cbef9f0834cb2d5d3e909887e5e4bda03b4227a7c9f40ce0ec183**. The final report hash is supplied separately to avoid a self-reference. After final readback all assigned writes are relinquished; root owns corrections, acceptance, fresh source review, scope/store updates and later selection.

```findings
- file: "target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md"
  line: 67
  category: "boundary"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "Bind the five generator calls to ess_gen::artifact::run; registry enumeration alone permits bypassing the production provenance and duplicate-path boundary."
- file: "target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md"
  line: 62
  category: "acceptance"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "Define live-campaign qualification so zero compiled callbacks or incomplete downstream execution cannot pass merely by recording counters and a successful fuzzer exit."
- file: "target/review-boundaries-11/fuzz-preparation/binding-draft-v2.md"
  line: 79
  category: "boundary"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "Close the observation and termination rules, including combined assembly/validation, stage-start attribution, complete-stream admission and watchdog kill/reap behavior."
```

