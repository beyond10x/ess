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
| `home-path:sha256:fa64e05d8f374cd566b2cc290d2fbe4b038981c0570d552e0e7a7dd2176de66d` | 2628 | `39c5dd5bbcfe2b05e4a30deadefb6e70e476f7580fea7dd2b6e1e46042163800` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:49c3712a082bd44088051a9d540765e029662521bd89db1160f7e99b8688ac7e` | 5778 | `ce68c3c41fc74d76e8d7ff66fe82bce2a5585454b89189965d0e063a7d5b0096` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:8cd39e32eeefeffde10adc563596c3e03c75af45eff4e83f944c799b73364939` | 8516 | `285e121a1069db99e89631663031db224dcb68e0caa7b6aa50852ccb036dcba7` | Hash verification only | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:9986209d7d4396e050120d651ae118335e07e38470fb42ec5b959ce4ee0bcd01` | 5637 | `a892f8c1d64c7955be1f865da635aabafdcd76579a28224ccf4d6a59434594dd` | Full | Full, 1–62 |
| `home-path:sha256:c387af20bfef9447ac27e3c8832bb4b77d5f194ba7105bc3d979bcc31975cd6c` | 3451 | `354e3de21f81b0f12918db91e5a3f5ff0c7b5a81ac17c776f7b71bbe65fb7855` | Full | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:a8a36b8b474ca55b09551207d222275febcabaab277e1f37e43a6c69aa652fe4` | 55490 | `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828` | 495–538 | 495–538 |
| `home-path:sha256:48c34e1b3c7579d5064b00cde71ceb59230723278d53ad9835ec9b5959f7e5d1` | 13833 | `4b97a19c378a960356ea69839423d36384b09a3cfc1248165e9bf29a4a9cdbd8` | Full, 1–309 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:6eec1dd66617afe0ab6bf7407164d64ef5137366eccc55ace9294f49a7674932` | 8656 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` | Full, 1–275 | 120–240 |
| `home-path:sha256:efef003b77d0031013e16aca66bf7a62369efa189d0e8325d91ba8562b495d18` | 12902 | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` | Full, 1–295 | 1–180 |
| `home-path:sha256:2882920543ba4d78998a12c52c69bce5447b86269e4f3e1a4beb14eac95dd396` | 13120 | `090d5cb2c2edc84b68f2840807b2087d45815047d505681af459508168908024` | Full, 1–300 | 1–85 |
| `home-path:sha256:f545ada7bb54e05ed921c9e61651085c7f07f71be48dda75a61663ccc56b08e6` | 13460 | `bb65078581853962d31244a53a89e56ee90106f3c366552e8046104fc2104243` | 1–50; 100–241 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:a99691e3023ee23f24720799c7d8470a6a84e9b1991526bca16771452ec40ec8` | 17051 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` | Full, 1–392 | Full, 1–392 |
| `home-path:sha256:a0a98a133e0ad708657d096256d96eaa8dfc9273d0b596034c345c1377cb3928` | 10371 | `3f58332150f3d5692b7766b039a8350d4b09876cc54857607d2a682541e8b59a` | Full, 1–274 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:6877c10a7717a691210139b749b1057d64364550a474b48ec930e342f825d684` | 72083 | `8df6236b197e791a9ebdb8890b956235004920c3eb3acb90f7ad6118eb4ab773` | 547–678; 730–775; 865–920 | 730–811 |
| `home-path:sha256:268398418f457389d9cb820036107d2c5edf1de22a92c6ce6eb5af67c0cc6929` | 17268 | `ccbd82ff8871b367b8a58779bddc3d32fae5275d13f1edb62355c4d8f4294aba` | 169–208 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:2197659f9f1f1967cfeacaecc6269eda2bc0536f3d5e59dbe49e9ae5b565934c` | 33048 | `f2f6cb7ece670dc062a7355c1ed01a9c0bc18f45d3496addb3e604c37bb28204` | Full, 1–513; truncated section reread | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:9b9bc10febc458d5890b02133cbb78060a40449ae3469f200cf141bebe44a44b` | 124938 | `2474deb86a02684b951697ce47a2e4e9cc3bdd52363f77dc7ef854001f493e62` | 1330–1375; 1650–1850; 1920–1990 | 1320–1420 |
| `home-path:sha256:be7792bddc21bf1285f7d10f2e05f4bd30c4530fe1fbc5791bac3cff9b9455f1` | 60534 | `253ad744a5aeb3b4ddd4899667a960dbe0cc4753442567259a2881de55377054` | 1–70; 428–465 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:f32117db287ee8c1110f4ba9f7caa0b151a8c1c8fdad25fa9a08bb31cb763e9f` | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Full prior reading reused; hash reverified | Full, 1–134 |
| `home-path:sha256:ea77bef3247a8135b1d66a5387d661f02322470fb8b91345d6fd4a4693d3d3c5` | 12902 | `990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f` | Full through byte identity with `assets/player.js` | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:3f35b102747803a6e1effc88638b1025d8cb8d333cfd75aa64c3655122b21343` | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` | 1–115; 150–200 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:f93162aae901a19b6b1b9b9093c43aa999c1e5f4be90b8e4177cecc0d2a2f9ed` | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | 453–474; 2719–2766 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:dad726eb24dec325c91eb507a81a735005f80c30e6b510198e835a80674280e9` | 7082 | `60390ca3b858b92ef2f970f9a37ab65ca2987bec1cfe23b8eae37ea363bd7a05` | 38–46; 133–158 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:b0c41625fb7c1f95e7e56e93388618baadf50c2260a12b9785102bed1142d59c` | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` | 711–838; 891–921 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:52c0890ff46b2cd4fec09b533c47056a8c7c7f2af38ec76eae4576390a491eb2` | 20682 | `488833a313a1943f3b2f8d7ef8cd07afc6769a9f8d959ab44d79328a8d2cb6ca` | 155–504 | 1–205 |
| `home-path:sha256:5f059d7697134ef03507ca93c9d6519056d6f1f28895733a5495f7aed6755369` | 9101 | `13bd30869cb53e12b193d032a5d53f33953663b47a52e11d18f9be766dfc7898` | 140–156 | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:485db3e45b87cdc28fba5367bc32afdfe8f22bae1ff93996231aec6dc2b5f9c8` | 2942 | `288d3b8f0f9feabbad32441b0a6bba86bbc2d0750fefd928288161706b41e7d9` | Replay/player search: no hits; hash verification | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:4385b0e376d0577e0a6fe484a13a5436e6595858808ac4d05d591078a0d92731` | 2744 | `dacccdd9700a13c1393ce098953d11278798d17fd9dd7c3fa64b05baffc6251c` | Full | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:878e18c427771899d9f62fee2e5ba91fae6ecc35f3617a3aa6d4554eb782afeb` | 19610 | `e46fb0661fba5d088a5c2e50ac3628fcf0c3135a703e220ca9638640019d34f2` | Full | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:98979ffe6b12528474213f29574efc2275fe523b09dabbfa0c1b79d3dd0d0ac6` | 27218 | `57b892e06040740ad128b99730abae57f74047945405e5495468cf1d0d99db5b` | Full | Full |
| `home-path:sha256:a964908c985c3342b424cccca9159510858f1739cbfc5be064087a10a29001b2` | 16097 | `0f5c99be78711142648b1a3b72beb896f3173ff21f79765285cbde1668181bba` | Full parsed JSON | Full parsed JSON; every input entry checked |
| `home-path:sha256:8f849d6db148f7c0ef1d2097890a4164162d785b65706a77fadacd83487faeb9` | 17027 | `0f37038008c1129a5b0286e164d05cf7df365f49fde9f8f95d693a27286bcc0b` | Full parsed JSON; input arrays compared | Full parsed JSON; input array compared with original scoper readback |
| `home-path:sha256:e82505babddc671cebf5b7f4c841d234b8ece153cba47dc082beaaf48b4a8210` | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Full | Prior semantic reading reused; full-byte hash reverified |
| `home-path:sha256:2fd1cf0f0c497525f29cd1819a4113b06319983176ef79c7de49cb609349935e` | 95379 | `7ac2a8248d74815fd2fac0cf207cea3e0e17de6afcbd9d7ee9b995e1102e11ec` | 230–460; 620–710; 1640–1730; 1880–2005; 2110–2270 | 280–315; 620–710; 1880–2010; 2117–2295 |
| `home-path:sha256:b065c48217847897ad88183a6191679b6412fa043a1bca3a097cbb48a495d73f` | 20996 | `33bc30a78fcf90e29141d24a4a82d23b9f86c44576d474079478927defa74f88` | — | Full |
| `home-path:sha256:304423b3cac9944d57192e93369c04cf9cf03af04ff91149c3fc4d36f9665402` | 4328 | `199dcc6ef9c2bd775b84c90999b86b63be505c24cd2ea3c794e6e02a61224e78` | — | Full |
| `home-path:sha256:a03021297d47b4fb4c2b86759d2a973b45dc9103e742b3dacb8a4722e877ac41` | 10216 | `b64600f50f7d1fa8a8d7ecd618e60202e06c40b97fe5581e7fdca63204e9de6e` | — | Full |
| `home-path:sha256:bf065ea9b3646b2efe433920fc542f3c679a2ce02b84f9d8f81f9a325a311599` | 12326 | `a12bbedbbe19470a14494d0ac16b5c3a5b87b5761ac5bedc56c5bee935c1fa4b` | — | Full parsed JSON; every input entry checked |
| `home-path:sha256:59cc8fbbc7a21ec218e476c8f2d9ad0534e0b05cae9400848285802085013de5` | 387 | `7efc8d501b41b45e7ce07cd070debe7731f301f65895865c99dcec9fdb0e2c28` | — | Full parsed JSON; every input entry checked |

Written paths are exactly:

- `home-path:sha256:61492f90d435fccab31097e013ac2f0e8eb17f659d2ef0907091ea1f9c14340b`
- `home-path:sha256:00a3962be317c2ff1a3fe918e43942948fede04befe9adcdad4b176d56c91ea9`

Both are new authorized scratch records under the coordinator target. No other file, build output or scratch path was written. No processes or pending tool calls remain. Quiescent at handoff; no acceptance, selection, store, Git or lifecycle action performed.

```findings
[]
```
