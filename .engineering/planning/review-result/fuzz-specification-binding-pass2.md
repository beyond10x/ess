---
format: aep.planning-md/1
id: review-result:fuzz-specification-binding-pass2
kind: review-result
status: active
title: Specification fuzzing binding correction review
relations:
- reviews: story:fuzz-the-specification-surface
revision: 1
---
unit: candidate fuzz binding v3, SHA256 225d2561dc92a8884180fee2e828ad1b9f2b211a72b5d75844e9066240a80f43
verdict: nothing found
cases: executed not run→not run, red not measured
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: No remaining document correction from this review; retain candidate-only status and refresh integrated source before future acceptance/selection.

Scope/diff proof: second and final candidate document correction review under the same installed aep-drive:adversary 0.8.0 charter and the parent's explicit document-only adaptation. No Git/diff command, test, build, gate, CLI probe, implementation, store or integration operation, or active-writer inspection occurred. Only this report and source-manifest.json were written in the new assigned fuzz-binding-review-2 directory. The v3 was read in full and its requested hash matched. All 27 retained review inputs were rehashed unchanged; seven relevant source files were refreshed with bounded excerpts. Hash correspondence replaces no claim of a Git diff or executed source review.

This review closes F1–F3 at the document-contract level. It found no new contradiction introduced by those corrections. It does not select this story, accept the binding on the coordinator's behalf, or establish implementation behavior.

## Prior finding closure

All v3 references below are to target/review-boundaries-11/fuzz-preparation/binding-draft-v3.md.

| Prior finding | Correction and exact v3 lines | Static closure |
| --- | --- | --- |
| F1: the generator wrapper could be bypassed | :80–86 explicitly requires RawSpecFile::parse, Specification::assemble, ess_compiler::compile, artifact::run for every registry entry, docs-ir construction/serialization and synthesize_for. It explicitly rejects direct Generator::generate as the checked boundary and preserves ordinary Err continuation versus panic failure. | CLOSED. The chosen wrapper is the source owner of provenance assertions and duplicate-path Result in ess-gen/src/artifact.rs:160–189. Registry matching, docs-ir and exhaustive Target matching remain intact at v3 :83–97. |
| F2: a live lane could exit zero without qualified downstream work | :68–76 requires at least one compiled in-domain live callback per entry, all required downstream calls exactly once for every compiled callback, admitted complete observations and successful termination. Zero compiled callbacks makes the lane non-green. :118–127 binds exact per-stage totals and separates live callbacks from stable replay and libFuzzer summary counters. | CLOSED. A decode-only/no-work engine cannot be called green by borrowing the stable vectors' counts. :219–223 explicitly requires that countercontrol to fail live qualification while stable replay stays green. Crash/timeout/observation failure cannot be reclassified by earlier successful counters. |
| F3: partial observations and child termination were insufficiently closed | :99–116 defines ordered attempt/input/entry identities, concrete stages, combined AssembleValidate, flushed stage-start/result records, once-only normal closure and rejection of malformed/incomplete/overlapping records. :129–134 requires a monotonic ten-second parent watchdog, termination and reaping, separate failure classifications and complete observations for exit-zero children. | CLOSED. The combined assembly/validation stage matches Specification::assemble's internal validate and roster checks at ess-domain/src/spec.rs:260–262. :219–226 adds a missing terminal/stage record despite exit-zero control and preserves original red evidence. |

These are verified textual corrections against actual unchanged source. No control was executed; CLOSED is not a passing mutation-test result.

## Corrections checked for new contradictions

**Go cause shape and bytes.** V3 :147–159 now selects one MissingTypeOwner cause per missing qualified type, exactly that name as the sole source, and the literal detail template:

```text
Go cannot assign a package to type <qualified-name>: no domain owns it
```

Substitution is explicit. The shared constructors at crates/generate/ess-synth/src/failure.rs:53–62 and :79–95 can represent this without new fields or public APIs, retain sorted/deduplicated causes and sources, preserve the neutral plan and choose Go's existing /2 envelope. Distinct missing type names remain distinct causes. The chosen detail is new Go behavior; it is not claimed to reproduce the retained Rust text. The existing Binary64 guard still precedes layout at go/mod.rs:277–279, and the proposed guard remains before the ownership-dependent allocation. Library cases, all four valid type families, direct/facade comparison, multiple missing owners, owned/empty-domain positives and original-byte preservation remain required at v3 :161–170. Their execution remains pending.

**CLI status, formats and destinations.** V3 :172–180 now requires the existing exact exit status 1 and empty stderr. It distinguishes typed JSON/YAML /2 assertions from text Display assertions and covers both spellings, no --out, absent output and parent, and a pre-existing sentinel destination. This matches the source: main.rs:2362–2370 prints the failure on stdout and returns 1 before the artifact writer at :2373. TargetFailure::fmt at failure.rs:119–125 prints target, source and detail, with no format discriminator or machine code. The existing Rust/Web empty-domain /1 case is explicitly preserved; the source test already separates text versus JSON/YAML at target_failure.rs:50–64. No CLI production write is implied.

**Actual admitted source version.** V3 :58–62 and :105–106 classify Specification.system().format after successful assembly and before this campaign's compile stage. The actual RawSpecFile assembly path defaults missing format to V1 at ess-domain/src/spec.rs:652, and the admitted SystemSpec exposes its typed format at system.rs:357. A valid V2 source can therefore be identified as out of this campaign without falsely reporting a production validation refusal or rewriting its header. This resolves the earlier textual-header ambiguity and does not widen the fuzz domain.

**Replay identity and stream rules.** V3 :50–55 specifies that libFuzzer mutates raw carrier bytes externally; the raw target performs no additional source mutation. The structured route remains deterministic and retains rendered documents, order and identities at :99–103. The observation protocol can thus identify the actual source collection used by either entry. Explicit record-size limits and digest-referenced retained large details at :108–116 leave transport/type naming to implementation while forbidding truncation into success. Stable child termination/reaping is required before continuation. Those are coherent requirements; no running child, stream admission or flush behavior was measured here.

**No silent success for incomplete work.** Normal Result refusals still count as completed downstream invocations, whereas an unfinished stage after crash/timeout remains partial failed evidence. Successful lanes require all ten calls per compiled in-domain attempt and count equality for each identity (:118–123). No-work and missing-record controls at :219–226 target the specific earlier bypasses. The bounded time cap may end a healthy campaign before 2,048 runs without weakening these conditions; v3 :204–212 correctly requires actual counts and keeps crash/timeout failures distinct. No random acceptance-rate requirement was added.

**Reservations and preserved limits.** The six path reservations, CLI choice correspondence, standalone stable and engine graphs, mandatory original seeds, finite structured diversity, Binary64 precedence, existing format/default boundaries, stable gate integration and public delivery order remain intact. The draft explicitly retains the setup prerequisite for both offline lock graphs. Nothing in these corrections requires compiler/domain or CLI production changes, root workspace membership, a new persisted ESS format, generated-program compilation or universal grammar/termination coverage.

## Remaining implementation evidence

No new binding finding is assigned to the following explicitly unmeasured work:

- Resolve both offline dependency graphs, build the chosen dated nightly/ASan lane and capture exact tool/source identities.
- Choose concrete bounded record framing/size and Rust types that obey the now-closed semantics; execute live no-work, incomplete-stream, stage-skip, panic and watchdog controls.
- Prove actual CLI offered-choice extraction is complete and refuses unknown/unextractable choices.
- Measure mandatory seed and structured/live acceptance, actual callback/stage counts, Go cause cases, CLI destination preservation and complete before/after valid Go bytes.
- Refresh coverage-integrated production, CLI, formats and scope before later binding acceptance and story selection; run all assigned source/delivery gates only when selected and authorized.

These are implementation and gate obligations already present in the candidate, not evidence that they passed or reasons to introduce more scope. This is the final assigned document correction pass, not a wave 11 writer source attack.

## Input and output integrity

The subject is 18,496 bytes and retains the exact requested SHA256. The first review's report and manifest are unchanged at SHA256 11c4ea08dd53e862004decac7ef21ece2b4363b0a76c5b5796dc4046d9b37d2d and 8b8a2222a83cbef9f0834cb2d5d3e909887e5e4bda03b4227a7c9f40ce0ec183 respectively. All 30 current inputs remained unchanged at sealing. All 16 inspected production files represented in the retained 504-file baseline map still match that map; this is byte correspondence to retained production, not a new Git HEAD or gate observation. Both immutable seed baseline reports and the earlier scope remain untouched.

One initial read-only metadata refresh failed because the orchestration embedded lowercase JSON booleans into a Python literal; the outer parser received a traceback instead of JSON. The surfaced tool error and correction are retained in the manifest. Its raw subprocess receipt was not retained, so no exact subprocess status is claimed. The corrected read-only projection verified every retained input. No source write, test case or product failure occurred.

Source-manifest.json: 16,656 bytes, SHA256 **14d6da98c9b52baa8319ad5bab768288c5dd775e9d4bcfc1e40eef57b4ad627e**. The report hash is supplied separately. After final readback, all writes are relinquished. Root retains binding acceptance, scope/store changes, fresh source verification and future selection.

```findings
[]
```

