---
format: aep.planning-md/3
id: story:a-response-root-requires-a-declared-response
kind: story
status: active
title: Naming the response root of a command without a response is refused
summary: --root <command>.response on a command that declares no response generates an empty type, while --all-commands and OpenAPI omit it.
tags:
- ess-0.70.0
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/command_view_roots.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:43Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:44Z", actor: "human:timo", revision: 7}
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/ess-gen/src/model_types.rs:285`: `demo.notes.Pin.response` is selected as an empty object with `additionalProperties: false`.
- Expected: refused as `unknown_root`, matching `--all-commands`.
- Red test: `a_command_without_a_response_has_no_response_root_by_name_either` (ess-gen, adversary_gu15_command_view_roots.rs).

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**. The finding's line numbers have drifted; current lines are given.

- **Primary surface:** `crates/generate/ess-gen/src/model_types.rs` — cited
- `body_root`, `RESPONSE_ROOT` arm `:365-371`, never checks presence and builds `Message::of_response` — cited. The `unknown_root` fallthrough is `select_root` `:280`, `:291` — cited. The rule to match sits in `bound_command_roots` `:185-196` (filters `response.is_empty()` at `:192-193`) — cited
- **Pins today:** `crates/generate/ess-gen/tests/command_view_roots.rs:93` pins `unknown_root` for `.input` and `.row` misses, not `.response`; `demo.notes.Pin` has no response (`tests/fixtures/command-view-roots.yaml:37-45`) — cited
- **New test:** `crates/generate/ess-gen/tests/command_view_roots.rs`, same fixture — location cited, choice inferred
- **Confidence:** high
- **Would collide with:** `model_types.rs` `body_root`, `select_root`. Row 15 is in the same file, a different function (`:513-521`) — cited
