---
format: aep.planning-md/1
id: review-result:execution-recovery-authority-pass1
kind: review-result
status: active
title: Execution recovery authority candidate attack pass 1
relations:
- reviews: obligation:review-execution-recovery-implementation
revision: 1
---
unit: execution-recovery authority candidate + addendum/model-v2; frozen source 9d84a425e3a0c052bb08975766c5dbd600d04ef0
verdict: NEEDS-CHANGE
cases: executed 0→0, red 0 — read-only contract attack
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: resolve the three contract findings before acceptance or implementation decomposition

```text
$ git --no-pager diff --stat
 docs/plan/2026-09-07-review-boundaries-14.md | 59 ++++++++++++++++++++++++++++
 1 file changed, 59 insertions(+)

$ git --no-pager log -1 --oneline
60279c3 docs: open public support remediation wave
```

This coordinator-owned diff was identical at entry and exit. I wrote no files.

No cases, builds, project binaries or validation commands were executed. The retained coordinator declaration receipts report validation and compilation exits of zero; their output lengths and hashes agree. The compiled JSON contains 44 types and no commands, entities or components. That establishes declaration structure, not recovery behavior.

Here, `C` means the exact directory:

`target/review-boundaries-14/preparation/execution-recovery-authority-candidate`

The complete candidate accounts explicitly for all R01–R29. Three proposed clauses still prevent a consistent implementation of that accounting:

| Location | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|
| `C/report.md:93` | blocker | NEEDS-CHANGE | introduced | Requiring every desired-or-baseline chart’s rendered inventory to equal the desired inventory makes baseline-only retirement inadmissible. |
| `C/report.md:440` | blocker | NEEDS-CHANGE | introduced | Requiring exactly one terminal journal fact conflicts with recovery from a valid interrupted prefix that has no terminal fact. |
| `C/report.md:468` | blocker | NEEDS-CHANGE | introduced | Synchronizing each journal entry and its immediate directory does not durably publish the newly created invocation directory in its parent. |

“Introduced” refers to these candidate clauses relative to the accepted binding’s unresolved mechanisms; it does not describe a newly introduced production defect.

1. **Bind inventory admission to the projection being admitted.** Report lines 86–95 apply the procedure to desired **or baseline** releases, but step 6 compares against the entire **desired** inventory. The v2 `ReleasePermit` explicitly permits `baseline: Some`, `desired: None` (`model-v2/domains/execution.yaml:135–145`; report line 434). For a retirement with a nonempty baseline inventory, the required desired comparator does not exist. The addendum’s preparation-before-observation sequence does not exempt removals. This reaches the positive R23–R26 paths and also baseline admission when an upgrade changes object membership. Specify separate baseline and desired admission comparisons, then explicit before/after predicates for apply and retirement. No runtime failure is claimed; this is a contradiction between the proposed procedure and an expressly permitted model state.

2. **Define interrupted-prefix admission separately from completed-journal admission.** The mandatory reader constraints require one `Stopped` or `Completed` terminal fact. A crash immediately after durable `Prepared` produces neither. Yet report line 472 and addendum lines 34–35 explicitly require that state to remain indeterminate and recoverable after quiescence. Treating its missing terminal as corruption instead triggers R18’s refusal, which quiescence explicitly cannot bypass. This affects R12, R15–R16, R25 and R29. Define a valid incomplete-prefix grammar with zero terminal facts, a closed grammar requiring exactly one terminal, and distinct handling for a torn publication, a sequence gap and a structurally valid interrupted prefix. This requires a reader-contract correction, not necessarily a field change.

3. **Include reservation publication in the durability barrier.** Report line 42 creates `invocations/<nonce>/` dynamically and relies on its retained reservation to prevent reuse. Line 468 synchronizes journal files and their immediate parent, `<nonce>/`; it does not require synchronizing `invocations/` after creating the nonce directory. Synchronizing the child’s contents does not establish durable publication of that child’s directory entry in its parent. Consequently the stated protocol permits target mutation after `Prepared` while its containing reservation/history remains outside the specified durability barrier. Specify creation and parent-directory synchronization for every newly introduced directory before lock/`Prepared` admission permits mutation, with corresponding failure and restart cases. This concerns the claimed storage durability in R12/R16/R29; no power-loss experiment was performed.

The remaining accounting was examined as follows:

| Families | Assessment |
|---|---|
| R01–R03 | Whole-input admission, local-only dry-run and target/baseline refusal remain explicit. The complete registry scan addresses competing active authorities and alias collisions within its declared scope. |
| R04–R11 | Missing observation remains unknown. The candidate correctly uses the current framed OCI proof and original descriptor verification, without reviving the obsolete checksum-pair mechanism or treating a warm artifact as applied state. |
| R12–R19 | Preparation/observation/`Prepared` ordering is now coherent. Started-call uncertainty, settled prefixes and fresh restart observations are preserved in intent; findings 2–3 leave journal admission/durability incomplete. |
| R20–R21 | Equal desired inputs still require observation; repair requires independently supplied authority. Matching current state does not fabricate historical acknowledgement. |
| R22–R27 | Early removal guarding, reverse ordering, incarnation refusal and authenticated direct-object absence are explicit. Finding 1 blocks the proposed positive retirement procedure. Finalizers and retained direct objects cannot become absence; descendant/PVC cleanup is expressly outside the claim. |
| R28 | Durable retained claims avoid automatic lock stealing and PID/timeout reclamation. Registry completeness, exclusion of outside writers, immutable tool installation and administrative quiescence remain declared trust assumptions—not independently established fencing. |
| R29 | Final publication failure prevents completion in the proposed behavior. Findings 2–3 must be resolved so the resulting interrupted history has a precise admissible representation and durability boundary. |

The optional predecessor reference does not itself establish complete historical admission; the proposal’s corruption-refusal, retained-history and generation checks must apply independently of whether `retry_of` is supplied. I did not identify a separate contradiction sufficient to add another finding. Likewise, caller-pinned post-defaulting fingerprints and the exact Helm semantic contract remain explicit administrative inputs; suitable help output or a successful declaration compilation cannot create those inputs.

All material input hashes cover complete bytes. `G` inputs were read from the frozen Git commit above and matched the corresponding local files. `L` paths are relative to the coordinator checkout unless absolute. “Full” denotes complete textual inspection; the compile output was fully parsed with inventory inspection.

| Input | Bytes | SHA256 | Read extent |
|---|---:|---|---|
| L `target/review-boundaries-14/preparation/execution-recovery-candidate-attack-brief.md` | 3,528 | `bb40a504f5db8fba610d9692c1124a67ce425cc1ab2b0bd785c804e06f5e07a5` | Full |
| L `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md` | 19,212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Full |
| L `C/report.md` | 43,739 | `1b0d0cbc32eef5957cb2ccb6d3435a46b3e388e03cd8d528727b28f8e8e93a0e` | Full, including superseded declarations |
| L `C/addendum-v2.md` | 21,155 | `fac0c4de5bcd6483d935a09324193abdf33116eadb4f373943a62f5b4ae5a87f` | Full |
| L `C/model-v2/system.yaml` | 75 | `c981b332cc9bdb08690c3f4f44f732138dc66bf725bcef930ce3ed4836825f28` | Full |
| L `C/model-v2/domains/execution.yaml` | 10,578 | `e03ca97fc429ab20950c2c75e0c8567e6174cc4c93aca6ca9c6860338c27e294` | Full |
| L `C/model-validation-v2/commands.json` | 1,420 | `b937c08c8aae232f1d43e25aabc6c26c1e7c6e7a8cf3a60067573051376d08cf` | Full |
| L `C/model-validation-v2/validator-pin.json` | 636 | `e9382fbd3cabcdb134696b5d8005683c860d1940889372365217534eb83fb6ba` | Full |
| L `C/model-validation-v2/validate.stdout` | 33 | `170db78f0a676d70973aa3d0486ebcf2bb0a638479d4ee1afb909b2b212ec3f9` | Full |
| L `C/model-validation-v2/validate.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Empty |
| L `C/model-validation-v2/compile.stdout` | 36,251 | `40a9fb6e2ac3c67d27e87b6c935422ada17490637a93d3517e9c9dc28cc7cd20` | Full JSON parse; top-level/type inventory |
| L `C/model-validation-v2/compile.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Empty |
| G `docs/design/review-execution-recovery.md` | 32,454 | `679482baa7e9584c78c715fed55d201f89b33d95e6488f7d9b318e5fb15e2783` | Full |
| G `.engineering/planning/obligation/review-execution-recovery-implementation.md` | 2,036 | `1b33b67f094baab6416a8f4339dd9e06bce1d6ee58d16f32089df53afc4c3187` | Full |
| G `AGENTS.md` | 7,816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Full |
| G `crates/generate/ess-deployment/src/environment.rs` | 35,923 | `fe92b2b0ef2b561f0cd862cd5c669dfcb1d12c0e9e695e7e44b928431d2fab7e` | 637–941 |
| G `crates/edge/ess-cli/src/main.rs` | 139,710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | 1634–1750; 3131–3214 |
| G `crates/edge/ess-cli/src/oci_cache.rs` | 26,840 | `e064382f41efcb81a9dea8352af8b973283dce0430739e71ffe8afb022098c6c` | 117–218; 378–410; 490–578 |
| G `crates/edge/ess-cli/tests/support/fake_delivery.rs` | 1,516 | `a5313312c51b171a4e5bd7585e47f5f9dbe2a03fc23e33eeb7e9cb0e621c6a1f` | Full |
| G `crates/edge/ess-cli/tests/persisted_delivery.rs` | 12,400 | `008e2929f9b303a9e81eb7065ace03ea16f22501a2598249a27e9883c2b94052` | 108–175 |

Total: **20 material inputs, 395,322 bytes**.

Quiescent. No files, tests, project binaries, network requests, integrations, store/Git mutations or lifecycle operations were performed. The public-support unit remains untouched and handed off.

```findings
- file: target/review-boundaries-14/preparation/execution-recovery-authority-candidate/report.md
  line: 93
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Requiring every desired-or-baseline chart’s rendered inventory to equal the desired inventory makes baseline-only retirement inadmissible.
- file: target/review-boundaries-14/preparation/execution-recovery-authority-candidate/report.md
  line: 440
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Requiring exactly one terminal journal fact conflicts with recovery from a valid interrupted prefix that has no terminal fact.
- file: target/review-boundaries-14/preparation/execution-recovery-authority-candidate/report.md
  line: 468
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Synchronizing each journal entry and its immediate directory does not durably publish the newly created invocation directory in its parent.
```