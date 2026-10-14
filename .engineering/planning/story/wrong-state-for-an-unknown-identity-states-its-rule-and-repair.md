---
format: aep.planning-md/3
id: story:wrong-state-for-an-unknown-identity-states-its-rule-and-repair
kind: story
status: draft
title: A wrong-state answer for an unknown identity is stated with its repair
relations:
- depends_on: story:read-only-command-answers-an-unknown-instance
revision: 1
---
## Summary

An adopter reported 15 synthesized scenarios such as `demo.session.Revoke/outcome/wrong-state` ("for an identity no record carries takes `wrong-state` and reports `demo.session.Denied`") as a defect: the scenario sends an identity no step arranges instead of arranging a row in a state the move does not start from. The behaviour is the decided rule, not a defect. Nothing at the point where it is read (the scenario text, the synthesis output) says it is a rule, or how a specification declares a different answer.

## The rule, with sources (origin/main 260a3d1e3)

- `docs/design/typed-literals-and-unknown-instances.md:59-104` § 2: "A command whose input selects a `moves:` or `updates:` branch, and whose `instance:` names no record, is answered with the command's `wrong_state` outcome, when it declares one." The witness is keyed by the `wrong_state` outcome's own id. The states the outcome answers in are covered by the illegal-move family, one scenario per state.
- `website/docs/guides/specify/commands-and-outcomes.md:30-40` (fallback to `wrong_state` when no not-found outcome is declared) and `:56-68` (`unknown_instance:` from `ess/15` is the first answer for an unknown identity).
- Code: `crates/verify/ess-conformance/src/synthesize.rs:11608-11660` (order: `unknown_instance:`, then a single not-found candidate, then `wrong_state`) and `:11732-11838` (the scenario: nothing arranged, a fresh identity).

So the 15 scenarios state what the specification says. A target that answers an unknown identity with a not-found error declares that with `unknown_instance: true, error: <its not-found error>`, or with an `external:` not-found refusal whose error carries the identity. The arranged-state cases are still checked under the illegal-move ids.

## Reproducer (ess/24, neutral)

```yaml
      - name: demo.session.Revoke
        input: [{name: id, type: demo.session.SessionId}]
        outcomes:
          - {name: wrong-state, wrong_state: true, error: demo.session.Denied}
          - {name: revoked, moves: demo.session.Session.revoke, instance: id}
```

The suite holds `demo.session.Revoke/outcome/wrong-state` with no arranging step (the unknown-identity witness) beside the illegal-move scenarios for the states `revoke` does not leave.

## Acceptance

1. `crates/verify/ess-conformance/tests/unknown_instance.rs`, `a_wrong_state_answer_for_an_unknown_identity_is_noted_with_its_repair`: for the reproducer, `ess verify conform synthesize` prints one `note:` naming the command, saying that `wrong-state` answers an identity no record carries because no `unknown_instance:` or not-found outcome is declared, and naming `unknown_instance: true` as the way to declare another answer. Red on main (no such note).
2. Same file, `a_declared_unknown_instance_branch_prints_no_fallback_note`: with `unknown_instance:` declared, the note is absent.
3. `website/docs/guides/specify/commands-and-outcomes.md`: a short "Which outcome answers an identity no record carries" passage near `:30`, with the three steps of the order and the reproducer, linked from `website/docs/guides/synthesize.md:254`. `task test-xtask` passes (the diagnostics and docs pins).
4. The scenario's purpose text is unchanged, so committed suite bytes do not move. If the implementor finds a wording that names the rule without moving bytes, the change says so. Otherwise the note carries the rule.

## Not done

The rule is not changed. Arranging a row instead would drop the only witness of the unknown-identity answer that the design adopted from the `billing` targets answering `undeclared`.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`: the `Note` enum (`:276`, `:426`) and the fallback arm (`:11640-11660`) (cited).
- The CLI's note printing in `crates/edge/ess-cli` (inferred; the implementor confirms the path).
- `website/docs/guides/specify/commands-and-outcomes.md`, `website/docs/guides/synthesize.md` (cited).
- Shared surfaces with wave l: `synthesize.rs` near `:11555`, which `read-only-command-answers-an-unknown-instance` edits (same unknown-instance area). Land it after that story, or in the same tree.

## Estimate

du about 8G (ess-conformance `--test unknown_instance` plus ess-cli). Adversary: no (a note and documentation).
