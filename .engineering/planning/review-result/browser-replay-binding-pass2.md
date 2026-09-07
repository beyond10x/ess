---
format: aep.planning-md/1
id: review-result:browser-replay-binding-pass2
kind: review-result
status: active
title: Browser replay fidelity binding attack pass 2
relations:
- reviews: story:review-browser-replay-fidelity
revision: 1
---
unit: browser replay candidate v2, 33bc30a78fcf90e29141d24a4a82d23b9f86c44576d474079478927defa74f88; frozen ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0
verdict: nothing found
cases: executed 0→0, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this final candidate attack; acceptance, integrated-source refresh and implementation selection remain yours

git --no-pager diff --stat: no output; tracked diff empty.

Observed coordinator HEAD: `bbecd8911f8d454e9ba331686739bfea8477a3fc fix(docs): integrate reviewed public support claims`.

The coordinator integrated public-support work during this pass. Entry tracked diff was four coordinator-owned files, 317 insertions and 3 deletions; exit status is clean. All material source reads and hashes used immutable Git blobs at the declared semantic subject, not moving working files. This attack made no source, planning or Git changes.

No remaining candidate findings. The one pass1 warning is resolved in v2; no new finding was identified. This is source-based candidate review, not runtime verification. Zero cases, tests, builds, project binaries and network operations were executed, and there is no red-test receipt.

The correction at binding-draft-v2.md:57,66 makes B06 distinguishable after sorted-map emission. For fixed names a_ref and z_ref, the valid reference vectors can be [(a_ref, instance, left), (z_ref, instance, right)] and [(a_ref, instance, right), (z_ref, instance, left)]. Arrange and capture both aliases first, and declare both inputs with their entity identity type. Authored values preserves the alias values while sorting field names (authored.rs:298,2118–2171); reference validity is checked at 2197–2240. The first emitted alias therefore differs. V2 additionally requires the emitted ordered-vector assertion before browser assertions, retains the literal-identity case and requires unknown subject throughout. These are construction deductions from source, not executed fixture results.

| Families | Candidate and frozen-source assessment |
|---|---|
| B01–B02 | Matching creates/capture can establish only a declared alias and initial lifecycle. Both move sets must produce markers and invalidate affected knowledge. Missing identity/field facts remain unknown internally and visibly; no earlier expectation is used as post-state. |
| B03–B05 | Model literal omission and conversion omission remain explicit. Typed ScenarioValue::Literal carries its admitted Node kind; only the Instance tag references an alias (scenario.rs:1326–1397; authored.rs:627–706). Valid model-specific field types can carry the listed literal vectors, including a literal mapping named instance. Conversion pairs compare the reduced assignment shape, not identity digests of different models. |
| B06 | Resolved as described above. Same-identity-type, already-bound aliases satisfy existing authoring requirements. Reordering YAML alone is no longer the proposed counterexample. |
| B07–B10 | Opposite declared ranking, parameter variation, compound filters, no candidates and missing post-field knowledge all have explicit unsupported results. B08's missing/unresolved wording is bounded by v2:57: a valid unresolved observation value supplies the positive browser vector; absent required authored parameters remain refusals. An earlier declared event supplies a legal Observed tag (authored.rs:1880–1898,2242–2263), without claiming that replay observed it. |
| B11 | V2 explicitly separates authored emission from persisted control fixtures. Emit actual current assets first; construct a valid carrier with the same declared model identity and a suite reference derived from its exact suite bytes, retaining complete parent strings and coverage. Existing coverage_cases.rs:13–43 and coverage_browser.rs:338–355 provide the carrier/pairing construction pattern; AdmittedReplay::from_json at web_replay.rs:179–218 enforces the pair. Test the actual emitted player and DOM with these fixtures, not only the admission function. Unexecuted control/expectation display does not require a new evaluator or CLI route. |
| B12 | Reconstructing the same prefix plus cancelling/invalidation of stale callbacks fits the reserved current assets. Both API knowledge and DOM are required to agree after Step/Back/Reset/Select/Play. |
| B13–B14 | Original-byte lineage, exact metadata, finite Nodes, repeated refusals and all closed-field failures remain existing regression obligations. Unknown semantic markers cannot bypass admission. Mismatched pairs and invalid UTF-8 still require no mounted replay or coverage banner. |
| B15 | Explicitly install the unchanged historical player for its old-input/new-metadata tests, then test both current emitted players separately. The old fixture does not acquire coverage admission, and it is not regenerated to match corrected current assets. |

The original story permits correct supported state or an explicit unsupported marker for each F15 counterexample. V2 stays within that conservative alternative: it does not calculate lost literal/conversion/subject/order semantics, change suite/replay bytes or formats, migrate primitives, or turn replay into implementation conformance. The eight reservations and B01–B15 family set are unchanged by the exact inspected diff. Integrated-source refresh before selection remains a binding obligation; this frozen-source pass does not substitute for it.

The complete original scoper report/readback and complete recorded pass1 report/readback were read. The readback JSON files were fully parsed, their manifests inspected and every retained input pin reverified. All 34 pass1 inputs and all 27 original scoper inputs matched their authorities. The v1-to-v2 diff was recomputed and matched byte-for-byte. The five newly retained review inputs bring this report's manifest to 39 inputs / 966,296 complete bytes hashed. The original 25 Git source inputs were read from the frozen commit; no local-source equality is asserted after concurrent integration.

The following manifest distinguishes full-byte hashing from semantic reading. Prior extents are the already completed pass1 reading in this session; the pass2 column identifies fresh semantic reading or explicit reuse. Absolute paths identify the retained inputs; Git entries are authoritative only at the frozen commit shown above.

| Input | Bytes | SHA256 | Prior semantic extent | Pass2 semantic extent |
|---|---:|---|---|---|
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-scope-brief.md` | 2628 | `39c5dd5bbcfe2b05e4a30deadefb6e70e476f7580fea7dd2b6e1e46042163800` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/story-scoper.md` | 5778 | `ce68c3c41fc74d76e8d7ff66fe82bce2a5585454b89189965d0e063a7d5b0096` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.codex/plugins/cache/beyond10x/ess-specify/0.8.0/skills/specify/SKILL.md` | 8516 | `285e121a1069db99e89631663031db224dcb68e0caa7b6aa50852ccb036dcba7` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/.engineering/planning/story/review-browser-replay-fidelity.md` | 5637 | `a892f8c1d64c7955be1f865da635aabafdcd76579a28224ccf4d6a59434594dd` | Full | Full, 1–62 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/.engineering/planning/story/review-primitive-semantics.md` | 3451 | `354e3de21f81b0f12918db91e5a3f5ff0c7b5a81ac17c776f7b71bbe65fb7855` | Full | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/docs/reviews/2026-09-05-architecture-review.md` | 55490 | `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828` | 495–538 | 495–538 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/web.rs` | 13833 | `4b97a19c378a960356ea69839423d36384b09a3cfc1248165e9bf29a4a9cdbd8` | Full, 1–309 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/web_replay.rs` | 8656 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` | Full, 1–275 | 120–240 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/assets/player.js` | 12902 | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` | Full, 1–295 | 1–180 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/assets/coverage-player.js` | 13120 | `090d5cb2c2edc84b68f2840807b2087d45815047d505681af459508168908024` | Full, 1–300 | 1–85 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/assets/index.html` | 13460 | `bb65078581853962d31244a53a89e56ee90106f3c366552e8046104fc2104243` | 1–50; 100–241 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/tests/coverage_browser.rs` | 17051 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` | Full, 1–392 | Full, 1–392 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/tests/support/browser.rs` | 10371 | `3f58332150f3d5692b7766b039a8350d4b09876cc54857607d2a682541e8b59a` | Full, 1–274 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-compiler/src/ir.rs` | 72083 | `8df6236b197e791a9ebdb8890b956235004920c3eb3acb90f7ad6118eb4ab773` | 547–678; 730–775; 865–920 | 730–811 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/docs/design/review-conformance-coverage-transport.md` | 17268 | `ccbd82ff8871b367b8a58779bddc3d32fae5275d13f1edb62355c4d8f4294aba` | 169–208 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/assets/coverage-admission.js` | 33048 | `f2f6cb7ece670dc062a7355c1ed01a9c0bc18f45d3496addb3e604c37bb28204` | Full, 1–513; truncated section reread | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/scenario.rs` | 124938 | `2474deb86a02684b951697ce47a2e4e9cc3bdd52363f77dc7ef854001f493e62` | 1330–1375; 1650–1850; 1920–1990 | 1320–1420 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/reference.rs` | 60534 | `253ad744a5aeb3b4ddd4899667a960dbe0cc4753442567259a2881de55377054` | 1–70; 428–465 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/AGENTS.md` | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Full prior reading reused; hash reverified | Full, 1–134 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/tests/fixtures/coverage/legacy-player.js` | 12902 | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` | Full through byte identity with `assets/player.js` | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/coverage.rs` | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` | 1–115; 150–200 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs` | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | 453–474; 2719–2766 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/Taskfile.yml` | 7082 | `60390ca3b858b92ef2f970f9a37ab65ca2987bec1cfe23b8eae37ea363bd7a05` | 38–46; 133–158 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/coverage.rs` | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` | 711–838; 891–921 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/tests/support/coverage_cases.rs` | 20682 | `488833a313a1943f3b2f8d7ef8cd07afc6769a9f8d959ab44d79328a8d2cb6ca` | 155–504 | 1–205 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/website/docs/guides/verify-conformance.md` | 9101 | `13bd30869cb53e12b193d032a5d53f33953663b47a52e11d18f9be766dfc7898` | 140–156 | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/website/docs/status/where-this-stands.md` | 2942 | `288d3b8f0f9feabbad32441b0a6bba86bbc2d0750fefd928288161706b41e7d9` | Replay/player search: no hits; hash verification | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/attack-pass1-brief.md` | 2744 | `dacccdd9700a13c1393ce098953d11278798d17fd9dd7c3fa64b05baffc6251c` | Full | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v1.md` | 19610 | `e46fb0661fba5d088a5c2e50ac3628fcf0c3135a703e220ca9638640019d34f2` | Full | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/report.md` | 27218 | `57b892e06040740ad128b99730abae57f74047945405e5495468cf1d0d99db5b` | Full | Full |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/readback.json` | 16097 | `0f5c99be78711142648b1a3b72beb896f3173ff21f79765285cbde1668181bba` | Full parsed JSON | Full parsed JSON; every input entry checked |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v1.readback.json` | 17027 | `0f37038008c1129a5b0286e164d05cf7df365f49fde9f8f95d693a27286bcc0b` | Full parsed JSON; input arrays compared | Full parsed JSON; input array compared with original scoper readback |
| `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md` | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Full | Prior semantic reading reused; full-byte hash reverified |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/authored.rs` | 95379 | `7ac2a8248d74815fd2fac0cf207cea3e0e17de6afcbd9d7ee9b995e1102e11ec` | 230–460; 620–710; 1640–1730; 1880–2005; 2110–2270 | 280–315; 620–710; 1880–2010; 2117–2295 |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v2.md` | 20996 | `33bc30a78fcf90e29141d24a4a82d23b9f86c44576d474079478927defa74f88` | — | Full |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/binding-v1-v2.diff` | 4328 | `199dcc6ef9c2bd775b84c90999b86b63be505c24cd2ea3c794e6e02a61224e78` | — | Full |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/candidate-attack-pass1/report.md` | 10216 | `b64600f50f7d1fa8a8d7ecd618e60202e06c40b97fe5581e7fdca63204e9de6e` | — | Full |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/candidate-attack-pass1/readback.json` | 12326 | `a12bbedbbe19470a14494d0ac16b5c3a5b87b5761ac5bedc56c5bee935c1fa4b` | — | Full parsed JSON; every input entry checked |
| `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/binding-draft-v2.readback.json` | 387 | `7efc8d501b41b45e7ce07cd070debe7731f301f65895865c99dcec9fdb0e2c28` | — | Full parsed JSON; every input entry checked |

Written paths are exactly:

- `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/candidate-attack-pass2/report.md`
- `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/browser-replay-candidate/candidate-attack-pass2/readback.json`

Both are new authorized scratch records under the coordinator target. No other file, build output or scratch path was written. No processes or pending tool calls remain. Quiescent at handoff; no acceptance, selection, store, Git or lifecycle action performed.

```findings
[]
```
