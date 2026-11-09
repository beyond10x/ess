---
format: aep.planning-md/3
id: story:an-author-key-named-action-is-not-an-action
kind: story
status: draft
title: An author key named action is not read as an action
summary: A record field or export param keyed action with a map value is read as an Action and misnames the refusal (UI-adopter wave 1 U4 pass-2 note, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:w086-u4-adversary-pass-2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

The loader reads a map key `action` that is author data as an Action whenever its value is a map.
`crates/ui/ess-ui/src/locate.rs`, `expected`, returns `Expected::Action` for any map that is not a
node and sits under a key the schema lists as a single action position. It does not check that the
parent is a construct holding actions. This predates UI-adopter wave 1 (base 83391ebaeb); that
wave's U4 adversary pass 2 found it (review-result:w086-u4-adversary-pass-2, note, pre-existing).
It still reproduces on 0.92.0:

- A state type's record with a field named `action` whose type is refused is reported with the
  Action's `unknown field` message at `…/type/record/action`.
- A valid record field named `action` makes the loader report that false Action refusal instead
  of the document's real refusal elsewhere.
- A nested export param keyed `action` is refused at the right path, but with the Action's
  message.

Minimal reproduction. In `tmpl.yaml`, `KEY` is replaced per variant:

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: a
  sections: [{name: all, pages: [a]}]
pages:
  a:
    kind: list_page
    title: A
    state:
      pick:
        class: page_state
        type:
          record:
            KEY: {list: 42}
    sections:
      - {name: rec, component: record, reads: {view: probe.Items}, fields: [title]}
```

```text
# KEY = other: {list: 42}
error document_loads pages/a: data did not match any variant of untagged enum TypeExpr
# KEY = action: {list: 42}
error document_loads pages/a/state/pick/type/record/action: unknown field `list`, expected one of `name`, `does`, `bind`, `opens`, …
# KEY = other: {list: string}, plus `bogus_key: 1` on section `rec`
error document_loads pages/a/sections/rec: `record`: unknown field `bogus_key`, expected one of `reads`, `fields`, `tabs`, `item`, `actions`
# KEY = action: {list: string}, plus `bogus_key: 1` on section `rec`
error document_loads pages/a/state/pick/type/record/action: unknown field `list`, expected one of `name`, `does`, `bind`, `opens`, …
```

## Acceptance

- `ess ui check` over the document with `action: {list: 42}` reports the same refusal as with
  `other: {list: 42}`, at `…/type/record/action`. The report never names an Action field.
- `ess ui check` over the document with `action: {list: string}` and `bogus_key: 1` on section
  `rec` reports `unknown field `bogus_key`` at `pages/a/sections/rec`, the same as the `other`
  variant, and no finding under `pages/a/state`.
- A bulk `export` whose `params` holds a nested map keyed `action` is refused with the params'
  own message at its path.

## Scope

- cited: `crates/ui/ess-ui/src/locate.rs`
- inferred: `crates/ui/ess-ui/tests/` (a load-error-path test beside the U4 cases)

confidence: high

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
