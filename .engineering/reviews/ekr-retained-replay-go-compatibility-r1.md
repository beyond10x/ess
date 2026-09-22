unit:                   retained-command-result-replay — legacy Step decoding correction addendum
verdict:                green
cases:                  retained fixture executed 185→186, red 0; existing legacy setup and optional-shape lanes restored
origin:                 n/a
wrote-outside-worktree: assigned legacy-setup scratch, approved Go cache/TMPDIR and managed lease state; private inventory retained
needs-coordinator:      no source changes outside assigned ownership; primary owns final Cargo verification

## 1. Scope and regression

This addendum preserves the earlier report and its hashes as historical evidence. The primary's full package run found a real compatibility regression outside the retained-only Go fixture. Owners remain this split for the same three Go files, the primary for Rust/Cargo, and the coordinator for governance/integration.

The prior `Step.UnmarshalJSON` unconditionally selected `UseNumber`, overriding the existing outer decoder's version policy. Legacy entity setup then returned `json.Number` rows to the unchanged legacy matcher, where the original decoded values had been float64. The deciding authored fixture's actual envelope is **ess-conformance/6**, despite an earlier message calling it suite/3.

The first correction deferred shape decoding until suite admission. This restored entity setup but failed the existing public direct `json.Unmarshal(&Step)` optional-shape assertion. That intermediate failure is retained rather than hidden. Both original repository assertions and fixtures remain unchanged.

## 2. Final implementation

Direct Step decoding now retains legacy numeric and event-shape behavior. Admitted suite12/13 alone re-decode their original raw step bytes through an explicit exact helper; replay preflight uses that same helper. The suite-version loaders keep their established numeric policy. No legacy matcher, global numeric representation, response-to-event semantics, dependency, or wire allocation changed.

The existing `TestRetainedInputLiteralRemainsExact` keeps its decoded-input assertion and additionally executes the actual target. Both original and retry callbacks must receive `json.Number("9007199254740993")` exactly. The whole retained fixture gains one actual scenario subtest, 185→186 terminal outcomes.

Only `src/go/runtime.go`, `src/go/replay.go`, and `tests/fixtures/retained-replay-runtime.go` were edited. The combined current diff is in `source-combined.patch`; `source-final.sha256` identifies the exact frozen files. The original report remains byte-identical at SHA256 `271984aef1b5e36b6923e7c3b8d395a4633b3099fc57aa5276ad558b4c3ab59c`.

## 3. Deciding reds

All runs use the task-provisioned Go1.25.10 with `GOTOOLCHAIN=local`, `GOWORK=off`, `GOPROXY=off`, and approved cache/TMPDIR. No Cargo was run by this split.

```sh
ESS_REPORT_FORMAT=2 ESS_SETUP_FAULT=good go test -json -count=1 .
```

The unchanged generated entity-setup fixture in `red/` reproduces the regression. `behavior-red.log`: **4 outcomes, 1 passed, 3 failed**, exit **1**, no build failure. Both real scenarios fail:

```text
step 3: `calls.history.CallHistory` holds no row where call_id = "00000000-0000-4000-8000-000000000001", duration_seconds = 12.0
step 3: `calls.history.CallHistory` holds no row where call_id = "00000000-0000-4000-8000-000000000003", duration_seconds = 12.0
```

The intermediate suite-only hydration is independently reproduced against the unchanged `optional-shape.go` fixture:

```sh
go test -json -count=1 .
```

`optional-red.log`: **19 outcomes, 18 passed, 1 failed**, exit **1**, no build failure:

```text
--- FAIL: TestShapeLeafKeepsItsOptionalFlag
`optional: true` did not survive unmarshal
```

These are distinct measured boundaries. The final correction addresses both.

## 4. Final evidence

| Exact final lane | Passed | Failed | Skipped | Exit |
|---|---:|---:|---:|---:|
| Legacy setup, good target | 4 | 0 | 0 | 0 |
| Legacy optional shape fixture | 19 | 0 | 0 | 0 |
| Whole retained fixture, 185→186 | 186 | 0 | 0 | 0 |
| Original-row admission disabled | 12 | 13 | 0 | 1 |
| Replacement-row admission disabled | 18 | 7 | 0 | 1 |
| Actual command input rounded through binary64 | 0 | 2 | 0 | 1 |

Counts are Go JSON terminal test/subtest outcomes, including parents, not Rust wrapper counts. Final logs are `green-final-2.log`, `optional-green-final-2.log`, `retained-final-2.log`, and the three `*-checked.log` mutation logs. Every listed mutation compiles; no failure is a build error. The actual input mutation reaches the callback boundary and reports:

```text
step 0: executing `retained.core.Seed`: large Integer input lost before target callback: json.Number 9007199254740992
actual original and retry callbacks did not receive the exact large Integer
```

All seven existing legacy target modes were rerun against the final runtime without changing their assertions or expected statuses:

| Mode | Pass / fail / skip outcomes | Process exit | Actual report status |
|---|---|---:|---|
| good | 4 / 0 / 0 | 0 | passed |
| empty | 1 / 3 / 0 | 1 | failed |
| wrong | 1 / 3 / 0 | 1 | failed |
| reversed | 1 / 3 / 0 | 1 | failed |
| unsupported | 2 / 0 / 2 | 0 | inconclusive |
| absent-capability | 2 / 0 / 2 | 0 | inconclusive |
| borrowed | 2 / 2 / 0 | 1 | failed |

The two deliberately unsupported modes remain inconclusive; they are not claimed as passing conformance. Their skips are the existing expected behavior. Each mode's JSON log and report are retained separately. No negative assertion was weakened.

`go vet` exits **0** independently for the setup, optional-shape and retained modules. Exact-file gofmt diff is empty. Owned `git diff --check` exits **0**, and all final source hashes verify. No source mutation remains; mutation packages live only in assigned scratch.

## 5. Limits

The primary found both compatibility failures through its full package lane, which the prior retained-only handback did not establish. This addendum does not claim that package result, a full repository gate, consumer qualification, release readiness, or independent review. The primary reruns the affected package against these exact frozen hashes. The earlier all-new complete-row checks remain stage-sensitive after the decoding correction.

## 6. Handback and storage

All additional retained output is beneath the assigned `legacy-setup/` scratch subdirectory. Earlier report/log/hash files remain intact. The private inventory records full paths, generated copies, logs, mutation modules, reports and hashes. Approved Go cache/TMPDIR and managed lease state are the only other written roots. No Go process remains owned by this split; primary Cargo may still run. `lease-release.log` records release of only the renewed Go correction lease. The source is frozen for combined verification and independent review.
