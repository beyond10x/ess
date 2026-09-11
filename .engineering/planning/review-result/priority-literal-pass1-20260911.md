---
format: aep.planning-md/1
id: review-result:priority-literal-pass1-20260911
kind: review-result
status: active
title: 'Literal documentation adversary: bounded validation overclaim'
relations:
- reviews: story:docs-literal-mapping-claims-unchecked
revision: 1
---
unit: story:docs-literal-mapping-claims-unchecked at 75845afb2ffa71e65c4d781a2f14ca2011baf10d plus adversarial tests
verdict: CONFIRMED
cases: executed 34→39, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 retained scratch files; assigned build target, configured compiler cache and managed lease registry
needs-coordinator: route the false membership guarantee to implementation; review the separate admission gap without widening this unit automatically

```text
 crates/generate/ess-gen/tests/docs.rs | 107 ++++++++++++++++++++++++++++++++++
 1 file changed, 107 insertions(+)
```

## 1. Test-only boundary and candidate

The diff above is `git --no-pager diff --stat` against candidate HEAD. Only the existing test file changed. Reviewed the complete three-dot implementation diff from `59afcf8caec5230a88aadfd7c590a703e0b5500b` to `75845afb2ffa71e65c4d781a2f14ca2011baf10d`, story Acceptance, implementor report, validator representation walker and the `Docs` → `binding_section` → `mapping_bullet` caller chain. No implementation, planning, Git-state or policy edits were made.

## 2. Cases added before any test execution

All five cases are in `crates/generate/ess-gen/tests/docs.rs`:

- `adversary_literal_docs_do_not_claim_unchecked_deep_enum_membership` at line 862: 32 Optional wrappers are accepted by normal source parsing; if normal assembly admits a non-variant, generated documentation must not call its membership compiler-verified. **Red.**
- `adversary_literal_docs_do_not_claim_unchecked_newtype_enum_membership` at line 884: the same guarantee for 33 separately declared newtypes ending in an enum. **Red.**
- `adversary_literal_docs_preserve_source_enum_names_and_mixed_wrappers` at line 914: enum type wire/display aliases cannot replace its source identity or become valid enum variants. **Green.**
- `adversary_literal_docs_do_not_invent_a_representation_for_optional_cycles` at line 934: source refusal or a cycle-aware fallback is acceptable; fabricated enum/String admission claims are not. **Green.** The fixture permits source refusal, so this test alone does not establish that optional-cycle source was admitted.
- `adversary_literal_docs_cannot_be_generated_from_nontext_literal_inputs` at line 951: quoted text cannot fill Integer, Boolean, Decimal, Uuid, List or Map inputs. **Green.**

First execution selected only the first newly written case:

```console
cargo test --offline --locked -p ess-gen --test docs adversary_literal_docs_do_not_claim_unchecked_deep_enum_membership -- --exact
```

It used the assigned literal target, two build jobs, development/test debug information disabled, incremental compilation disabled and the prescribed sccache wrapper. The exact environment/path inventory is retained privately at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-private-inventory.md`.

Exit 101. Verbatim runner excerpts:

```text
running 1 test
test adversary_literal_docs_do_not_claim_unchecked_deep_enum_membership ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.01s
```

The panic reports `an admitted non-variant received a membership guarantee`, followed by generated documentation containing the literal `not_a_variant` and the exact sentence:

```text
The compiler verified that this is a declared variant of `notifications.core.Reason`.
```

Full original output, including the generated document and compiler lines, is retained unedited at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-first.log`, SHA256 `fdbcb9a09cce9045b9948418c38a871b6ceb3908fdd41d23f1fc8b7b950dadb6`. The source was parsed, assembled and compiled through ordinary public APIs; no forged IR or implementation mutant was used.

## 3. Affected suite after the cases existed

```console
cargo test --offline --locked -p ess-gen --test docs
```

Same assigned native build environment. Exit 101. Verbatim runner summary:

```text
test result: FAILED. 37 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

The two failures are the Optional-depth and named-newtype-depth cases. The before count of 34 is the implementor's retained runner result; no unchanged baseline suite was rerun. After count 39 comes from the current runner. Full original output is retained unedited at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-docs.log`, SHA256 `97a2c846d62d06a610876e0770ffcd3dd15b2249ac895c6e6e613d637d2d092c`.

The initial formatting check found only layout in the newly added test cases. Ran rustfmt on that test file; final `cargo fmt --check -p ess-gen` and `git diff --check` exit 0. These were whitespace-only test edits after the recorded run; no unchanged test rerun was performed. Final empty formatter log: `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-fmt-final.log`, SHA256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

## 4. Finding

| File:line | Verdict | Origin | Finding |
|---|---|---|---|
| `crates/generate/ess-gen/src/docs.rs:1730` | CONFIRMED | introduced | Beyond the validator's wrapper limit, the documentation claims compiler-verified enum membership for a literal that is not a declared variant. |

**What was measured:** the two new cases use declared enum variants `[declined, unavailable]` with literal `not_a_variant`. Both normal source compilations succeed, and both generated interaction documents publish the new affirmative membership guarantee. The first isolated case failed at its guarantee assertion; the affected docs suite confirms both forms fail for that reason.

**What reaches it:** ordinary specification text with exactly 32 nested Optional wrappers, or a shallow-reference chain of 33 newtype declarations, followed by normal `RawSpecFile::parse`, `Specification::assemble`, compiler resolution and `Docs` generation. The Optional input is within `TypeRef::MAX_TYPE_DEPTH`; this is reachable without bypassing public input admission. No existing consumer using this many layers was identified; reachability is established by the accepted source fixtures, not a claim about their frequency.

**Cause and class:** `ess-domain/src/binding.rs:1223` bounds `representation` to 32 iterations; exhaustion returns None, and `check_literal` at line 1135 accepts None on the assumption another pass reports an error. The new `literal_guarantee` walker is unbounded except for repeated type handles, so it reaches the enum and claims a check the validation walker never performed. The class covers both Optional layers and distinct newtype layers, including mixtures; cycle detection alone cannot address it.

**Origin:** the affirmative compiler-verification sentence is introduced by this candidate. Inspection of the base shows the old literal branch made no affirmative membership claim; the domain admission files have no diff between base and candidate. This report classifies the new documentation overclaim, not the underlying compiler admission bug. The latter was not executed separately on a base checkout and is not assigned a separate measured origin here.

**Required repair:** make the generated guarantee reflect checks actually completed by validation, including its traversal boundary, while retaining exact guarantees for ordinary checked enum literals. Review the related categorical IR comments on the same basis. Do not fix this by inventing verification evidence, silently changing the domain's admission policy inside this documentation unit, or weakening existing checked-case assertions. The underlying source-admission gap requires separate tracking or explicitly authorized scope.

## 5. Other attacked boundaries

- Existing String/newtype invariant and external-resource limitation assertions remain green in the affected docs suite.
- Mixed Optional/newtype enum traversal with distinct type wire/display names did not produce a false source identity.
- Invalid shallow primitive and structured literal targets remained refused.
- The optional-cycle probe did not report an invented representation or fail to terminate.
- Existing docs tests remained green; the only failures are the two added guarantee probes.

No package-wide, ownership, full-gate, site or release tests were run. No mutation of implementation files was performed. This review makes no approval or full-release correctness claim. Token count, tool-usage aggregate and wall-time aggregate are unavailable from this role's harness; none are estimated as measured.

## 6. Retained paths and hand-back

The nine retained scratch files are the `adversary-first.log`/`.exit`, `adversary-docs.log`/`.exit`, `adversary-fmt.log`, `adversary-fmt-final.log`/`.exit`, this `adversary-report.md` and `adversary-private-inventory.md`, all under `local-evidence:ess-evolution-20260910/priority-wave/literal/`. Full original compiler logs and the exact local build/cache/lease path inventory remain in private scratch, as required by this review's publication boundary. Only the assigned lease was acquired/renewed and released; target and tree are retained for coordinator action.

```findings
- file: crates/generate/ess-gen/src/docs.rs
  line: 1730
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Beyond the validator's wrapper limit, the documentation claims compiler-verified enum membership for a literal that is not a declared variant.
```
