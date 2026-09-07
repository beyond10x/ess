---
format: aep.planning-md/1
id: review-result:browser-replay-binding-pass1
kind: review-result
status: active
title: Browser replay fidelity binding attack pass 1
relations:
- reviews: story:review-browser-replay-fidelity
revision: 1
---
unit: browser replay candidate v1, e46fb0661fba5d088a5c2e50ac3628fcf0c3135a703e220ca9638640019d34f2; frozen ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0
verdict: NEEDS-CHANGE
cases: executed 0→0, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this pass and route the B06 fixture correction; acceptance and selection remain yours

```text
 .engineering/planning/journal.jsonl          |  10 ++
 docs/plan/2026-09-07-review-boundaries-14.md | 167 +++++++++++++++++++++++++++
 2 files changed, 177 insertions(+)
```

HEAD: `60279c34737863b59e9b46fa4fb07deaf1b010a3 docs: open public support remediation wave`.

The tracked changes and three untracked review-result records are coordinator-owned. This attack changed nothing. No cases were added; no runtime tests, builds, project binaries or network operations were executed. There is no red-test receipt.

| Location | Verdict / severity / origin | Finding |
|---|---|---|
| `target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v1.md:64` | NEEDS-CHANGE / warning / introduced | B06's authored insertion-order variants serialize to the same input order, so they do not exercise opposite first-reference choices in the emitted players. |

This is a candidate validation defect established by source inspection. `authored.rs:298` stores authored inputs in `BTreeMap<String, Written>`; `values` accepts and returns sorted maps at `2118–2129`; `ScenarioStep::ExecuteCommand.input` remains a `BTreeMap` at `scenario.rs:1687`. Both CLI web routes compile authored scenarios (`main.rs:2726–2747`, CLI `coverage.rs:173–185`). Reversing the same written key/value pairs therefore cannot reverse the input traversal used by the current first-reference heuristics.

Correct B06 by swapping two already-bound aliases across fixed ordered input names, or renaming the subject/decoy fields so the first emitted reference actually changes. Assert distinct emitted ordered `(field, tag, alias)` sequences before the browser assertions. Retain the literal-identity counterexample and require unknown subject in every case. This needs no production or format expansion.

The remaining assessment is bounded to the candidate and inspected source:

- **B01–B05:** Declared capture aliases, typed ScenarioValue handling, omitted literal/conversion markers and sets accompanying moves cover the corresponding F15 failures without inventing unavailable semantics. The requirement to remove affected facts from internal known state closes the display-only workaround.
- **B07–B10:** Explicitly unsupported view results cover ordering, parameter filters, unsupported predicates and unknown-versus-empty cases. B08 can use an admitted unresolved observation value; malformed missing required parameters must retain their authored refusal.
- **B11:** Keeping original steps and expectations visibly unexecuted prevents assertions, declared events and binding cards from manufacturing observations or instances. The authored frontend does not emit every ScenarioStep control; this inspection does not establish that every control variant can originate through CLI authoring. Additional variants can be carried by admitted persisted fixtures exercising the actual emitted assets, as existing browser admission tests already do.
- **B12:** Prefix reconstruction plus timer cancellation/generation checks is implementable within the reserved assets. Comparing API state and rendered DOM after reset/selection addresses stale callbacks and stale visible state.
- **B13–B15:** Admission-before-state remains separate from unsupported semantics. The frozen historical player can be installed explicitly in its compatibility bundle while both current emitted players receive separate tests. No historical fixture regeneration, coverage weakening or parser reinterpretation is necessary.
- **Scope:** The conservative option fits the story’s explicit-unsupported acceptance. Stronger evaluation, primitive migration and persisted-format changes remain outside it. Refresh against integrated public-support source remains necessary before selection.

For input accounting, paths below use these exact prefixes:

```text
R = /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/
C = R + target/review-boundaries-14/preparation/browser-replay-candidate/
V = R + crates/verify/ess-conformance/
E = R + crates/edge/ess-cli/
D = /home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/
S = /home/timo/.codex/plugins/cache/beyond10x/ess-specify/0.8.0/
```

All **34 inputs, 918,043 bytes** were read for SHA256 verification. All original manifest entries matched their retained size/hash; the 25 Git source inputs matched the frozen commit and local bytes. “Full” below means semantic reading of the entire file; line ranges distinguish partial reading from whole-file hashing. The historical fixture was additionally verified byte-identical to the fully read current default player.

| Input | Bytes | Semantic read extent | SHA256 |
|---|---:|---|---|
| `R/target/review-boundaries-14/preparation/browser-replay-scope-brief.md` | 2628 | Hash verification only | `39c5dd5bbcfe2b05e4a30deadefb6e70e476f7580fea7dd2b6e1e46042163800` |
| `D/agents/story-scoper.md` | 5778 | Hash verification only | `ce68c3c41fc74d76e8d7ff66fe82bce2a5585454b89189965d0e063a7d5b0096` |
| `S/skills/specify/SKILL.md` | 8516 | Hash verification only | `285e121a1069db99e89631663031db224dcb68e0caa7b6aa50852ccb036dcba7` |
| `R/.engineering/planning/story/review-browser-replay-fidelity.md` | 5637 | Full | `a892f8c1d64c7955be1f865da635aabafdcd76579a28224ccf4d6a59434594dd` |
| `R/.engineering/planning/story/review-primitive-semantics.md` | 3451 | Full | `354e3de21f81b0f12918db91e5a3f5ff0c7b5a81ac17c776f7b71bbe65fb7855` |
| `R/docs/reviews/2026-09-05-architecture-review.md` | 55490 | 495–538 | `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828` |
| `V/src/web.rs` | 13833 | Full, 1–309 | `4b97a19c378a960356ea69839423d36384b09a3cfc1248165e9bf29a4a9cdbd8` |
| `V/src/web_replay.rs` | 8656 | Full, 1–275 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` |
| `V/assets/player.js` | 12902 | Full, 1–295 | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` |
| `V/assets/coverage-player.js` | 13120 | Full, 1–300 | `090d5cb2c2edc84b68f2840807b2087d45815047d505681af459508168908024` |
| `V/assets/index.html` | 13460 | 1–50; 100–241 | `bb65078581853962d31244a53a89e56ee90106f3c366552e8046104fc2104243` |
| `E/tests/coverage_browser.rs` | 17051 | Full, 1–392 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` |
| `E/tests/support/browser.rs` | 10371 | Full, 1–274 | `3f58332150f3d5692b7766b039a8350d4b09876cc54857607d2a682541e8b59a` |
| `R/crates/specify/ess-compiler/src/ir.rs` | 72083 | 547–678; 730–775; 865–920 | `8df6236b197e791a9ebdb8890b956235004920c3eb3acb90f7ad6118eb4ab773` |
| `R/docs/design/review-conformance-coverage-transport.md` | 17268 | 169–208 | `ccbd82ff8871b367b8a58779bddc3d32fae5275d13f1edb62355c4d8f4294aba` |
| `V/assets/coverage-admission.js` | 33048 | Full, 1–513; truncated section reread | `f2f6cb7ece670dc062a7355c1ed01a9c0bc18f45d3496addb3e604c37bb28204` |
| `V/src/scenario.rs` | 124938 | 1330–1375; 1650–1850; 1920–1990 | `2474deb86a02684b951697ce47a2e4e9cc3bdd52363f77dc7ef854001f493e62` |
| `V/src/reference.rs` | 60534 | 1–70; 428–465 | `253ad744a5aeb3b4ddd4899667a960dbe0cc4753442567259a2881de55377054` |
| `R/AGENTS.md` | 7816 | Full prior reading reused; hash reverified | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` |
| `V/tests/fixtures/coverage/legacy-player.js` | 12902 | Full through byte identity with `assets/player.js` | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` |
| `E/src/coverage.rs` | 8137 | 1–115; 150–200 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` |
| `E/src/main.rs` | 139710 | 453–474; 2719–2766 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` |
| `R/Taskfile.yml` | 7082 | 38–46; 133–158 | `60390ca3b858b92ef2f970f9a37ab65ca2987bec1cfe23b8eae37ea363bd7a05` |
| `V/src/coverage.rs` | 33620 | 711–838; 891–921 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` |
| `E/tests/support/coverage_cases.rs` | 20682 | 155–504 | `488833a313a1943f3b2f8d7ef8cd07afc6769a9f8d959ab44d79328a8d2cb6ca` |
| `R/website/docs/guides/verify-conformance.md` | 9101 | 140–156 | `13bd30869cb53e12b193d032a5d53f33953663b47a52e11d18f9be766dfc7898` |
| `R/website/docs/status/where-this-stands.md` | 2942 | Replay/player search: no hits; hash verification | `288d3b8f0f9feabbad32441b0a6bba86bbc2d0750fefd928288161706b41e7d9` |
| `C/attack-pass1-brief.md` | 2744 | Full | `dacccdd9700a13c1393ce098953d11278798d17fd9dd7c3fa64b05baffc6251c` |
| `C/binding-draft-v1.md` | 19610 | Full | `e46fb0661fba5d088a5c2e50ac3628fcf0c3135a703e220ca9638640019d34f2` |
| `C/report.md` | 27218 | Full | `57b892e06040740ad128b99730abae57f74047945405e5495468cf1d0d99db5b` |
| `C/readback.json` | 16097 | Full parsed JSON | `0f5c99be78711142648b1a3b72beb896f3173ff21f79765285cbde1668181bba` |
| `C/binding-draft-v1.readback.json` | 17027 | Full parsed JSON; input arrays compared | `0f37038008c1129a5b0286e164d05cf7df365f49fde9f8f95d693a27286bcc0b` |
| `D/agents/adversary.md` | 19212 | Full | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` |
| `V/src/authored.rs` | 95379 | 230–460; 620–710; 1640–1730; 1880–2005; 2110–2270 | `7ac2a8248d74815fd2fac0cf207cea3e0e17de6afcbd9d7ee9b995e1102e11ec` |

Quiescent. No files or scratch paths written, no retained processes, no pending tool calls. No acceptance, implementation selection or lifecycle action performed.

```findings
- file: target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v1.md
  line: 64
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: B06's authored insertion-order variants serialize to the same input order, so they do not exercise opposite first-reference choices in the emitted players.
```