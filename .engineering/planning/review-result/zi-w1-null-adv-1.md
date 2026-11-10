---
format: aep.planning-md/3
id: review-result:zi-w1-null-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-required-response-member-answered-as-null-fails-exploration
relations:
- reviews: story:a-required-response-member-answered-as-null-fails-exploration
revision: 2
---
```
unit: story:a-required-response-member-answered-as-null-fails-exploration
verdict: broken
new failing cases: 1 (crates/verify/ess-conformance/tests/explore_direct_response_json.rs::a_required_json_member_answered_as_null_passes_exploration)
mutants survived: 1 (TS explore.ts:2314, dropping the `=== undefined` clause: no case in the unit's suite answers undefined; read from the code, no mutated copy was run; my green case `a_required_member_answered_as_undefined_fails_naming_it` now covers it)
```
- **What I changed in the tree (HEAD e9695ca22f):** `git diff --stat` is empty. I added 4 new test files, all uncommitted: `tests/explore_direct_response_json.rs`, `tests/fixtures/explore-direct-response-json.yaml`, `tests/fixtures/explore-direct-response-json-target.mjs` and `tests/fixtures/explore_direct_response_json_target.go`. No implementation file was touched.
- **F1, the fix wrongly fails a valid answer (introduced, blocker, outside the acceptance text):** a REQUIRED `Json` response member answered as JSON null now fails exploration in both languages. Null is one of `Json`'s own values (`docs/design/wire-presence-json-prefix.md:38`), and both suite runtimes accept it: Go `prerequisites.go:496` `directJSON(nil)` returns nil, and TS `direct_response.ts:262`/`:176` accepts null. So the new comment at `explore.go:2682`, "the suite refuses it as `invalid_input`", is false for this type. Red output, `adv/run3.txt` (exit 101): `typescript \`null-body\`: failed: explore: seed 1: response: the target's response to \`minted\` has no \`body\`, a member the specification requires`, and the Go line is identical.
- **Origin of F1:** `adv/run1.txt` happened to run the base explorer (that binary embeds `hasOwnProperty.call(response, field.name)) {`). With the base code `null-body` passed. Who would hit this: any spec whose command response declares a `Json` member that a target can answer as null.
- **Suggested fix (not applied):** at `explore.go:2691` and `explore.ts:2314`, skip the null test when the member's declared type is the plain `Json` primitive. That matches the Go suite, which allows null only for plain `Json` and not for a newtype over it.
- **F2, shared build slots can test another tree's code (coordinator infrastructure, high):** `adv/run1.txt` linked an `ess-conformance` lib built from a sibling tree. Cargo records the files a lib depends on with package-relative paths, and the fingerprint hash does not include the tree path (all 4 `lib-ess_conformance.json` files share `"path":16525492813408108397`). Cargo then checks freshness by modification time only. A lib built from tree X therefore counts as fresh for tree Y whenever Y's sources are older than that build. Any gate or adversary run through `build.sh` can pass or fail on code it never compiled. Workaround I used: `--config 'profile.{dev,test}.package.ess-conformance.codegen-units=17'` forces a rebuild, and I checked the binary afterwards for the new check text.
- **Green cases I added and kept:** `a_required_member_answered_as_undefined_fails_naming_it` covers the TS `=== undefined` clause. `a_required_json_member_left_out_fails_naming_it` checks that a `Json` member is still presence-checked.
- **Suite run, `adv/run3.txt`:** `explore_direct_response` 5/5 ok, `explore_direct_response_json` 2 ok and 1 FAILED, exit 101. Cases executed went from 5 to 8; the 5 is from the implementor's green4.txt.
- **Pre-existing, not run (backlog):** for a newtype over `Json` answered as null, the two suites disagree. Go `runtime.go:8741` checks for nil before resolving the newtype and refuses it. TS `direct_response.ts:201-203` resolves the newtype first and accepts it.
- **What I could not break:** `optionalType` behaves the same in both languages, including through newtypes. A Go typed nil gets past both the explorer and the suite, so they stay consistent. A missing or null `response` is handled the same way in both. List and nested members are still checked for presence only at the top level, which is by design.
- **Paths written outside the worktree:** the scratch dir `.../a-required-response-member-answered-as-null-fails-exploration/adv/` (`head.rs`, `tail.rs`, `run1.txt`, `run2.txt`, `run3.txt`). Through `build.sh` I also wrote build output into the shared slots `.../ess/targets/A` and `.../ess/targets/B`. `needs-coordinator: yes` for F2.

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2691
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a REQUIRED Json response member answered as JSON null (a Json value both suite runtimes accept) now fails exploration in Go and TypeScript (explore.ts:2314), and the comment at explore.go:2682 says the suite refuses it
- file: ~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/build.sh
  category: judgement
  severity: blocker
  verdict: CONFIRMED
  origin: undecided
  message: shared slot target dirs reuse a lib built from a sibling worktree because cargo fingerprints use package-relative paths and modification times, so a run can test code it never compiled
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 8741
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the Go suite refuses a null newtype-over-Json member before resolving the newtype while the TypeScript suite (direct_response.ts:201) resolves it and accepts null
```
