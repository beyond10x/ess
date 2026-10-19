---
format: aep.planning-md/3
id: specification:the-wrong-state-scenario-id-is-the-unknown-identity-fallback
kind: specification
status: draft
title: The <Command>/outcome/<wrong-state> id is the unknown-identity fallback; wrong states are the illegal-move family
summary: Decline of downstream finding 14 with the idiom, run on the integration build
tags:
- adopter-report
relations:
- informed_by: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Statement

`<Command>/outcome/<wrong-state>` is not a stable name for "the command refuses a record in the
wrong state". It is the scenario for an identity no record carries, filed under the `wrong_state`
branch only while the command declares no other answer for an unknown identity. Once the command
declares one — `unknown_instance: true` on a branch, or an `external:` refusal whose error carries a
field of the identity's type — that scenario moves to that branch's id. The wrong states themselves
are covered, before and after, by the illegal-move family `<Entity>/state/<S>/refuses/<Command>`,
one scenario per refused state.

Answers finding 14 of a downstream specification report (2026-10-19): adding `unknown_instance:
true`, or an `external:` refusal whose payload copies the addressed identity, "removes" the
`<Command>/outcome/wrong-state` id that the adopter's plan stories named. Decision: **decline, with
this idiom** — the behaviour is the documented design.

## Sources

- `website/docs/guides/specify/commands-and-outcomes.md:28-34`: an unknown instance is answered by
  the not-found outcome when one is declared; "Only a command declaring no not-found outcome falls
  back to its `wrong_state` branch, checked under the branch's own id".
- `docs/design/typed-literals-and-unknown-instances.md:83-86`: the `wrong_state` id's scenario
  "covers the one case that family cannot arrange" (the unknown identity); the refused states are
  the illegal-move family.
- `crates/verify/ess-conformance/src/synthesize.rs` (`unknown_instances`, ~12239-12331).

## The idiom

Name the illegal-move ids for wrong states in plans and acceptance, and the declared not-found
branch's id for an unknown identity:

- wrong state: `<Entity>/state/<S>/refuses/<Command>` (one per state the move does not start from);
- unknown identity: `<Command>/outcome/<not-found branch>`, or `<Command>/outcome/<wrong-state>` only
  where the command declares no not-found answer.

## Run

Neutral models `.engineering/repro/wrong-state-id-dropped/base.yaml` and `unknown-instance.yaml`
(the same model plus `unknown_instance: true` on a `Play` refusal branch), integration build (0.62.0
plus the 2026-10-13 port), 2026-10-19. Scenario ids for `Play`:

```console
base.yaml:             Item/state/Played/refuses/Play  Play/outcome/played  Play/outcome/wrong-state
unknown-instance.yaml: Item/state/Played/refuses/Play  Play/outcome/played  Play/outcome/unknown-item
```

`Item/state/Played/refuses/Play` is in both suites. (Both also carry `ESS-SYNTH-004` for
`Item/state/Discarded/refuses/Play`; `Discarded` is reached only through another command's
`affects:`, which is `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` and the
`affects:` writer half of https://github.com/beyond10x/ess/issues/553.)
