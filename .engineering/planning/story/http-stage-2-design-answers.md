---
format: aep.planning-md/3
id: story:http-stage-2-design-answers
kind: story
status: draft
title: The ess-http/1 design page answers its stage-2 questions
revision: 1
---
## Outcome

`docs/design/http-presentation-binding.md` answers its stage-2 open questions 7, 8 and 9, so the
stage-2 units of `ess-http/1` build against settled keys. Unit k0 of wave k; it writes no code.

Part of stage 2 of `ess-http/1` for https://github.com/beyond10x/ess/issues/493
(`story:a-command-binds-to-a-protocol-or-deployment-http-route`, stage 1 shipped in 0.60.0).

## Answers to write (the recommended option of each)

- **7. A `single` view with more than one row.** Compile refuses `body: single` unless the view's
  parameters select by the source entity's identity, so a `single` route can never hold two rows at
  run time. New code `ESS-HTTP-031`, naming the route and the view. Rejected: answering the first
  row by key order (hides a model error), answering `500` (a run-time failure a compile can prevent).
- **8. `securitySchemes`.** Not in stage 2. A challenge is a response header only, until the model
  has an authentication construct (`openapi.rs:164`, `openapi.rs:171-191` keep `security` omitted).
  Rejected: deriving a scheme from each declared challenge (claims authentication the model does
  not declare).
- **9. Error fields under `{<member>: error}`.** The error's own fields under their wire names,
  unchanged. A protocol field such as `error_description` (RFC 6749 §5.2) is a field of that name
  on the model's error. Rejected: a per-field rename map in the binding (a second mapping of names
  the model already owns).

Each answer moves from "Open questions" to a "Decided for stage 2" section, as stage 1's did
(`http-presentation-binding.md:448-458`), and the stage-2 table "What compiling checks" gains the
`ESS-HTTP-031` row.

## Acceptance

- The page's "Open questions" no longer lists 7, 8 or 9; a "Decided for stage 2" section states
  each answer and the option rejected, citing the lines above.
- The stage-2 compile table lists `ESS-HTTP-031` with what it refuses and names.
- `task test-xtask` passes (the page is under `docs/`; AGENTS.md pins table, row "`docs/`").
- No failing test precedes this unit: it is a design page, the precondition for k1-k3
  (AGENTS.md, "a construct is a design page before it is code").

## Scope

| surface | confidence |
|---|---|
| `docs/design/http-presentation-binding.md` (sections at lines 226-410 and 448-478) | cited |

## Delivery

- depends on: nothing
- du: 0 (no build beyond `task test-xtask`, about 2G, inferred)
- adversary: no
