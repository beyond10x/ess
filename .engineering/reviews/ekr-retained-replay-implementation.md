unit:                   story:retained-command-result-replay — exact retained command results
verdict:                green
cases:                  executed 1673→1696, red 0; seven pre-existing ignored cases remain
origin:                 n/a
wrote-outside-worktree: assigned scratch, dedicated target, assigned TMPDIR and Go cache; private-inventory.md lists full paths
needs-coordinator:      yes — independent review, full repository/consumer/site gates, version/release and adopter activation remain coordinator-owned

## 1. Unit and acceptance

Implement the adopted retained-command-results contract: an additional command-local success replays its actual originating typed response and identity, with no new direct event, error or subject change. Source ess/7, ordinary conformance/12, coverage/13 and diff/6 are explicit; older source/IR/suite acceptance and canonical bytes remain covered. No IR/2 was introduced.

The coupled implementation covers strict model admission and resolution; effect-free finite held-state refusals; actual original invocation/capture followed by an unforced retry; strict replay sequence and original identity authority; lossless native i64 response observation; native Rust/Go generation; Rust/Go execution; explicit TypeScript/browser refusal; projections, graph/diff accounting and generated schema. The real adopter-shaped external Commit/stale route is included. Source7 ordinary WrongState refusals now require complete subject snapshots and zero direct events through compiler-minted complete_refusal metadata; older sources omit that marker and retain their bytes.

Scope was checked against actual source. The CLI consumer relation roster needed one test-only expansion and its unsupported-major probe moved from newly admitted ess/7 to ess/8, both within the subsequently approved exact file. Schema metadata required a separate coordinator review of three container pins. No CLI production change, new dependency, generic lookup authority or global numeric decoder change was made.

Owners: the primary implementor owns the Rust/compiler/generator source and tests. The explicitly dispatched Go implementor owns the four-file Go runtime/fixture split; its combined report and unchanged source hashes remain separately retained. The coordinator owns the adopted design, planning, capability catalogs, website prose, diagnostic inventory, reviewed schema-container pins and metadata amendment. Those inherited edits appear in the whole-tree inventory but are excluded from the primary's authored source patch.

## 2. Observed change

The exact output of git --no-pager diff --stat is retained in diff-stat.txt. It describes tracked changes only; untracked-source-files.txt separately lists the new source and fixture files. authored.patch includes tracked source changes and each new source file. source-final.sha256 pins every modified/new source file; whole-tree-final.sha256 additionally includes inherited coordinator files. No commit was created.

## 3. Behavioral red evidence

All raw output is retained; setup/compiler failures are not substituted for behavioral reds.

```sh
cargo test --offline --locked -p ess-domain --test retained_replay command_local_replay_retains_the_original_create_identity_without_input_id -- --nocapture
```

red-model.log, exit101:

```text
a document-only create has an observed original identity: "unknown field `replays`, expected one of `name`, `when`, `when_subject`, `when_subject_state`, `when_state_changes`, `external`, `wrong_state`, `refuses`, `creates`, `moves`, `updates`, `preserves`, `instance`, `emits`, `payload`, `sets`, `error`, `summary`, `refs`"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

```sh
cargo test --offline --locked -p ess-conformance --test retained_replay replay_synthesis_captures_original_result_and_identity_before_real_retry
```

red-synthesis-final.log, exit101:

```text
[Refusal { subject: Outcome { name: OutcomeRef { command: CommandRef(QualifiedName(retained.core.Seed)), outcome: OutcomeName(replayed) } }, scenario: Some(Outcome { outcome: OutcomeRef { command: CommandRef(QualifiedName(retained.core.Seed)), outcome: OutcomeName(replayed) } }), cause: StrategyWithoutGuard { strategy: ReplayResult } }]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Additional deciding reds: red-state-default-behavior.log (finite refused states omitted), red-asyncapi-behavior.log (retained relation missing from projection), red-external-stale.log (three actual Stale obligations refused as Unwitnessable), red-identity-field.log (malformed identity authority field admitted). Each command's full test-runner output is preserved. The Go split observed suite12 refusal before implementation (14 failed outcomes), then actual identity substitution failures. Its final adopter witness exposed unknown events and subject mutation passing Validate/Stale: 108 passed/3 failed out of111 before the source7 complete-refusal correction; its unchanged assertions then passed.

Some earlier files contain fixture or compilation errors: red-source7-refusal.log removed a command while leaving its transitions and is not acceptance evidence. Early synthesis/state/AsyncAPI setup logs are likewise retained without being counted as behavioral proof.

Five distinct deliberate Rust mutations were caught (six executions because the native integer mutation was strengthened): removed exact result equality; removed identity binding admission; ignored ambiguous original identity events; converted actual native i64 through f64; disabled source7 complete-refusal preservation. Every mutation run reports 0 passed/1 failed, exit101. The stronger integer witness observes 9007199254740993 becoming9007199254740992. Mutation files were restored byte-for-byte before final suites. The Go combined report records eight caught artifact mutations, including a missing actual Stale/Commit obligation.

## 4. Executed verification

Commands use the dedicated assigned target, jobs2, incremental/debug disabled, assigned TMPDIR and the coordinator-provided TypeScript/Node declaration toolchain. Full commands and exact check statuses are in verification-commands.txt; full runner outputs are named below. Counts are aggregated only from runner summary lines (Go counts use terminal test/subtest records, excluding package records).

The baseline and final six-package selections are identical:

```sh
cargo test --offline --locked -p ess-domain -p ess-compiler -p ess-conformance -p ess-synth -p ess-gen -p ess-diff
cargo test --offline --locked -p ess-domain -p ess-compiler -p ess-conformance -p ess-synth -p ess-gen -p ess-diff --no-fail-fast
```

baseline.log: 1673 passed,0 failed,7 ignored across144 summary lines, exit0. packages-final.log:1696 passed,0 failed,7 ignored across148 summary lines, exit0. --no-fail-fast changes failure continuation only. No old cases were marked ignored. After the final lint-only helper extraction, the full ess-diff package ran again successfully (its182 passed precede the separately failing xtask lane in diff-xtask-final.log).

| Lane | Observed execution | Exit / evidence |
|---|---|---|
| Six complete affected packages | 1673→1696 passed; seven existing ignored | 0, baseline.log and packages-final.log |
| Rust retained replay target | initial synthesis0pass/1fail→15pass | 0, source7-refusal-green2.log; counts increased as accepted scope grew |
| Generated Go runtime, original+coverage+held states+actual external Stale | 14 failed→105pass→111pass,0failed/skipped | 0, Go combined-report.md and raw JSON; ordinary Rust integration now emits and runs the canonical fixture |
| Native generated Rust fixture |1passed,0failed |0, native-rust.log; generated target absent on base, no invented baseline count |
| Native generated Go fixture |1test passed; two packages have no test files |0, native-go.log; generated target absent on base |
| CLI consumer model ingress |23pass/1fail future-version probe→24pass |0, cli-ingress-green.log; untouched-base count was not measured for this later-approved lane |
| Full xtask package, native authority profile |217passed,0failed,3pre-existing ignored; untouched-base count was not measured |0, xtask-native-final.log |
| Scoped eight-package all-target clippy |warnings denied |0, clippy-final2.log |
| Scoped eight-package rustdoc |warnings denied |0, rustdoc-final.log |
| Scoped fmt and git diff check |all selected files |0, fmt-final.log and diff-check-final.log |
| Schema, documentation catalog, legacy generation checks |all three commands |0, schema-check-final.log, docs-final.log and generate-check-final.log |
| Legacy generated Rust preservation |git diff --exit-code -- generated |0, generated-restoration-final.log |

The first combined diff/xtask run was red:320 passed/9 failed. Eight failures were reviewed schema metadata drift and one was the deliberately strict provider's refusal of CARGO_TARGET_DIR. The unchanged wire extractor measured the new container hash with a temporary diagnostic test (1passed), which was removed and restored byte-for-byte. Root reviewed only the three container rows. The final xtask command follows Taskfile's native flags and uses --target-dir while unsetting the prohibited environment variable; no authority guard was relaxed.

The actual EKR draft development preflight was run by the coordinator, not this worker: validate/compile/synthesis all0,33generated/0refused, suite13, with both retained retries and actual Stale refusal observation. This supports compiler compatibility only; it is not EKR runtime evidence.

Release-readiness inspection found the new generated Go fixture requested1.26 while CI provisions1.25.10. The unchanged actual witness failed on cached Go1.25.9 with GOTOOLCHAIN=local: `go: go.mod requires go >= 1.26 (running go 1.25.9; GOTOOLCHAIN=local)` (go125-red.log,0passed/1failed,exit101). The fixture requires only Go1.25 features, so its go.mod minimum was corrected to1.25. The subsequent complete retained target reports15Rustpassed and111Gooutcomes passed/0failed/0skipped in go125-green.log. The coordinator then provisioned checksum-verified exact CI Go1.25.10 privately: the deciding ordinary test passed1Rustcase and111actualGooutcomes,0failed/0skipped (go12510-green.log,exit0; go12510-version.log identifies the tool). Initial local Go runs used the host toolchain; the exact CI claim rests on this final explicit run.

## 5. Boundaries and disclosed correction

Full workspace task check, consumer qualification, website/release gates, independent adversary review and release selection remain coordinator work. Scoped xtask tests do not run fresh full consumer extraction/claims qualification. New source/IR/reflection declarations may require exact entry classification and claim disposition under that full gate; no such qualification was claimed or registry debt hidden. No release, commit, AEP mutation, installation, EKR live store access or adoption was performed. Immediate retry synthesis does not prove restart or later-head EKR behavior; those remain authored adopter obligations.

Exactness is certified at the actual typed target boundary. Native handler i64 values preserve adjacent integers above2^53 and both endpoints; the mutation proves conversion loss is detected. A JSON adapter must establish exact Integer decoding before Node or refuse observation. The inherited generic JSON-to-Node raw decimal/exponent rounding probe is explicitly not cross-runner parity evidence. Decimal/Binary64 recursively refuse in new replay result schemas. Global Number and legacy response decoders remain unchanged.

An accidental cargo fmt --all invocation reformatted14 pinned generated Rust files. The coordinator identified the scope violation. The exact accidental diff is retained in accidental-formatter.patch; only those generated files were restored from opening HEAD. Subsequent checks use scoped formatter commands. Final git diff --exit-code -- generated returns0; generated-restoration-hashes.txt records matching working-tree and HEAD blob identities. No legacy generated fixture regeneration was carried forward.

## 6. Outside writes and stable handback

private-inventory.md gives full private paths; written-paths.txt and target-paths.txt enumerate retained scratch and target contents. The Go worker's separate inventory covers its assigned split. No other session's scratch, cache, process or worktree was removed. The generic Go test mechanism used the pre-existing shared Go build cache; this use is disclosed and that cache was not cleaned. Final source/hash/lease status is recorded in handback-status.txt.
