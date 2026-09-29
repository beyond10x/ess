---
format: aep.planning-md/3
id: review-result:adversary-c-binding-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit binding-context
relations:
- reviews: story:bindings-read-the-delivery-context
revision: 1
---
unit: binding-context (beyond10x/ess#195), working tree ~/.local/state/worktree/trees/b10x/ess/ess-c-binding (base 969394c28e + uncommitted diff, as handed over)
verdict: NEEDS-CHANGE
cases: executed 1795→1801, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (listed in part 6)
needs-coordinator: yes. ess-diff/10 registration is not in decisions.md (F3); a format bump belongs to whichever unit the coordinator names

## 1. Diff stat

Tracked diff after my pass: `43 files changed, 966 insertions(+), 86 deletions(-)`. This is the same as before I started. I added two untracked test files and nothing else:

```
?? crates/verify/ess-conformance/tests/adv1_delivery_context.rs
?? crates/verify/ess-diff/tests/adv1_delivery_context.rs
```

No non-test path was touched.

## 2. Cases added (each run alone before any suite run)

| file | case | asserts | now |
|---|---|---|---|
| ess-conformance/tests/adv1_delivery_context.rs | `adv1_a_target_that_reads_the_same_named_payload_field_fails` | The payload also has `account_id: String`. A target that fills `account_id` from the payload fails some binding scenario. | red |
| same | `adv1_a_target_that_fills_an_integer_context_input_from_the_payload_fails` | Context `shard: Integer` and payload `sequence: Integer`. A target that fills `shard` from `event.sequence` fails. | red |
| same | `adv1_a_target_that_swaps_two_integer_context_fields_fails` | Context `region, shard: Integer`. A target that swaps them fails. | red |
| same | `adv1_control_swapping_two_string_context_fields_is_caught` | Harness control: the same swap with `String` fields is caught. | green |
| ess-diff/tests/adv1_delivery_context.rs | `adv1_an_external_cause_is_not_writable_for_a_format_that_predates_it` | `to_canonical_json_for(ess-diff/3..9)` refuses a delta that holds an `external` cause. | red |
| same | `adv1_a_context_only_change_describes_what_changed` | For an authority-only change, the rendered before and after texts differ. | red |

In every conformance case, the correct target passes all four binding scenarios before the mutant is run.

Red output, verbatim, from `cargo test -p ess-conformance --test adv1_delivery_context`:

```
mutant `payload lookup by the context field's name` passes every delivery-context scenario: {
    "received/binding/delivery": Passed,
    "received/binding/flow": Passed,
    "received/binding/mapping": Passed,
    "received/binding/on-failure": Passed,
}
mutant `region and shard swapped` passes every delivery-context scenario: { ...all four Passed }
mutant `shard read from event.sequence` passes every delivery-context scenario: { ...all four Passed }
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Red output, verbatim, from `cargo test -p ess-diff --test adv1_delivery_context`:

```
assertion `left != right` failed: the rendered change does not say what changed: cause or required host contract changed: `demo.inbox.MessageReceived` → `demo.inbox.MessageReceived`
a delta holding an `external` cause is written as ess-diff/3, whose readers do not know the variant; current format ess-diff/3
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 3. Suite runs (after the cases existed)

`<before>` comes from the implementor's own logs, `binding/after2-ess-conformance.log` and `binding/after-ess-diff.log`: 1589 + 206 executed.

| command | summed result lines | final lines, verbatim |
|---|---|---|
| `cargo test -p ess-conformance --no-fail-fast` | passed 1590, failed 3, ignored 3 | `test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` / `error: 1 target failed:` / `EXIT=101` |
| `cargo test -p ess-diff --no-fail-fast` | passed 206, failed 2, ignored 0 | `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s` / `error: 1 target failed:` / `EXIT=101` |

The only failures are my 5 cases, and no existing case regressed.

## 4. Findings (tree: working tree above)

| file:line | severity | verdict | finding | fix |
|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/delivery_context.rs:178 | blocker | NEEDS-CHANGE | Payload and context witnesses come from two independent `witness::fields` builders. A payload field with the same name as a context field gets the same `String` witness (the path), in both occurrences. A target that looks up the context in the payload passes all 4 scenarios. The issue rules this out: "A missing context is a refusal, never a lookup in the payload." What reaches it: any model whose event payload carries a field named like a context field, such as the sender's `account_id` next to the subscription's. | Build context values in a namespace disjoint from the payload, for example path `context.<field>`. Or refuse the scenario when a mapped context value equals any payload value of the same type. |
| crates/verify/ess-conformance/src/witness.rs:401 | blocker | NEEDS-CHANGE | The new `fields` doc says "two fields of one table differ". That holds only for `String`, `Uuid` and `Json`. `Integer`/`Decimal` is `1 + ordinal`, and `Timestamp`, `Duration`, `Bytes` and `Boolean` also ignore the path (`primitive_value`, :2626-2656). So two `Integer` context fields, or an `Integer` context field and an `Integer` payload field, carry the same value in each occurrence. Swap and wrong-field mutants pass everything. `contexts_differ` (:141) only compares the two occurrences, never fields with each other. What reaches it: any context with a non-text type beside another field of that type. | Give every mapped field a distinct value across payload and context together, for example by per-field offset. Or refuse with a reason when two mapped sources of one type collide. |
| crates/verify/ess-diff/src/change.rs:419 | warning | NEEDS-CHANGE | A `CauseChanged` holding `BindingCause::External` is still minimum `ess-diff/3`. `to_canonical_json_for(ess-diff/3..9)` succeeds and writes `{"external": …}`, which no released reader of formats 3–9 knows. Such a reader fails with an unknown-variant parse error, not `unsupported_format_version`. That is loud, not silent, but it breaks the reader contract in `raw.rs`. `ess-diff/5` bumped for new `TypeChange` cases, which is the precedent. **ess-diff/N must be bumped: yes, to `ess-diff/10`**, as the minimum for a cause change that holds `external`. | Add `SUPPORTED_DELTA_FORMATS` 10 and use it as `minimum_format` when either side is `External`. The coordinator registers it. |
| crates/specify/ess-domain/src/binding.rs:207 | warning | NEEDS-CHANGE | `Display` for `BindingCause::External` prints only the event. `ess verify diff` therefore renders a context-only change as "`X` → `X`". The same happens when a context is added to or removed from the same event. What reaches it: any edit to `context_authority` or `context_fields`. | Show authority and fields in `Display`, or give the context change its own `BindingChange` text. |
| crates/specify/ess-domain/src/binding.rs:774 | note | CONFIRMED | `MappingSource::parse` now reads `context.<x>` as a context reference in every format. An `ess/17` document with the literal text `context.foo` is now refused where it used to validate, so an older format's meaning changed. There were 0 hits in the repo's own yaml/json corpus, and `host_context.` has the same ungated precedent. This is disclosed in changelog "Changed". | Gate the parse at `ess/18` (literal below), or keep the change as disclosed. |

## 5. Attacked, not broken

- A context ignored entirely, or the first context used on a redelivery when a payload field is mapped, is caught by existing cases.
- A swap of two `String` context fields is caught (my control is green).
- Go/TS refusal: `emit` and `emit_input` in both `go/mod.rs` and `ts/mod.rs` call `delivery_context::refuse_generation`, and the `*_with_model` wrappers delegate to them. The code `UnsupportedTarget` has the same shape as direct-response.
- Unsupported path: `target_failure` (runner.rs:3240) turns only `is_unsupported()` into unsupported, and every other error is recorded as errored. An unsupported `observe_invocations` in `expect_every_invocation` is recorded as unsupported, not passed.
- Older ess-conformance readers: suite/30 and /31 fall outside the old 1–29 major check and are refused by major, so nothing is silently misread.
- Byte stability for models without the key: I did not byte-compare against the base (this is inferred). The new IR field has `skip_serializing_if = "Option::is_none"`, the suite format moves only when `delivery_context::used_by`, and no committed generated artifact is in the diff.
- Validation: a context field named like a payload field is admitted, and correctly so, because the namespaces are separate; the hazard is in conformance (F1). `context_authority` is a free binding-local name, and ESS has no channel registry to check it against. `context_authority` alone below ess/18 gives `MissingDeclaration`, not `UnsupportedFormatVersion`; it is refused either way.
- Already disclosed by the implementor and not re-reported: with no payload field mapped, a redelivery that uses the first context cannot be observed.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/binding/adv1/review.md
- ~/.cache/ess-wave-n2/binding/adv1/red-ess-diff.log
- ~/.cache/ess-wave-n2/binding/adv1/red-ess-conformance.log
- ~/.cache/ess-wave-n2/binding/adv1/suite-ess-conformance.log
- ~/.cache/ess-wave-n2/binding/adv1/suite-ess-diff.log
- ~/.cache/b10x-target/ess-c-binding (the assigned build dir, shared with the implementor; test binaries were added)

Also outside the worktree: the worktree lease `adv1-binding-context` (released at report time), and the harness task output ~/.cache/claude-tmp/claude-1000/-home-timo-beyond10x/c8c080b7-d6ba-44b3-a1f1-53496d986908/tasks/bbg5q1tn4.output.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
  line: 178
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: payload and context witnesses are built independently, so a payload field named like a context field carries the same value and a target that reads the context from the payload passes every scenario
- file: crates/verify/ess-conformance/src/witness.rs
  line: 401
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: non-text witnesses ignore the field path, so two Integer context fields, or an Integer context and payload field, carry one value and swap and wrong-field mutants pass every scenario
- file: crates/verify/ess-diff/src/change.rs
  line: 419
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a cause change holding the new external variant is still minimum ess-diff/3 and is written for formats 3 to 9 whose readers do not know the variant, so ess-diff needs a /10
- file: crates/specify/ess-domain/src/binding.rs
  line: 207
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Display of an external cause prints only the event, so a context-only change renders as the same text on both sides of the diff
- file: crates/specify/ess-domain/src/binding.rs
  line: 774
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: context-prefixed text is parsed as a context reference in every format, so an older-format document with that literal is now refused
```
