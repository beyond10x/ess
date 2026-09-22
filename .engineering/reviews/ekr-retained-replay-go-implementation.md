unit: story:retained-command-result-replay — Go runtime and exact external-Stale adopter witness
verdict: green
cases: executed 14→105→111, red 0
origin: n/a
wrote-outside-worktree: assigned scratch and TMPDIR; private-inventory.md and written-paths.txt enumerate retained outputs
needs-coordinator: yes — complete repository gate, independent review and publication are coordinator/primary-owned

The final Go fixture reports 111 passed test/subtest outcomes, 0 failed and 0 skipped. Its explicit valid external-Stale execution reports all eight generated scenarios passing, plus the enclosing test. Go vet, formatting and whitespace checks pass. Own leases are released; source edits are stopped.

## 1. Implemented acceptance and scope

The four-file runtime split implements strict CaptureCommandResult, ExpectReplayResult and complete ExpectNoEvents for suite12/coverage13. It preserves the actual originating response, command/outcome schema, actor, input and subject identity. Required identity authority names either the original command's input field or exactly one original direct-event field. Neighboring instances, stale bindings, changed input/actor/schema, overwritten captures and missing or late queries refuse before callbacks. Runtime comparison validates complete results and rejects errors, any direct event, changed values/keys/list order, missing responses and absent-versus-null differences. Decimal/Binary64 response schemas are recursively refused.

Only these paths were edited in the combined split, all under `crates/verify/ess-conformance/`:

- `src/go/replay.go` (new).
- `src/go/runtime.go`.
- `src/go/mod.rs` (runtime fragment inclusion).
- `tests/fixtures/retained-replay-runtime.go` (new).

The last increment changes only the fixture. It models the actual adopter shape: Commit can take an external stale outcome from Validated, with no standalone Stale command. It retains actual subject state and applies the signal only to the eligible Commit invocation; it never forces retained replay. The generated runner must actually reach Commit refusals in Proposed, Rejected and Stale, plus Validate refusing Stale. The fixture requires those exact obligations to exist and every negative target to fail each applicable refusal, rather than accepting an unrelated failure elsewhere.

## 2. Stable change and provenance

`combined-source.patch` holds the complete four-file change. `stale-final-source.sha256` pins the final sources. `stale-increment.patch` isolates the final fixture extension. The first handback's complete report, source patch/hashes, passing105 evidence and fixture are preserved under `first-handback/`; `report.md` is that original detailed report. `stale-increment-report.md` details the final correction. Three source implementation files are unchanged from the first handback.

## 3. Red evidence

Initial fixture: `baseline.log`, 14 outcomes, all14 failed because suite12 was unsupported. New identity source authority also had a deciding red before implementation: three failures in `identity-red.log`.

The external-Stale witness first ran against the old reference target: `stale-red.log`, 6 outcomes, 1 passed and5 failed. It reached the external arrangement and failed because that older target rejects injected outcomes entirely. The new reference target then exposed a generated Validate/Stale gap: unknown direct events and changed subject data passed its limited refusal assertions. `stale-validate-controls-red.log` records111 outcomes with108 passing and3 failing; the exact initial suite is retained as `stale-external-before-refusal-strengthening.json`. This was measured in the active unit; no base probe establishes an origin classification.

Root directed the primary to strengthen ordinary wrong-state witnesses for ess/7 with complete subject preservation and zero direct events, preserving older-format output. The Go assertions remained unchanged and passed against the new export. That generation change and legacy-byte verification belong to the primary, not this Go-only split.

## 4. Green and deciding mutations

Actual command, from the assigned emitted module with private GOCACHE, assigned TMPDIR and GOPROXY=off:

```sh
go test -json -count=1 ./essconform
go vet ./essconform
```

Go terminal test/subtest counts:14→105→111, final111 passed,0 failed/skipped, exit0. Counts are read from Go JSON terminal records, including parent tests and subtests and excluding package records. `stale-combined-green.log` is the final complete run. `stale-actual-scenarios-green.log` separately records eight valid generated external-Stale scenarios and their parent, all passing.

Seven original behavioral mutations fail: removed result equality, bypassed sequence admission, omitted complete event refusal, lost request copy, missing fresh-query authority, omitted original identity comparison, and native int64 conversion through float64. Each uses a private emitted artifact, not repository source. Their exact counts/commands/logs remain in the first handback. The last mutation is distinct from a target returning a wrong retry result: it proves the adapter itself preserves adjacent integers above2^53 and i64 endpoints. The final combined run retains all corresponding controls.

An eighth deciding mutation deletes only the Stale/Commit refusal scenario. The suite remains structurally admissible, but the explicit obligation test fails (`stale-obligation-mutation.log`,1 executed/1 failed). Negative actual targets separately emit an undeclared event, omit the declared error, or mutate the subject; every required refusal must detect each defect.

## 5. Numeric and execution boundary

The certified boundary is the actual typed native target result: Go int64 values retain their exact integer identity, matching the primary's native Rust i64-to-Node tests. Small integral spellings1,1.0,1e0,-0,-0.0 remain admitted according to the measured Rust Node contract. The proposed lexical-only restriction was removed after its hypothesis was refuted; its scratch logs are not a fixed-bug claim. Legacy `response.go` semantics and global decoders are unchanged.

Generic Rust JSON-to-Node conversion of large decimal/exponent lexemes can round before observation. This unit does not certify that transport. An adopter must establish a lossless typed conversion or refuse observation. Immediate retry conformance likewise does not prove EKR restart/later-head behavior.

No AEP calls, dependency changes, commits or publication were performed by this split. No Cargo builds were run here; the primary now wires ordinary Cargo testing to execute the canonical generated Go fixture. Full repository and review claims remain its and the coordinator's responsibility.

## 6. Inventory and release

Private full paths are in `private-inventory.md` and `written-paths.txt`. Retained scratch includes source patches/hashes, first-handback evidence, emitted baseline/final modules, mutation modules, refuted numeric experiment, isolated Go cache and logs. Own lease releases are recorded in `lease-release.log` and `stale-lease-release.log`. The primary's lease and all unowned repository files are untouched by this split.
