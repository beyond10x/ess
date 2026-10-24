---
format: aep.planning-md/3
id: story:a-bound-command-input-decodes-the-published-request-body
kind: story
status: draft
title: A bound command input type decodes the request body its OpenAPI document publishes
summary: Under an ess-http/1 route with path parameters, the generated input type requires the path fields and refuses the body OpenAPI publishes.
tags:
- ess-0.70.0
revision: 2
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `Message::of_command` (`crates/generate/ess-gen/src/model_types.rs:313`) keeps every input field. With a route carrying `note_id` in its path, `demo.notes.Create.input` refuses `{"isPinned":true,"title":"Buy milk"}` with `"note_id" is a required property`.
- story:generate-types-emits-command-input-response-and-view-row-types accepts on "decodes what the OpenAPI document publishes"; the 0.69.0 CHANGELOG states the exception ("Under an ess-http/1 binding the input type still holds the fields a route carries in its path").
- Changing the input type later changes generated output adopters may pin: decide here whether the bound body gets its own root (for example `<command>.body`) or the input root drops path fields, and classify it for `ess verify diff`.
- Red test: `a_bound_command_input_decodes_the_request_body_the_bound_projection_publishes` (ess-gen, adversary_gu15_command_view_roots.rs).
