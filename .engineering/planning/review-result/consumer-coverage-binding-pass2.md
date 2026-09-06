---
format: aep.planning-md/1
id: review-result:consumer-coverage-binding-pass2
kind: review-result
status: active
title: Consumer coverage candidate binding correction review
relations:
- reviews: story:review-consumer-coverage
revision: 1
---
unit: consumer-coverage candidate binding v2, SHA256 4789563e5c2de9e05c5750661a40916aee6a7096ed36053a1c9f78762254f75d, ESS coordinator wt-752828a285ba
verdict: CONFIRMED (one candidate wording warning; four earlier findings resolved)
cases: executed 0→0, red 0; no test, compiler, generator, build or suite command was run
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: correct the v2 definition-container wording to the locked provider's actual definitions map; preserve the staged profile/baseline freeze

git --no-pager diff --stat: NOT RUN — this assigned document-only correction review explicitly prohibits Git commands. Only the two new records listed in part 6 are written.

## 1. Scope, subject and exact comparison

This is bounded document pass 2 against the four findings in the first immutable review. It is not a third implementation attack, either wave11 writer source attack, an approval, selection, accepted baseline or verifier-produced implementation evidence. The assignment overrides the adversary charter's test/Git sequence for this document-only task. The exact installed adversary 0.8.0 charter, current repository AGENTS, current consumer story and complete scoper report were read in the first pass and retained; all their input hashes were checked again. The complete v2 draft and its exact text delta from the original immutable draft were read. Semantic inspection here is limited to those corrections and their source-dependent consequences.

Covered candidate:
- target/review-boundaries-11/next-scope/consumer-binding-draft-v2.md
- SHA256 4789563e5c2de9e05c5750661a40916aee6a7096ed36053a1c9f78762254f75d

Unchanged comparison authorities:
- Original candidate SHA256 9a2a0daa07efa9d065044d8ea8d42df3cd76ee1bdbe0841ce15b5aba1bc1cda6.
- Complete scoper report SHA256 2194fea88d194d9c54b22e38f5990d6e2b00b64ed633ac039ebf833ca29abd61.
- Current story:review-consumer-coverage revision 6 SHA256 1095be495bf9afc1291c00c33a69a57876bd931e46ca64c643d95a67ed96fead.
- First review report SHA256 7147237c3e40e72590f7c3f85ad388c39ba2beb73ac702ebd529791eeb1b1fa4.
- First review manifest SHA256 84051401451a0f4a09ad977116ec4f86faff7e2ede764a006021b8ba7ae565bb.

The coordinator's supplied source identity remains published 239996d846460aee342ce42514378c25b2be5152 with opening d2057ffb944455d0ef3a90ab7c5043ae70027289. This review makes no fresh Git-object, remote, CI or publication claim. Its identity check is the exact 58 prior inputs plus the three comparison/v2 records and four locked-dependency files: 65 before/after hashes in input-manifest.json, all stable.

## 2. Cases and suite execution

No cases were authored or executed. No source copy was mutated. No compiler, schema generator, helper, test binary, Rust/Go/browser/TypeScript runtime, formatter, Clippy, gate or integration ran. There is no executable implementation test result and no measured red runtime case. The header's 0→0 is an explicit absence of execution, not a passing zero-case suite.

Read-only operations were file reads, numbered source excerpts, exact text comparison and SHA256 comparisons. A local dependency checksum-map read initially exited 1 because schemars-0.8.22/.cargo-checksum.json is absent. The successful replacement read the cached .crate archive without extracting files to disk, checked its SHA256 against Cargo.lock, and compared the three cited source members byte-for-byte. That read setup failure is not a semantic finding or suite failure.

## 3. Closure of the four original findings

These are comparison results, not active finding rows in this pass's YAML.

| Prior finding/signature | V2 correction | Bounded result |
| --- | --- | --- |
| C1: original draft:35, NEEDS-CHANGE / undecided | Lines 47–71 bind the separate wire namespace, root JSON-pointer identities, complete structural fingerprints, annotation versus literal-data positions, local-reference edges and a closed keyword/position profile that must be frozen before baseline acceptance. | Resolved as the original identity/accounting gap. The new definition-container spelling error is reported separately as V2-1 below. |
| C2: original draft:168, NEEDS-CHANGE / undecided | Lines 171–175 bind scanned declarations, compiled schema provider and actual owner binaries to the same source/profile receipt. Lines 205–221 distinguish discovery fixtures from semantic evidence, require matching compiled source copies, reject uncompilable setup as matrix red, require a compile-valid production optional-field accounting failure and actual production F01/control execution. | Resolved. A discovery fixture or stale binary cannot satisfy the stated production causal proof. |
| C3: original draft:179, NEEDS-CHANGE / undecided | Lines 136–145 define extractor/profile/unaccepted-baseline bootstrap, root review and exact eligibility checkpoint, followed by enforcement. Lines 229–237 preserve this staged contract and explicitly withhold acceptance from output that does not exist. | Resolved. Concrete extraction representations, consumer IDs, case IDs and eligibility contents remain staged decisions; their present absence is not a new circularity finding. |
| C4: original draft:30, CONFIRMED / undecided | Lines 29–33 name compiler-private assembly parts, explain AST visibility and require public compilation or existing compiler-local tests without widening EssIrParts or EssIr::from_parts. | Resolved. No new external callable API or visibility expansion is implied. |

No original finding is carried as an active row. Their previous records and signatures remain unchanged.

## 4. Current finding

| ID | Primary location | Category | Severity | Verdict / origin | Finding |
| --- | --- | --- | --- | --- | --- |
| V2-1 | target/review-boundaries-11/next-scope/consumer-binding-draft-v2.md:51 | contract-drift | warning | CONFIRMED / introduced | The new wire-identity wording names an actual $defs key although the locked schemars 0.8.22 provider serializes definitions and uses #/definitions/ references. |

What was observed: v2 lines 49 and 51 say properties/$defs and “the actual $defs key.” The same paragraph makes literal JSON-pointer locations and local reference resolution authoritative, while lines 66–71 require a closed grammar matching actual current schemars output. The current owned provider at crates/edge/ess-xtask/src/main.rs:425 calls schemars::schema_for!(ess_domain::spec::RawSpecFile), serialized at :426. Cargo.lock:1110–1113 pins schemars 0.8.22 and its package SHA256.

The locked package's exact source proves the relevant vocabulary without running a generator:
- schemars-0.8.22/src/macros.rs:41–43 uses SchemaGenerator::default().into_root_schema_for.
- src/gen.rs:53–66 selects Draft 7 with definitions_path equal to #/definitions/; :150–153 derives the generator default from those settings.
- src/gen.rs:329–335 places collected definitions into RootSchema.
- src/schema.rs:101–109 explicitly documents and serializes the field as definitions. The serde alias for $defs is a deserialization alias, not a serialized-key rename.

What reaches it: the draft itself selects that same schema_for!(RawSpecFile) provider at :36 and literal generated-root JSON pointers at :48. The current xtask caller is a concrete source route to that provider. This is a static declaration/contract contradiction, not an assertion that a future unimplemented extractor currently fails, and there is no executed exit status for schema behavior. Reading committed generated schema bytes was not used as schema authority.

Consequence: implementing the literal $defs spelling can misidentify the definition-map pointer or reject current definitions under the required closed grammar. The surrounding “actual current output” requirement may lead an implementor to choose correctly; that makes this a narrow warning to correct before the vocabulary becomes an accepted profile, rather than an implementation feasibility blocker.

Minimum correction: name the actual locked Draft 7 definitions map, its /definitions/<key> pointer positions and #/definitions/<key> local references. Keep RFC6901 escaping and the existing reference-resolution rule. Alternatively, state “the actual definition-map key” and explicitly identify current definitions in the selected profile. Do not rename emitted schemas or silently admit $defs as an additional dialect. The existing future profile/keyword freeze remains the place to enumerate exact grammar; no broader format migration or new execution is needed for this wording correction.

Origin: introduced refers solely to the new v2 document paragraph, established by comparing the exact immutable v1 and v2 bytes. It does not attribute a production regression. The source behavior predates this candidate; no base implementation was executed.

## 5. Bounded checks with no additional finding

- C1's separate tagged identities remove the need for a fabricated total Rust-to-wire correspondence; literal data and schema annotations now have distinct handling.
- C2's receipts bind AST, schema provider and behavioral binaries; compile failure or fixture-only output does not count as production causal evidence.
- C3 now allows extraction output to exist and be reviewed before exact unknown eligibility is admitted. It neither preapproves that output nor regenerates eligibility in ordinary gates.
- C4 respects the actual compiler-private assembly boundary.
- The declared acceptance remains finite public-entry-point/family discovery, not whole-Rust-callgraph proof. No whole-program completeness claim was required or established.
- Future concrete profiles, IDs and qualifying cases are staged implementation freeze decisions. This review neither executes them nor treats their future output as accepted.
- Existing nested-runtime fidelity limits remain explicit. An outer Rust pass or skip cannot silently become browser/Go/TypeScript execution evidence under the revised wording.

No new writer reports or outputs were inspected. This pass does not qualify an actual ESS producer, execute an AEP helper, review a new runtime implementation or expand the active writer's scope.

## 6. Preservation, exact records and relinquishment

All 58 previous input files and the seven additional records/dependency files have matching before/after SHA256 values. The exact source/hash manifest is input-manifest.json in this directory and includes the sealed report's own SHA256. In particular, Cargo.lock remains 8ca4848311f5c82eef7e170f3b8562fe0132b874b9b6f630f2bd28e7d7fc5842 and ess-xtask/src/main.rs remains 24061b8a4ca4402228da0796fa75ee7488e1bd5127b372ab6315d7709438f6b5.

Additional primary dependency identities:
- /home/timo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/schemars-0.8.22/src/macros.rs: SHA256 1e05381f227b9c4b830e2191e20697ed7448599f45e5df41b65dd8967d8a70ab.
- /home/timo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/schemars-0.8.22/src/gen.rs: SHA256 48d9c76f34cbe039a3ee878f239bc2e4318c9144b53fb1d36b28019e4bd5c008.
- /home/timo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/schemars-0.8.22/src/schema.rs: SHA256 1e368bf8110defaee419cf6d81dc8a9380f095677251e778d03c56cc63f5b015.
- /home/timo/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/schemars-0.8.22.crate: SHA256 3fbf2ae1b8bc8e02df939598064d22402220cd5bbcca1c76f7d6a310974d5615, matching Cargo.lock; cited cached source bytes match the archive members.

Only these new files were written, both beneath the exact assigned ESS coordinator scratch:
- target/review-boundaries-11/next-scope/consumer-binding-review-2/correction-review-report.md
- target/review-boundaries-11/next-scope/consumer-binding-review-2/input-manifest.json

Outside-worktree writes: none. Existing draft versions, first review records, sources, tests, manifests, planning records, Git state and build outputs were not edited. No cleanup occurred. This completes the assigned second document pass; all scratch and source write authority is relinquished on handoff.

```findings
- file: "target/review-boundaries-11/next-scope/consumer-binding-draft-v2.md"
  line: 51
  category: "contract-drift"
  severity: "warning"
  verdict: "CONFIRMED"
  origin: "introduced"
  message: "The new wire-identity wording names an actual $defs key although the locked schemars 0.8.22 provider serializes definitions and uses #/definitions/ references."
```
