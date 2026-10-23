---
format: aep.planning-md/3
id: story:short-names-combine-with-all-commands-and-all-views
kind: story
status: draft
title: Short names combine with --all-commands and --all-views
summary: --names short shortens every body root to Input, Response or Row, so two commands or views collide as short_name_collision.
tags:
- ess-0.70.0
revision: 2
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/schema-contract/src/realize.rs:676`: `short_name_collision: "demo.notes.Create.input" and "demo.notes.Pin.input" both shorten to Input`; view rows collide as `Row`.
- `--names short` was asked for in #409.
- Red test: `all_commands_with_short_names_generates` (schema-contract, adversary_gu15_body_roots.rs).
