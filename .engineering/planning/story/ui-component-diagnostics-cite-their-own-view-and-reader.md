---
format: aep.planning-md/3
id: story:ui-component-diagnostics-cite-their-own-view-and-reader
kind: story
status: draft
title: UI component diagnostics cite their own view and reader
summary: ESS-COMPONENT-005 cites one file:line for every unplaced view; overlay_params flags a parameter only the submission reads (0.77.0)
tags:
- defect
- ess-batch-6
- ess-next
- ess-ui
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 7
---
## Outcome

An author fixing an ESS-COMPONENT-005 refusal is pointed at the view it is about, and `ess ui check`
no longer refuses a form overlay whose parameters only its submission reads.

## Problem

Reported on 0.77.0 by a downstream adopter; both halves reproduced on 0.92.0.

**ESS-COMPONENT-005 cites one place for every view.** A command-line component that owns two
domains, each declaring a view in its own file (`domains/pass.yaml` declares `shop.pass.ExpectedVisits`,
`domains/audit.yaml` declares `shop.audit.Entries`), with a `cli.groups` tree that lists commands and
no views:

```
$ ess specify validate --path spec
  - error[ESS-COMPONENT-005]: `shop-service` projects `shop.audit.Entries` and the command tree gives it no place
  --> components.yaml:2:5
  - error[ESS-COMPONENT-005]: `shop-service` projects `shop.pass.ExpectedVisits` and the command tree gives it no place
  --> components.yaml:2:5
```

Both point at the component, because the error is located at `component <name>`
(`crates/specify/ess-domain/src/component.rs`, `validate_command_line_views`). The hint offers two
repairs, "give it a verb" (the component's tree) or "stop projecting it" (the view), and only the
first is where the location points.

**`overlay_params` refuses a parameter the form submission sends.** A drawer form overlay opened by
a row action:

```yaml
rename:
  kind: drawer
  component: form
  title: Rename
  does: items.Rename
  params: {item_id: row.id, unused: row.name}
  fields: [{field: name, label: Name}]
```

```
$ ess ui check --path ui.yaml
error overlay_params pages/items/overlays/rename: `params.item_id` is passed to this overlay and read nowhere inside it; it reads no parameter
error overlay_params pages/items/overlays/rename: `params.unused` is passed to this overlay and read nowhere inside it; it reads no parameter
exit 1
```

Yet `ess ui test` with `act` on the row action, `type` into `name`, `act` on the overlay and
`expect_command: {command: items.Rename, input: {item_id: i-1, unused: Anvil, name: Wrench}}`
passes: both renderers spread the overlay's `params` into a form submission's input
(`crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`, `{ ...params, ...draft }`).
`read_params` (`crates/ui/ess-ui-check/src/names.rs`) counts only `params.<key>` text.

## Acceptance

- Over a model with unplaced views in two domain files, each ESS-COMPONENT-005 cites the file and
  line where its own view is declared (`domains/pass.yaml:<line of ExpectedVisits>`,
  `domains/audit.yaml:<line of Entries>`), and the message still names the component.
- `ess ui check` on the form overlay above reports no `overlay_params` finding and exits 0.
- A key that no `params.<key>` expression and no form submission reads (the existing
  `crates/ui/ess-ui-check/tests/checks.rs::overlay_params` case, a `record` overlay) is still reported
  by `overlay_params`.

## Related

story:ess-ui-check-judges-navigation-and-overlay-parameters-by-their-use covers an overlay parameter
bound into a `does` input. The form-submission case here is the same rule; whichever story is taken
first carries both acceptance lines and the other drops its `overlay_params` half.

## Scope

- cited: `crates/specify/ess-domain/src/component.rs` (`validate_command_line_views`, the `MissingDeclaration` location)
- inferred: `crates/specify/ess-compiler/src/resolve.rs` (how a `component`-family location is mapped to a span; a view location may need a locator entry)
- cited: `crates/ui/ess-ui-check/src/names.rs` (`overlay_params`, `read_params`)
- inferred: `crates/ui/ess-ui-check/tests/checks.rs` (`overlay_params` cases)
- inferred: `crates/specify/ess-domain/tests/` or `crates/specify/ess-compiler/tests/` (the two-file location case)
- inferred: `CHANGELOG.md`

confidence: medium
