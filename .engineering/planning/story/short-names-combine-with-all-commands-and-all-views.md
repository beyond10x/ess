---
format: aep.planning-md/3
id: story:short-names-combine-with-all-commands-and-all-views
kind: story
status: active
title: Short names combine with --all-commands and --all-views
summary: --names short shortens every body root to Input, Response or Row, so two commands or views collide as short_name_collision.
tags:
- ess-0.70.0
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T06:49:20Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T06:49:20Z", actor: "human:timo", revision: 7}
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/schema-contract/src/realize.rs:676`: `short_name_collision: "demo.notes.Create.input" and "demo.notes.Pin.input" both shorten to Input`; view rows collide as `Row`.
- `--names short` was asked for in #409.
- Red test: `all_commands_with_short_names_generates` (schema-contract, adversary_gu15_body_roots.rs).

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D343 on 45a1beba7. Every line is cited or inferred.

- **Primary surface:** `crates/generate/schema-contract/src/realize.rs` `Plan::with_short_names` (:875-919; the collision is raised at :905-911; the finding's :676 has drifted) — cited. `Plan::from_model` (:698) keys names by the root string `<owner>.<member>` (:745-770), so the owner is readable there — cited.
- **Body roots:** `<owner>.input|response|row` (`crates/generate/ess-gen/src/model_types.rs:296-298`) — cited; no change needed there.
- **Also likely:** the `Names::Short` doc at `crates/edge/ess-cli/src/model_types.rs:40-41` if the rule text changes — inferred.
- **Tests pinning short names:** `schema-contract/tests/model_ergonomics.rs:201,255`, `adversary_ess_issues_2_u7.rs:182`, `ess-cli/tests/model_types.rs:36`, `ess-cli/tests/client.rs:174` — cited; none combines `--names short` with `--all-commands`/`--all-views`. The red test `all_commands_with_short_names_generates` was never committed — cited.
- **Confidence:** high for file and symbol; medium for the naming rule (owner's last segment plus member, e.g. `CreateInput`) — inferred.
- **Would collide with:** a unit editing `with_short_names`/`Plan::from_model`, or `Names` in ess-cli.

## Decisions

D343, 2026-11-01 (coordinator, inside the orchestrator-approved item; the request is beyond10x/ess#409's
`--names short`, accepted as an existing flag whose combination with body roots is a defect): under
`--names short` a body root `<owner>.<member>` shortens to the owner's last segment followed by the
member (`demo.notes.Create.input` -> `CreateInput`, `demo.notes.ByOwner.row` -> `ByOwnerRow`); type
and event short names are unchanged; a collision that remains (two `Create` commands in two
domains) is still refused as `short_name_collision`, naming both. `--names short --all-commands
--all-views` then generates for a model whose commands and views have distinct last segments.
