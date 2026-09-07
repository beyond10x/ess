---
format: aep.planning-md/1
id: review-result:execution-recovery-authority-pass2
kind: review-result
status: active
title: Execution recovery authority candidate attack pass 2
relations:
- reviews: obligation:review-execution-recovery-implementation
revision: 1
---
unit: execution-recovery authority candidate, report + addendum-v2 + addendum-v3; frozen source 9d84a425e3a0c052bb08975766c5dbd600d04ef0
verdict: nothing found
cases: executed 0→0, red 0 — read-only contract attack
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this final candidate pass before deciding acceptance or decomposition; implementation proof remains outstanding

```text
$ git --no-pager diff --stat
 .engineering/planning/journal.jsonl          |  8 +++
 docs/plan/2026-09-07-review-boundaries-14.md | 82 ++++++++++++++++++++++++++++
 2 files changed, 90 insertions(+)

$ git --no-pager log -1 --oneline
60279c3 docs: open public support remediation wave
```

This coordinator-owned diff was identical at entry and exit. I wrote no files.

No cases, builds, project binaries or validation commands were executed. Retained declaration validation and compilation both report exit 0; all output lengths and hashes match. The compiled document contains 44 types and no commands, entities or components. These receipts establish declaration structure, not recovery enforcement.

Here, `C` means `target/review-boundaries-14/preparation/execution-recovery-authority-candidate` within the coordinator checkout.

The three first-pass contradictions are resolved by the complete candidate’s precedence rules:

| Prior finding | Correction examined |
|---|---|
| `C/report.md:93`: baseline-only retirement required a desired inventory | `C/addendum-v3.md:16–55` admits each projection against its own complete inventory. Observation covers the baseline/desired union. Apply predicates require desired-only addresses to be absent beforehand and baseline-only addresses absent afterward; retirement admits a baseline without a desired projection. |
| `C/report.md:440`: interrupted prefixes required a terminal fact | `C/addendum-v3.md:59–90` distinguishes empty reservations, valid incomplete prefixes, closed histories, unpublished stages and corrupt published entries. Missing acknowledgement remains uncertainty. Omitting `retry_of` cannot bypass relevant retained-history admission. |
| `C/report.md:468`: invocation-directory publication lacked its parent sync | `C/addendum-v3.md:94–120` requires synchronization of the new reservation directory and its publishing parent before claim publication or mutation. The rule covers every dynamically introduced directory and preserves subsequent lock, entry and finalization barriers. |

No remaining finding arose from the complete R01–R29 accounting:

| Families | Assessment |
|---|---|
| R01–R03 | Whole-input refusal, complete registry admission and local-only dry-run remain explicit. Target/principal, baseline, physical store aliases and conflicting release/object indexes have named admission obligations. |
| R04–R11 | Missing evidence cannot establish absence. Current framed OCI proof admission remains separate from target progress; cold failures, corrupt warm entries and valid warm reuse retain their distinct outcomes. |
| R12–R19 | Preparation → fresh durable `Observed` → durable `Prepared` → launch ordering is coherent. Process uncertainty stops later mutation. Prefix admission, corruption refusal, complete history checks and invocation-local freshness preserve the restart distinctions. |
| R20–R21 | Equal desired inputs still require observation. Drift requires independently supplied repair authority; a current match neither replays an uncertain mutation nor invents historical acknowledgement. |
| R22–R27 | Early removal guarding, reverse order, baseline-only retirement, union inventory checks, foreign-incarnation refusal and authenticated direct-object absence remain explicit. Finalizers and retained direct objects prevent an absence claim. |
| R28 | Durable claims survive parent death and prohibit automatic stealing. Changed registry, target/principal, binary or claim checks stop launch. Administrative quiescence remains an explicit additional precondition where a predecessor claim survives. |
| R29 | Failed final recording prevents completion; retained valid prefixes support fresh observation after required quiescence without unconditional replay. |

The implementation must distinguish mechanically checked bindings from the candidate’s administrative assumptions: complete registry enrollment, unique uncloned host/cluster identities, excluded outside writers, protected immutable executable installation, actual Helm semantics, caller-approved post-defaulting fingerprints, storage synchronization semantics and independently established quiescence. Neither hashes nor compatible help output establish those assumptions. The observation claim remains bounded to direct resources at an observation point; descendants, PVC/PV cleanup and perpetual convergence remain outside it.

All hashes below cover complete bytes. `G` inputs were read from the frozen Git commit and matched local bytes. “Full” denotes complete textual inspection; compiled JSON was fully parsed and its inventories inspected.

| Input | Bytes | SHA256 | Read extent |
|---|---:|---|---|
| L `target/review-boundaries-14/preparation/execution-recovery-candidate-attack2-brief.md` | 4,664 | `91c8592cfe16b63f63aa9e740e47c5c13f864d6b81d94524973130ff0b6e82a0` | Full |
| L `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md` | 19,212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Full |
| L `C/report.md` | 43,739 | `1b0d0cbc32eef5957cb2ccb6d3435a46b3e388e03cd8d528727b28f8e8e93a0e` | Full, including superseded declarations |
| L `C/addendum-v2.md` | 21,155 | `fac0c4de5bcd6483d935a09324193abdf33116eadb4f373943a62f5b4ae5a87f` | Full |
| L `C/addendum-v3.md` | 9,150 | `845cac69786aecc40de6b4a8dc324c32bc147610087090760bd4f880ecfe9248` | Full |
| L `C/candidate-attack-pass1/report.md` | 11,594 | `6598a0907927d4746e1ad508713f5008cb93bb583f2c5459bf19022d025e1f1e` | Full |
| L `C/model-v2/system.yaml` | 75 | `c981b332cc9bdb08690c3f4f44f732138dc66bf725bcef930ce3ed4836825f28` | Full |
| L `C/model-v2/domains/execution.yaml` | 10,578 | `e03ca97fc429ab20950c2c75e0c8567e6174cc4c93aca6ca9c6860338c27e294` | Full |
| L `C/model-validation-v2/commands.json` | 1,420 | `b937c08c8aae232f1d43e25aabc6c26c1e7c6e7a8cf3a60067573051376d08cf` | Full |
| L `C/model-validation-v2/validator-pin.json` | 636 | `e9382fbd3cabcdb134696b5d8005683c860d1940889372365217534eb83fb6ba` | Full |
| L `C/model-validation-v2/validate.stdout` | 33 | `170db78f0a676d70973aa3d0486ebcf2bb0a638479d4ee1afb909b2b212ec3f9` | Full |
| L `C/model-validation-v2/validate.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Empty |
| L `C/model-validation-v2/compile.stdout` | 36,251 | `40a9fb6e2ac3c67d27e87b6c935422ada17490637a93d3517e9c9dc28cc7cd20` | Full JSON parse; inventory inspection |
| L `C/model-validation-v2/compile.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Empty |
| G `docs/design/review-execution-recovery.md` | 32,454 | `679482baa7e9584c78c715fed55d201f89b33d95e6488f7d9b318e5fb15e2783` | Full |
| G `.engineering/planning/obligation/review-execution-recovery-implementation.md` | 2,036 | `1b33b67f094baab6416a8f4339dd9e06bce1d6ee58d16f32089df53afc4c3187` | Full |
| G `AGENTS.md` | 7,816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Full |
| G `crates/generate/ess-deployment/src/environment.rs` | 35,923 | `fe92b2b0ef2b561f0cd862cd5c669dfcb1d12c0e9e695e7e44b928431d2fab7e` | 637–941 |
| G `crates/edge/ess-cli/src/main.rs` | 139,710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | 1634–1750; 3131–3214 |
| G `crates/edge/ess-cli/src/oci_cache.rs` | 26,840 | `e064382f41efcb81a9dea8352af8b973283dce0430739e71ffe8afb022098c6c` | 117–218; 378–410; 490–578 |
| G `crates/edge/ess-cli/tests/support/fake_delivery.rs` | 1,516 | `a5313312c51b171a4e5bd7585e47f5f9dbe2a03fc23e33eeb7e9cb0e621c6a1f` | Full |
| G `crates/edge/ess-cli/tests/persisted_delivery.rs` | 12,400 | `008e2929f9b303a9e81eb7065ace03ea16f22501a2598249a27e9883c2b94052` | 108–175 |

Total: **22 material inputs, 417,202 bytes**.

Quiescent. No files, tests, project binaries, network requests, integrations, store/Git mutations or lifecycle operations were performed. The public-support unit remained untouched. This report makes no acceptance, implementation-completion or live-deployment claim.

```findings
[]
```