unit:                   retained-command-result-replay — complete Go subject observation correction
verdict:                green
cases:                  executed 111→185, red 0
origin:                 n/a
wrote-outside-worktree: assigned correction scratch, task-private Go cache/TMPDIR, managed lease state; exact roots and inventory in private-paths.md
needs-coordinator:      no source patch outside assigned scope; combined Rust checks and independent re-review remain

## 1. Unit and ownership

The new complete-subject protocol must validate each actual selected row against its retained typed descriptor before original capture and before replacement comparison. Legacy snapshot semantics remain available, including mixed suite12 inputs. Owners: this split owns only `crates/verify/ess-conformance/src/go/runtime.go`, `src/go/replay.go`, and `tests/fixtures/retained-replay-runtime.go`. The primary owns Rust synthesis, admission, runtime, generated fixture data and Cargo execution; the coordinator owns planning, design, commits, integration and release.

Opening commit: `89cab6479075efa37f2578a86913f8248db96872`. The adopted complete-observation/reachable-retry addendum supplies the authority; no new product decision or dependency was introduced. All four independent review regressions and the adversary Go fixture were left unchanged by this split.

## 2. Source change

```text
 crates/verify/ess-conformance/src/go/replay.go     | 294 +++++++++++++++++-
 crates/verify/ess-conformance/src/go/runtime.go    |  57 ++--
 .../tests/fixtures/retained-replay-runtime.go      | 342 ++++++++++++++++++++-
 3 files changed, 660 insertions(+), 33 deletions(-)
```

The closed `SubjectShape` carries `identity_field`, `fields` and reachable `declarations`. Complete steps admit only in suite12/13. The new shape key is decoded separately from the existing event-shape vocabulary; coverage comparison also preserves that distinction. Admission requires the typed original/comparison pair, an already bound identity, fresh queries and exactly one intervening invocation; it refuses legacy substitution, overwritten identity/snapshot, unsupported or malformed type authority, and an incomplete pair. Generic error/no-events steps acquire no subject requirement.

Original and replacement rows each undergo the same recursive declared-field validation. Deep owned copies retain optional presence and every extra view key. The selected identity and complete row use exact numeric comparison, without changing the legacy matcher or response-to-event behavior. A valid native i64 identity above 2^53 and its adjacent neighbor are separately exercised. The original complete-row implementation's native-integer extra-key mismatch was also exposed by a positive test and corrected.

## 3. Red evidence

All Go execution used the task-provisioned **Go 1.25.10**, `GOTOOLCHAIN=local`, `GOWORK=off`, `GOPROXY=off`, and the assigned external cache/TMPDIR. Full private command expansions and raw JSON logs are retained beside this report.

```sh
go test -json -count=1 ./essconform -run '^TestCompleteActualSubjectRows$'
```

Against the original emitted runtime, `complete-rows-behavior-red-2.log` exited **1**: 25 terminal test/subtest outcomes, **6 passed, 19 failed, 0 skipped**, no build failures. Its identity-only actual rows incorrectly certified preservation:

```text
complete row outcome complete-both-identity-only: <nil>
complete row calls=2
--- PASS: TestCompleteActualSubjectRows
    --- PASS: TestCompleteActualSubjectRows/retained.core.Seed/outcome/replayed
PASS
--- FAIL: TestCompleteActualSubjectRows/complete-both-identity-only
```

The inner PASS is the invalid target result; the outer negative control correctly fails. Each original/both/retry omission and type error is a distinct subtest, so a preceding failed assertion cannot conceal another control.

Excluded setup failures are retained honestly: `complete-rows-red.log` used an older scratch export still declaring Go1.26; `complete-rows-behavior-red.log` contained my missing test brace. Neither is behavioral red evidence. The initial integer-identity experiment also stopped at an unchanged UUID event expectation; after aligning that fixture's declared event kind with its Integer identity, the isolated matcher mutation reached the deciding subject-selection failure. Only the latter is counted as integer-matcher evidence.

## 4. Green, parity and mutations

```sh
go test -json -count=1 ./essconform
go vet ./essconform
```

`baseline-full.log` re-executed the opening fixture/runtime: **111 passed, 0 failed, 0 skipped**, exit **0**. `green-final.log` executes the corrected fixture/runtime with the primary's updated eligibility artifacts: **185 passed, 0 failed, 0 skipped**, exit **0**. Counts are terminal Go JSON test/subtest outcomes, including parent tests; the final run contains 26 top-level tests. They are not Rust wrapper counts and do not double-count nested subprocess JSON. The package terminal output is `ok replay-conformance/essconform`.

The added cases cover original and replacement omissions/type errors, state enum membership, malformed/missing descriptor authority, unsupported recursive/floating observers, required identity bindings, standalone new-step envelope refusal, actual mixed legacy/complete execution, subjectless error/no-events admission, nested Struct/List/Map fields, optional absence/null, retained extra keys and native Integer identities. The inherited 111 controls remain present and green, including the external-Stale refusal obligations and response/adapter mutations.

| Lane | Executed | Passed | Failed | Exit |
|---|---:|---:|---:|---:|
| Whole fixture, opening → corrected | 111 → 185 | 185 | 0 | 0 |
| Original row check disabled | 25 | 12 | 13 | 1 |
| Replacement row check disabled | 25 | 18 | 7 | 1 |
| Descriptor semantic admission disabled | 22 | 10 | 12 | 1 |
| Exact complete identity matcher replaced by legacy matcher | 3 | 1 | 2 | 1 |

Each mutation compiles and reaches its intended assertions; all have zero build failures and zero skipped outcomes. Original-row controls require the typed diagnostic before a second command (`calls=1`). Replacement controls require that diagnostic after the retry (`calls=2`); merely failing generic inequality is insufficient. For example the replacement-check mutation reports:

```text
required actual-row type check did not decide at retry: exit status 1
step 12: complete subject changed in retained.core.Records
complete row calls=2
--- FAIL: TestCompleteActualSubjectRows/complete-retry-missing-stamp
```

Mutation copies are isolated emitted packages in assigned scratch. Managed source stayed unchanged. `sha256sum -c source-final.sha256` reports all three files OK. `go vet` exits **0**. Exact-file gofmt diffs are empty (`format.log` is zero bytes); replay.go is a Go fragment, so it was formatted with a temporary package prefix and the prefix removed. `git diff --check` for the owned files exits **0**.

## 5. Bounds and remaining ownership

No Cargo, repository-wide gate, consumer qualification, release operation, dependency change, AEP command, adversary-fixture edit or commit was performed. F2 eligibility and the deliberate source7 complete-to-legacy generation mutation are the primary Rust implementor's lane. This report does not replace the combined wrapper/package run or independent re-review.

New complete-row nested-value tests directly exercise Struct inside List inside Map. A separate valid complete-row Union payload is not added; existing retained-result recursive floating-family controls traverse Union/newtype, and complete rows reuse that finite type machinery. Generic raw JSON-to-Rust numeric transport remains outside the previously adopted certified native-target boundary. No legacy matcher or global numeric decoder was changed.

## 6. External outputs and handback

All new retained artifacts are under the assigned `correction-r1/go` scratch root: public report, raw logs, generated baseline/green/mutation modules, exact source patch/stat/hash manifests and counts. `private-paths.md` identifies full writable roots; `written-paths.txt` enumerates retained scratch files. Earlier handbacks were preserved. No Go process remains owned by this split; the primary's Cargo process may continue. The split releases only `codex-ess-retained-replay-go-correction`; `lease-release.log` records the result. The source is frozen for the primary and coordinator.
