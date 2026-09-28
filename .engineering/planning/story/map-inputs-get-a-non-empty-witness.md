---
format: aep.planning-md/3
id: story:map-inputs-get-a-non-empty-witness
kind: story
status: active
title: 'A Map-typed input is always witnessed as {}, so a target that drops the value passes the sets: assertion'
refs:
- provider: github
  reference: beyond10x/ess#196
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 3}
---
# Story: A Map-typed input is always witnessed as {}, so a target that drops the value passes the sets: assertion

## Why

beyond10x/ess#196. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #196 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect is still present on `origin/integrate/ess-next` at `e932781965` ("chore: release 0.40.0"). I found it by reading the code; I did not run it. Issue #196 is still open.

**Does it reproduce, and is it already fixed**
- [cited] `crates/verify/ess-conformance/src/witness.rs:2307`: the witness builder returns an empty map for every `Map` type, `ResolvedTypeRef::Map { .. } => Ok(Node::Map(BTreeMap::new()))`. It ignores key type, value type, path and distinction, so every map input gets `{}`.
- [cited] `witness.rs:77`: the module doc states this as intended ("a map is `{}`"). It is a documented choice, so no later change has fixed it.
- [cited] `witness.rs:2458`: a `Json` input already gets a non-empty witness keyed by its path (ess#138). This is the gap the issue describes: `Json` is covered, `Map` is not.
- [cited] `crates/verify/ess-conformance/src/synthesize.rs:4985-4989`: `flatten_leaf` copies a `Map` value into the view expectation whole. That is why `{}` is asserted for both `tags` and `item.attrs`.
- [cited] `synthesize.rs:5295-5345`: `freshened` (the ess#161 logic that moves an update's input away from the prior value) gets its alternative values from `candidates(…, Distinction::further(nth))`. Those also come from `witness.rs:2307`, so they are always `{}` too, and an update that writes a map can never be moved off the prior value.

**Where the fix lands**
- [cited] `witness.rs` `Builder::value`, `Map` arm (`:2307`): build one entry. The key is a canonical spelling of the key primitive, derived from the path. The value comes from recursing `self.value(of, &path.child(<key>), …)`, the same way `list` recurses at `:2244-2275`. The instance number (`Distinction`) should be carried into the key or the value.
- [cited] `witness.rs:77`: update the doc rule.
- [inferred] `synthesize.rs` `freshened` / `held_value` (`:5295`): no logic change expected once further witnesses differ. Needs a test for an `updates:` branch that writes a map.
- [inferred] Tests: `crates/verify/ess-conformance/tests/witness.rs` (`:356-360` asserts that map `.count` is not projected into facts), `set_effects.rs`, `leaf_payloads.rs`, `adversary_leaves_pass2.rs`, and any golden suites that contain `{}` for a map input.

**Collisions with the other issues** [all inferred, from issue text plus file ownership]
- #198 and #199: `synthesize.rs` and `synthesize/subject_fact.rs` (subject and route search). They share `synthesize.rs` and the `candidates` / `witness.rs` path. Medium risk.
- #209: refusal arrangement in `synthesize.rs` (`arranged` / `prepare`, around `:2055`). Shares a file with `freshened`. Low-to-medium risk.
- #201: `ess-domain/src/command.rs` (`wrong_state`) and synthesis of state refusals. Low risk.
- #210: `mutate.rs`. No overlap.
- #195: `ess-domain/src/binding.rs` and binding witnesses. No overlap.

**Design decisions for the implementor**
1. **Key spelling per key primitive.** Keys can be `Integer`, `Uuid`, `Timestamp`, `Boolean` and so on; only `Binary64` and `Json` are refused (`ess-domain/src/types.rs:210-218`). Each generated key must pass `setup_map_key` (`input.rs:345-352`) and carry the instance number. [cited]
2. **Nested value types.** A map value can be a struct, a union or a newtype. Decide whether leaves inside the entry are recorded (`record`) as fact paths or built with `record = false`, the way a union is. Recording them means map paths reach the candidate ladders. [inferred]
3. **Guards that read `map.count`.** The count is not projected into facts (`tests/witness.rs:356-360`). A one-entry witness quietly makes a `count == 0` guard's real value false while the decision engine cannot see it. Decide whether to project the count, add an expansion like the list's "rule 3", or refuse such branches. [inferred]
4. **Scope.** The issue asks for a non-empty witness only where a `sets:` or payload value copies the map. Decide whether that applies to every map input or only to copied ones; copied-only needs the `expand`-style gating that lists use. [inferred]

Confidence: high on reproduction and the fix site (read directly); medium on collisions and the `.count` interaction (not run).

Paths:
- crates/verify/ess-conformance/src/witness.rs
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/input.rs (inferred: key spelling reference only)
- crates/verify/ess-conformance/tests/witness.rs (inferred)
- crates/verify/ess-conformance/tests/set_effects.rs (inferred)

Verdict: needs-design. The defect is open. Decisions 1-3 must be settled before implementing.
