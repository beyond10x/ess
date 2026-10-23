---
format: aep.planning-md/3
id: story:a-response-root-requires-a-declared-response
kind: story
status: draft
title: Naming the response root of a command without a response is refused
summary: --root <command>.response on a command that declares no response generates an empty type, while --all-commands and OpenAPI omit it.
tags:
- ess-0.70.0
revision: 2
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/ess-gen/src/model_types.rs:285`: `demo.notes.Pin.response` is selected as an empty object with `additionalProperties: false`.
- Expected: refused as `unknown_root`, matching `--all-commands`.
- Red test: `a_command_without_a_response_has_no_response_root_by_name_either` (ess-gen, adversary_gu15_command_view_roots.rs).
