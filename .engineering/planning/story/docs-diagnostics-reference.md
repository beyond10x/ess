---
format: aep.planning-md/3
id: story:docs-diagnostics-reference
kind: story
status: implemented
title: Every diagnostic ess prints is explained on one reference page
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:11:12Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T14:11:12Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T15:05:49Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`website/docs/reference/diagnostics.md` explains every diagnostic `ess` can print — compiler
family × class codes, ESS-AUTHOR, ESS-SYNTH, ESS-MUTATE, the dotted refusal names, and the
realization, deployment and composition diagnostics — each with its meaning and repair, generated
from catalogues the owning crates export.

## Acceptance

- `cargo xtask diagnostics --check` fails when a code exists in the code but not on the page, and
  when a catalogued code has an empty summary.
- The page has a row for ESS-AUTHOR-001 to 037, ESS-SYNTH-001 to 018, ESS-MUTATE-001 to 005, the
  compiler class table (001 UNDECLARED to 018 UNSET_AT_CREATION) with the 12 families, and the 15
  dotted refusals.
- A refusal printed by `ess` links nowhere new; the page is found from the CLI reference and the
  sidebar.

## Scope

- crates/edge/ess-xtask/src/diagnostics.rs (new)
- crates/specify/ess-compiler/src/resolve.rs (catalogue export)
- crates/verify/ess-conformance/src/authored.rs, synthesize.rs, mutate.rs (catalogue export)
- website/docs/reference/diagnostics.md (new)
