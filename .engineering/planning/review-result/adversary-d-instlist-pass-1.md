---
format: aep.planning-md/3
id: review-result:adversary-d-instlist-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit instlist
relations:
- reviews: story:authored-instances-inside-structured-inputs
revision: 1
---
unit: story:authored-instances-inside-structured-inputs (beyond10x/ess#242), uncommitted working tree over cb6cf8493 in ess-f-instlist
verdict: NEEDS-CHANGE
cases: executed 1796→1804, red 3
origin: introduced 0 / pre-existing 0 / undecided 3
wrote-outside-worktree: 4 paths (below)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: 23 files changed, 519 insertions(+), 29 deletions(-). This is the implementor's own diff; my pass adds no tracked change. The only file I added is untracked: `crates/verify/ess-conformance/tests/adversary_instlist_pass1.rs`, a test file. No implementation path was touched.

## 2. Cases added (tests/adversary_instlist_pass1.rs)

### Blockers

None.

### Red (NEEDS-CHANGE)

| test | asserts | now |
|---|---|---|
| `a_reference_as_the_payload_of_an_identity_variant_of_a_union_authors` (:236) | `target: {kind: ring, value: {$instance: b}}` for a union whose `ring` variant is `ReleaseRingId` authors as `members{kind: literal, value: instance b}` | red |
| `a_reference_at_an_undeclared_member_inside_a_map_value_is_refused` (:258) | `by_pair: {canary: {primary: {$instance: a}, fallbak: {$instance: b}, note: x}}` (`Map<String, RingPair>`) is refused, naming `fallbak` | red |
| `a_reference_inside_a_map_value_of_the_wrong_shape_is_refused` (:280) | `by_pair: {canary: [{$instance: a}]}` is refused | red |

Red output from the first run of these cases, before any suite run (`~/.cache/ess-wave-n2/instlist/adv1/red1.log`):

```
thread 'a_reference_as_the_payload_of_an_identity_variant_of_a_union_authors' panicked at crates/verify/ess-conformance/tests/adversary_instlist_pass1.rs:242:5:
refused:
refusal[ESS-AUTHOR-022]: `release.rings/authored/aim-at-rings` in scenario.yaml
  the input of `release.rings.AimAt`: `target.value` is `release.rings.Target`, and `b` is a `release.rings.ReleaseRing`, whose identity is `release.rings.ReleaseRingId`

thread 'a_reference_at_an_undeclared_member_inside_a_map_value_is_refused' panicked at crates/verify/ess-conformance/tests/adversary_instlist_pass1.rs:264:5:
compiled; the aim sends {"kind":"members","members":{"canary":{"kind":"members","members":{"fallbak":{"kind":"literal","value":{"$instance":"b"}},"note":{"kind":"literal","value":"x"},"primary":{"instance":"a","kind":"instance"}}}}}

thread 'a_reference_inside_a_map_value_of_the_wrong_shape_is_refused' panicked at crates/verify/ess-conformance/tests/adversary_instlist_pass1.rs:283:5:
compiled; the aim sends {"kind":"members","members":{"canary":{"items":[{"kind":"literal","value":{"$instance":"a"}}],"kind":"list"}}}

test result: FAILED. 0 passed; 3 failed
```

### Green probes (the attack held; kept in `mod probes` as regression cases)

- `nested_optional_and_repeated_references_run_where_written`: covers `Optional<List<Id>>` with the same ring twice, `List<Optional<Id>>` with a null, a list of structs of lists, and a map of structs. It authors /32, runs against the interpreter, and sends each ring where it was written, with three distinct ids. `go`/`ts` `emit` and `emit_with_model` all refuse.
- `a_nested_reference_before_its_capture_is_refused`: an element that references `c` before `c` is captured gives `UnboundInstance`.
- `a_sibling_shape_error_beside_a_reference_is_still_reported`: `stages.0.name` is still refused beside an admitted reference, so `ShapeErrors::without` is not too broad.
- `hand_written_structured_values_are_admitted_only_when_well_formed`: eight malformed raw /32 mutations are all refused (items not an array, items missing, an extra key, `now_offset`/`observed`/`fixture` elements, members as an array, an element without `kind`). A relabel to /30 is refused.
- `a_coverage_suite_relabelled_31_is_refused_for_its_vocabulary`: the coverage lane gives /33. A relabel to /31, which keeps the inventory, is refused with `UnsupportedScenarioValue: members`. Go and TS `emit_input_with_model` refuse.

## 3. Suite run (after the cases existed)

Command: `cargo test -p ess-conformance --no-fail-fast`. Log: `~/.cache/ess-wave-n2/instlist/adv1/suite.log`.

```
test a_reference_inside_a_map_value_of_the_wrong_shape_is_refused ... FAILED
test a_reference_at_an_undeclared_member_inside_a_map_value_is_refused ... FAILED
test a_reference_as_the_payload_of_an_identity_variant_of_a_union_authors ... FAILED
test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p ess-conformance --test adversary_instlist_pass1`
EXIT=101
```

Sum over 251 binaries: 1801 passed, 3 failed, 3 ignored. That is 1804 executed, against 1796 in the implementor's `gate.conformance`. Every other binary is green.

Other checks on this tree:
- `cargo clippy -p ess-conformance --test adversary_instlist_pass1 -- -D warnings`: Finished, clean.
- `cargo fmt -p ess-conformance --check`: clean.
- `cargo xtask generate --check`: "projections are up to date".

## 4. Findings

| # | file:line | verdict | origin | finding | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/verify/ess-conformance/src/authored.rs:2851 | NEEDS-CHANGE | undecided | `Container::of` treats every union as `Opaque`. A reference at a union payload whose declared variant type is the identity is refused as ESS-AUTHOR-022 (`target.value is release.rings.Target`). The Outcome says "wherever the declared type at that position is an identity type". | A command input typed as a union with an identity variant. Written `{<tag>: variant, value: …}` (input.rs:452). No model in this repo shown to do so. |
| 2 | crates/verify/ess-conformance/src/authored.rs:2497 | NEEDS-CHANGE | undecided | Inside a map value, a reference at an undeclared struct member is returned as `Literal({"$instance": …})` and left "to the shape check" (doc at :2480). The shape check never walks map values (input.rs:1001). The step compiles and the target is sent `{"$instance":"b"}`. This is the silent-drop class. | A misspelt member inside any `Map<_, Struct>` input value. Only an authoring mistake reaches it. |
| 3 | crates/verify/ess-conformance/src/authored.rs:2497 | NEEDS-CHANGE | undecided | Same hole through `Container::inside()`: a map value of the wrong container shape holding a reference compiles and sends the raw `{"$instance":"a"}` inside a list. | An authoring mistake inside a map value. |

Origin is undecided for all three. I did not run the base. Reading the base, #2 and #3 look present there too: base input.rs:1001 already skips map values, and a base literal was sent verbatim. #1 at base compiled as a raw literal rather than being refused. The unit's own docs state the closing claims in #2 and #3 as done.

Fix, not applied:
- #2 and #3: pass an "under a map value" flag through `structured`. Where it is set, refuse `Slot::Undeclared` itself (UndeclaredField / WrongShape naming the `[key]` position) instead of deferring to a shape check that never runs there.
- #1: add `Container::Union { tag, variants }`. Read the tag literal and type the payload key (`value`, or `content` when the tag is `value`) as the variant's type.

Notes, not raised as findings:
- The raw refusal for list/members under a pre-/32 major is `UnsupportedScenarioValue: members`. It does not name the required format; `structured_values::REQUIRES` is only reachable through the typed `admit`. This matches its siblings.
- `now_offset::visit` now also visits `ExpectEveryInvocation`. That changes `now_offset::used_by` and `Resolved::fix` for that step. It looks like an incidental fix, but the diff does not mention it.
- `coverage-player.js` `valueText` was not taught `list`/`members`. It cannot be reached: the coverage replay admits only /5 and /9 (coverage-admission.js:521).

## 5. Attacked and not broken

- `Optional<List<Id>>`, `List<Optional<Id>>` with null, and a nested list of structs of lists: all author, run, and are sent in place (probe).
- The same instance twice in one list is sent twice (probe).
- A reference before its capture is refused as `UnboundInstance` (probe).
- Event-payload claims holding references are refused with `NotComparable` per position (unit test). Error claims share `literals()` (authored.rs:2075).
- View `contains`/`excludes`/`at` and view params resolve through the runner's one `resolve` (runner.rs:2967).
- Hand-written malformed /32 values are refused (8 mutations), and so are /30 and /31 relabels (probes).
- Go/TS: `emit`, `emit_input`, `emit_with_model` and `emit_input_with_model` all refuse (probes).
- Suites without the construct keep their bytes: `generate --check` is clean, and the unit's plain-suite case stays below /32.
- Count and coverage readers: `count_json` includes /32 and /33, and `coverage.rs` includes /33. The coverage-lane /33 admits and runs (unit test).
- A map key holding `{$instance}` is out of the acceptance (values only), and `Node` keys are text. Not attacked further.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/instlist/adv1/red1.log
- ~/.cache/ess-wave-n2/instlist/adv1/probe1.log
- ~/.cache/ess-wave-n2/instlist/adv1/suite.log
- ~/.cache/ess-wave-n2/instlist/adv1/review.md
- Build output went to the assigned `~/.cache/b10x-target/ess-f-instlist`.

I took no worktree session lease.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2851
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: undecided
  message: a reference at a union payload whose variant type is the identity is refused, because Container::of treats every union as opaque
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2497
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: undecided
  message: inside a map value, a reference at an undeclared struct member compiles and sends {"$instance":…} verbatim, because the deferred shape check never walks map values
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2497
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: undecided
  message: inside a map value of the wrong container shape, a reference compiles and is sent as a raw {"$instance":…} literal
```
