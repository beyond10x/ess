---
title: "ess-ui/1 reference"
sidebar_label: "ess-ui/1"
description: "Every construct of an ess-ui/1 document, generated from the schema."
---

# ess-ui/1 reference

Every construct of an `ess-ui/1` document: what it is for, its properties, the short forms it accepts, an example and the checks that apply to it. This page is generated from `schemas/ui/ess-ui.schema.yaml` by `ess ui docs`; change the schema, not the page.

## Foundations

How types, expressions, short forms and nesting work in every construct below.

### Type rule

Types are YAML structure, never strings to be parsed.

A field's `type` is one of three things. (1) A lowercase primitive name from `primitives`. (2) A constructor map from `constructors`. (3) A capitalized bare name: a construct of this schema (Page, Reads, State …) or, inside an app document, an entry of the document's `types` map or a type of the ESS model (PartnerId, Money). Names that must resolve at validation time (a page, an overlay, a view, a command) use `{ref: kind}`. `required`, `default` and `note` are siblings of `type`, never part of it.

**Primitives**

| Primitive | Meaning |
|---|---|
| `string` | text |
| `integer` | whole number |
| `number` | decimal number |
| `boolean` | true or false |
| `duration` | a length of time; pattern: `"^[0-9]+(ms\|s\|m\|h)$"`; example: `30s` |
| `timestamp` | an instant |
| `date` | a calendar date |
| `time` | a time of day |
| `json` | any JSON value |
| `expr` | a binding expression (see `expressions`); a value, never a type constructor |
| `name` | a node name; matches NodePath.syntax.segment\_pattern |

**Constructors**

| Constructor | Form | Example | Notes |
|---|---|---|---|
| `list` | `{list: T}` | `{list: PartnerId, unique: true}` | options: `{unique: {type: boolean, default: false, effect: no_duplicate_elements, note: a set}}` |
| `map` | `{map: {key: K, value: V}}` | `{map: {key: StepId, value: {record: {x: number, "y": number}}}}` |   |
| `optional` | `{optional: T}` | `{optional: UserId}` |   |
| `enum` | `{enum: [a, b]}` | `{enum: [light, dark]}` |   |
| `one_of` | `{one_of: [T1, T2]}` | `{one_of: [duration, expr]}` |   |
| `record` | `{record: {field: T}}` | `{record: {amount: number, currency: {enum: [EUR, USD, GBP]}}}` |   |
| `ref` | `{ref: kind}` | `{ref: view}` | view, command and event resolve in the ESS model; the others in the document; kinds: `[shell, page, section, overlay, channel, state, page_kind, composite_kind, widget, view, command, event]` |
| `const` | `{const: value}` | `{const: ess-ui/1}` |   |

### Expressions

Binding expressions — the values of fields typed `expr`.

Expressions connect a node to state and data. They are short paths plus a few operators; a renderer evaluates them, a validator resolves every path.

| Form | Meaning |
|---|---|
| `state.<name>` | page, section or overlay state |
| `params.<name>` | path params of the page, or values passed to an overlay |
| `args.<name>` | a widget parameter, inside the widget's body |
| `row.<field>` | the current row of a collection, record or board |
| `selection.<field>` | the selected row(s) of the section |
| `draft.<field>` | the unsaved input of the enclosing form |
| `section.<name>.selection` | another section's selection |
| `channel.<name>.<field>` | a named field of a channel |
| `shell.<name>` | shell state |
| `url.query.<name>` | a query parameter (guards only) |
| `actor.<field>` | the signed-in actor: user\_id, account\_id, roles, signed\_in, may(\<command\>) |
| `widget.<field>` | the widget row inside a board |
| `same_as(<section>)` | the params of another section's read |
| `matches(params)` | true when a live row satisfies the section's read params |
| `operators` | ==, !=, in \[..\], not, and, or, literals |

### Short forms

Abbreviated forms and special values, each defined by what it accepts and what it expands to.

Validation: a short form is checked against its `accepts` before it expands; one that `accepts` does not admit is refused with its path

**Placeholders in an expansion**

| Placeholder | Meaning |
|---|---|
| `$value` | the value written (`$value.<field>` is one of its fields) |
| `$key` | the map key the value sits under |
| `$page` | the enclosing page |
| `$action` | the enclosing action |
| `$item` | one value of an enum type |

**Operators in an expansion**

| Operator | Meaning |
|---|---|
| `expr` | the expanded value is this expression string |
| `first_present` | the first listed source that has a value |
| `each_value_of_enum_type` | one entry per value of the named enum type, shaped by `as` |
| `remove_inherited` | the inherited entry of that name is removed during page kind merge |
| `merge_under` | the named overlay is copied and the local props are merged over it |

A placeholder that resolves to nothing: an entry of `expands_to` whose placeholder resolves to nothing is dropped

**Inheritance**

How a page, or a page kind, is merged over the kind it names.

```yaml
layout: {merge: replace}
maps:
  merge: deep
  key_order: [inherited, then_new]
  conflict_winner: page
  null_value: remove_inherited
named_lists:
  match_by: name
  order: [inherited_in_their_order, then_new_in_page_order]
  remove: "{name: <n>, remove: true}"
  different_component: page_entry_replaces
other_lists: {merge: replace}
```

### Layers

What may nest inside what.

| Layer | Contains | Note |
|---|---|---|
| `shell` | `page` | application frame: regions, preload, guards, navigation |
| `page` | `section`, `overlay` | one route; owns page state and layout |
| `section` | `composite`, `widget`, `primitive` | a region with its own read and lifecycle |
| `composite` | `composite`, `widget`, `primitive` | one member of the composite union |
| `widget` | `composite`, `widget`, `primitive` | an app-defined composite; may not contain itself, directly or indirectly |
| `primitive` | nothing | one of 9 renderer-neutral leaves |

## The document

Root, naming, node addressing and shared types.

### Document

The root of a ess-ui/1 file describing one application.

One document describes one frontend. It names the ESS model every view, command and event resolves against, the placement profile that decides where state lives, the shells, navigation, reusable widgets and pages. Names are dot-separated and follow navigation, not files: page `partners.list` of app `portal` is `portal.partners.list`, and its route is derived from the name. Old routes are kept as page `aliases`.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `format` | exactly `ess-ui/1` | yes |   | format marker |
| `app` | `name` | yes |   | root of every fully qualified name |
| `title` | `string` |   |   | human title of the application |
| `model` | `string` | yes |   | the ESS system that views, commands and events resolve against |
| `actor` | one of: `from_session` \| `anonymous` |   | `from_session` | whose grants decide what is visible |
| `placement_profile` | one of: `thin` \| `fat` \| `hybrid` | yes |   | default state placement for the whole document |
| `placement_defaults` | map of [StateClass](#stateclass) → [Store](#store) |   |   | overrides entries of the profile's default table |
| `shells` | map of `name` → [Shell](#shell) | yes |   | application frames |
| `navigation` | [Navigation](#navigation) | yes |   | menu structure and home page |
| `page_kinds` | map of `name` → [PageKind](#pagekind) |   |   | app-defined page templates |
| `widgets` | map of `name` → [Widget](#widget) |   |   | app-defined composites usable wherever a composite kind is |
| `types` | map of `name` → [Type](#type) |   |   | named value types |
| `pages` | map of `name` → [Page](#page) | yes |   | every route of the app |
| `channels` | map of `name` → [Channel](#channel) |   |   | live data sources |
| `fixtures` | [FixtureIndex](#fixtureindex) |   |   | sample data so renderers run without a backend |
| `unmapped` | list of `string` |   |   | document-level gaps found by a retrofit |

**Example**

```yaml
format: ess-ui/1
app: portal
title: Partner portal
model: partners.platform
actor: from_session
placement_profile: hybrid
placement_defaults: {draft: session_storage, preference: server}
fixtures: {dir: fixtures, index: fixtures/index.yaml}
navigation: {home: overview, sections: [{name: home, label: Home, pages: [overview]}]}
shells: {app: {regions: {main: {kind: page_outlet}}}}
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections: [{name: board, component: board, reads: {view: dashboards.Mine}}]
```

### Type

A named value type declared once in the document's `types` map.

Use `types` for value shapes several states, params or filters share — money, a time window, an enum of stages. ESS entity ids (PartnerId, UserId) are not redeclared; they resolve in the model.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (value) | one of: `string` \| map of `name` → `json` |   |   | a primitive name, a constructor map, or a capitalized named type |

**Example**

```yaml
Money: {record: {amount: number, currency: {enum: [EUR, USD, GBP]}}}
DealStage: {enum: [lead, qualified, proposal, won, lost]}
TimeWindow:
  record:
    from: timestamp
    to: timestamp
    preset: {enum: [this_month, last_30d, this_quarter, custom]}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `types_structural` | `**.type` | `{must_match: type_rule, forbidden: string_holding_a_type_expression}` | error |

### NodePath

The stable address of any node — shell region, page, section, overlay, composite, widget instance or primitive.

Every node has a `name` unique among its siblings: map entries are named by their key, list entries carry `name` (or a derived default). A node's path is the chain of container keys and names from the document root, joined with `/`. Page names keep their dots, since `/` is the separator. Paths never contain list indices. Rows of a collection are addressed by their row key under `rows/`. Editors address a node by path to insert, replace or remove it; validators report every finding by path.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (value) | `string` |   |   | a path; see syntax |

**Syntax**

```yaml
separator: /
segment_pattern: "^[A-Za-z0-9_.-]+$"
forbidden: [list_indices, empty_segment]
grammar:
  - {segment: container_key, then: child_name, note: e.g. pages/partners.list}
  - segment: field_key
    then: none
    note: "a single-valued child such as header or reads: pages/partners.list/header"
containers:
  - container:
      - shells
      - page_kinds
      - widgets
      - types
      - pages
      - channels
      - regions
      - overlays
      - state
      - params
      - board_widgets
    child_name: map_key
    note: maps; order carries no meaning
  - {container: [columns, inputs, fields], child_name: {first_present: [name, field]}}
  - container:
      - row_actions
      - bulk_actions
      - actions
      - item_actions
      - node_actions
      - edge_actions
      - alternatives
    child_name: {first_present: [name, derived_action_name]}
  - container:
      - sections
      - item
      - choices
      - parts
      - metrics
      - toolbar
      - guards
      - tabs
      - groups
      - children
      - body
      - navigation_sections
    child_name: name
    note: lists of named nodes; order is meaningful
  - {container: preload_views, child_name: view}
  - container: rows
    child_name: {row_key: {first_present: [live.match, id]}}
    note: runtime rows of a collection
widget_instances: {expanded_body_path: <instance path>/body/<node name>}
uniqueness: {scope: siblings, severity: error}
stability: "inserting, removing or reordering a sibling changes no other node's path"
```

**Examples**

`shells/app/regions/nav`, `pages/partners.list/sections/list/columns/tier`, `pages/partners.list/sections/list/row_actions/edit`, `pages/partners.list/sections/list/item/card/body/tier`, `pages/partners.list/sections/list/rows/pt-003/item/card`, `pages/tickets.detail/sections/reply/children/send`, `pages/partners.detail/layout/columns/side`, `widgets/partner_card/body/title`

**Example**

```yaml
pages/partners.list/sections/list/item/card/body/tier
```

## Shell and navigation

The application frame and how pages are reached.

### Shell

An application frame with regions, preloads, guards and shell-wide state.

A shell is what stays on screen while pages change: navigation, page outlet, overlay slot, notifications. Use one shell per distinct frame (the signed-in app, the sign-in screens). Guards run before any page of the shell opens; preloads warm views many pages need, without blocking navigation unless the policy says so.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `regions` | map of `name` → [Region](#region) | yes |   | named areas of the frame |
| `preload` | [Preload](#preload) |   |   | views read when the shell starts |
| `guards` | list of [Guard](#guard) |   |   | checks run before each page of the shell opens |
| `state` | map of `name` → [State](#state) |   |   | shell-wide state: nav expanded, theme, credentials |
| `overlays` | map of `name` → [overlay](#overlay) |   |   | overlays reachable from every page |
| `source` | list of `string` |   |   | traceability to retrofitted files |

**Example**

```yaml
regions:
  nav: {kind: navigation, props: {collapsible: true, search: {over: [label, synonyms]}}}
  main: {kind: page_outlet}
  overlay: {kind: overlay_outlet, props: {one_at_a_time: true}}
  notify: {kind: notifications}
state:
  nav_expanded: {type: boolean, class: preference, default: true}
  credentials:
    type: {record: {access_token: string, refresh_token: string}}
    class: credential
    sensitive: true
    store: server_session
    pinned: true
preload:
  policy: in_background
  except_on: [auth.sign_in]
  views: [{view: session.Me}, {view: tags.All}]
guards: [{name: signed_in, when: not actor.signed_in, then: {redirect: auth.sign_in}}]
```

### Region

One named area of a shell.

Regions say what a shell area is for, not how it looks. A renderer maps `navigation` to a sidebar, a TUI pane or a command palette; `overlay_outlet` is where drawers and dialogs appear; `assistant` hosts a chat helper.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `kind` | one of: `navigation` \| `page_outlet` \| `overlay_outlet` \| `notifications` \| `assistant` \| `account_menu` | yes |   | role of the region |
| `props` | map of `name` → `json` |   |   | kind-specific options: collapsible, search, one\_at\_a\_time, actions |
| `visible` | `expr` |   | `"true"` | shows the region only when true |

**Example**

```yaml
kind: account_menu
props:
  actions:
    - {name: switch_org, opens: switch_org, label: Switch organization}
    - {name: sign_out, does: session.SignOut, label: Sign out}
```

### Preload

Views a shell reads before or alongside the first page.

Preloading replaces route guards that block navigation until many stores are filled. `in_background` starts the reads and lets the page render at once; `before_first_page` blocks and should be rare.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `views` | list of record \{ `view`: name of an ESS `view`, `params`: optional map of `name` → `expr`, `endpoint`: optional `string` \} |   |   | views to read; each is named by its view |
| `policy` | one of: `before_first_page` \| `in_background` \| `on_demand` |   | `in_background` | whether the first page waits |
| `except_on` | list of name of a [Page](#page) |   |   | pages that skip the preload |

**Example**

```yaml
policy: in_background
except_on: [auth.sign_in]
views: [{view: session.Me}, {view: tags.All}]
```

### Guard

A condition checked before a page opens, with a redirect, refusal or state change.

Guards express sign-in requirements and link-borne state. Evaluated in order; the first whose `when` holds decides. Business permissions belong in actor grants.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` | yes |   | node name among the shell's guards |
| `when` | `expr` | yes |   | condition over actor, url and page |
| `then` | one of: record \{ `redirect`: name of a [Page](#page), `params`: optional map of `name` → `expr` \} \| record \{ `refuse`: `string` \} \| record \{ `set`: map of `expr` → `expr` \} | yes |   | redirect, refuse with a message, or set shell state |

**Example**

```yaml
name: invite_link
when: url.query.invite
then: {set: {shell.invite_token: url.query.invite}}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `page_refs` | `**.navigate.to`, `**.to.to`, `pages.*.switch_to[]`, `shells.*.guards[].then.redirect` | `{must_resolve: {ref: page}}` | error |

### Navigation

The menu — sections of pages, the home page, hidden routes and search.

Navigation lists every page exactly once: in a section, or in `hidden` when it is reached by link or redirect only. With `visibility: by_actor_grants` a page appears when the actor may read the views its eager sections read, so role lists disappear from UI code.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `home` | name of a [Page](#page) | yes |   | page opened at the app root |
| `sections` | list of [NavSection](#navsection) | yes |   | menu groups in display order |
| `hidden` | list of name of a [Page](#page) |   |   | routed pages not shown in the menu |
| `visibility` | one of: `by_actor_grants` \| `by_roles` |   | `by_actor_grants` | how the menu decides what to show |
| `search` | record \{ `over`: list of (one of: `label` \| `synonyms`) \} |   |   | menu search fields |
| `restrictions` | list of record \{ `when`: `expr`, `sections`: list of `name`, `unmapped`: optional list of `string` \} |   |   | when a condition holds only these sections show |

**Example**

```yaml
home: overview
visibility: by_actor_grants
search: {over: [label, synonyms]}
sections:
  - name: sales
    label: Sales
    icon: briefcase
    pages: [partners.list, deals.list, deals.pipeline]
hidden: [partners.detail, auth.sign_in]
restrictions: [{when: "actor.roles == {partner}", sections: [home, billing, support]}]
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `nav_resolves` | `navigation.sections[].pages[]`, `navigation.hidden[]`, `navigation.home` | `{must_resolve: {ref: page}}` | error |

### NavSection

One group of the menu.

A section is a heading with pages under it — a fixed list, or entries generated from a view.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` | yes |   | node name |
| `label` | `string` |   |   | heading text |
| `icon` | `string` |   |   | semantic icon name mapped by the renderer |
| `pages` | one of: list of name of a [Page](#page) \| [DynamicNavEntries](#dynamicnaventries) | yes |   | fixed pages or entries from a view |

**Example**

```yaml
name: billing
label: Billing
icon: receipt
pages: [invoices.list]
```

### DynamicNavEntries

Menu entries generated from the rows of a view.

Use when the menu depends on data, such as one entry per saved view.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `from_view` | name of an ESS `view` | yes |   | one entry per row |
| `page` | name of a [Page](#page) | yes |   | page each entry opens |
| `param` | `name` | yes |   | page param filled from the row id |
| `label` | `expr` |   | `row.label` | entry text |
| `synonyms` | list of `string` |   |   | extra search words |
| `filter` | `expr` |   |   | rows to skip |

**Example**

```yaml
from_view: saved_views.Mine
page: deals.saved
param: view_id
label: row.label
synonyms: [filter, view]
```

### NavEntry

How a page appears in the menu, declared on the page as `nav`.

The label and search synonyms of a page, plus the role list a retrofitted app used.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `label` | `string` |   | `$page.title` | menu text |
| `synonyms` | list of `string` |   |   | extra words the menu search matches |
| `roles_today` | list of `string` |   |   | traceability only; visibility comes from grants |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `label` | nothing: leave it out | `$page.title` |

**Example**

```yaml
label: Partners
synonyms: [resellers, accounts]
```

## Pages and sections

Routes, layouts, page templates, and the section as the unit of loading.

### Page

One route — its state, layout, header, sections and overlays.

A page owns the state a link should reproduce (filters, paging, selection) and composes sections, each loading on its own. Start from a page kind and declare only what differs. `layout` arranges sections renderer-neutrally. Use `switch_to` for sibling pages shown as a view switch.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `kind` | name of a [PageKind](#pagekind) | yes |   | template the page starts from |
| `shell` | name of a [Shell](#shell) |   | `{first_present: [shells.app, only_shell]}` | frame the page renders in |
| `title` | `string` |   |   | header title |
| `nav` | [NavEntry](#naventry) |   |   | menu label and synonyms |
| `params` | map of `name` → [Type](#type) |   |   | path parameters (always in the URL) |
| `aliases` | list of `string` |   |   | legacy route paths |
| `switch_to` | list of name of a [Page](#page) |   |   | sibling pages offered in the header |
| `visible` | `expr` |   |   | extra condition beyond grants, such as a feature flag |
| `layout` | [PageLayout](#pagelayout) |   | `stack` | how sections are arranged |
| `state` | map of `name` → optional [State](#state) |   |   | page state |
| `header` | [header](#header) |   |   | title, total, actions, live status |
| `sections` | list of [Section](#section) | yes |   | regions of the page, named, in declaration order; merged with the kind's by name |
| `overlays` | map of `name` → optional [overlay](#overlay) |   |   | drawers and dialogs of the page (unordered) |
| `profile` | one of: `thin` \| `fat` |   |   | placement for this page in a hybrid document |
| `source` | list of `string` |   |   | files the page was retrofitted from |
| `unmapped` | list of `string` |   |   | gaps the retrofit could not resolve |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `state.* \| overlays.* \| header.*` | exactly `null` | `{remove_inherited: $key}` |
| `sections[]` | record \{ `name`: `name`, `remove`: exactly `true` \} | `{remove_inherited: $value.name}` |
| `shell` | nothing: leave it out | `{first_present: [shells.app, only_shell]}` |

**Inheritance**

```yaml
from: kind
merge: shorthands.inheritance
conflict_winner: page
applies_to: [state, sections, overlays, header, layout]
```

**Example**

```yaml
kind: detail_page
title: Partner
params: {id: PartnerId}
layout:
  columns:
    - {name: side, sections: [summary], width: narrow}
    - {name: main, sections: [deals], width: wide}
header: {actions: [{name: edit, opens: edit, label: Edit}]}
sections:
  - name: summary
    component: record
    reads: {view: partners.ById, params: {id: params.id}}
    fields: [name, {field: tier, as: badge}]
  - name: deals
    component: collection
    reads: {view: deals.ByPartner, params: {partner_id: params.id}}
    columns: [title, {field: stage, as: badge}]
overlays: {edit: {kind: drawer, component: form, same_as: partners.list.edit}}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `page_reachable` | `pages.*` | `{must_be_in: ["navigation.sections[].pages[]", "navigation.hidden[]", "navigation.sections[].pages.page"]}` | error |
| `page_refs` | `**.navigate.to`, `**.to.to`, `pages.*.switch_to[]`, `shells.*.guards[].then.redirect` | `{must_resolve: {ref: page}}` | error |

### PageLayout

A renderer-neutral arrangement of a page's sections.

`stack` shows sections one below the other in declaration order. `columns` splits the page into named columns, each listing its sections. `areas` names grid cells row by row and places sections into areas; an area spanning several cells repeats its name. Renderers without columns or areas fall back to a stack in section declaration order.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `stack` | exactly `stack` |   |   | sections top to bottom in declaration order |
| `columns` | list of record \{ `name`: `name`, `sections`: list of name of a [Section](#section), `width`: optional (one of: `narrow` \| `normal` \| `wide`) \} |   |   | named columns left to right |
| `areas` | record \{ `grid`: list of list of `name`, `place`: map of `name` → list of name of a [Section](#section) \} |   |   | grid of area names and the sections placed in each |

**You may also write**

| Where | You write | It means |
|---|---|---|
| the value itself | nothing: leave it out | `stack` |

**Form**

```yaml
one_of:
  - {const: stack}
  - record:
      columns:
        list:
          record:
            name: name
            sections: {list: {ref: section}}
            width: {optional: {enum: [narrow, normal, wide]}}
  - record:
      areas:
        record:
          grid: {list: {list: name}}
          place: {map: {key: name, value: {list: {ref: section}}}}
```

**Default**

```yaml
stack
```

**Degrades**

```yaml
no_columns: stack
no_areas: stack
```

**Example**

```yaml
areas:
  grid: [[kpis, kpis], [pipeline, feed], [board, board]]
  place: {kpis: [kpis], pipeline: [pipeline], feed: [feed], board: [board]}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `section_refs` | `**.header.total`, `**.header.filters`, `**.depends_on`, `pages.*.layout.columns[].sections[]`, `pages.*.layout.areas.place.*[]` | `{must_resolve_in: [page.sections]}` | error |
| `layout_complete` | `pages.*.layout` | `{must: place_every_section_once}` | warning |

### PageKind

A page template that fills the same sections, state, header and overlays.

Most admin pages share one grammar: header with title and total, filters, a collection, a create/edit drawer and a delete confirm. A page kind captures it once. Built-in kinds are listed in `builtins`; a document adds its own under `page_kinds` with `extends`.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `extends` | name of a [PageKind](#pagekind) |   |   | kind to start from |
| `purpose` | `string` |   |   | one line on when to use it |
| `state` | map of `name` → [State](#state) |   |   | state every page of the kind has |
| `sections` | list of [Section](#section) |   |   | sections every page of the kind has, named, in order |
| `overlays` | map of `name` → [overlay](#overlay) |   |   | overlays every page of the kind has |
| `header` | [header](#header) |   |   | default header |
| `layout` | [PageLayout](#pagelayout) |   |   | default layout |

**Builtins**

```yaml
list_page:
  purpose: "rows of one entity with search, filters, paging and row actions"
  state:
    search: {type: string, class: page_state, store: url, pinned: true}
    page: {type: integer, class: page_state, store: url, pinned: true, default: 1}
    size: {type: integer, class: page_state, store: url, pinned: true, default: 25}
    sort:
      type: {optional: {record: {by: string, dir: {enum: [asc, desc]}}}}
      class: page_state
      store: url
      pinned: true
  sections:
    - name: filters
      component: filter_bar
      binds: [state.search]
      search: {binds: state.search}
    - {name: list, component: collection}
  header: {title: from_page, total: list, filters: filters}
report_page:
  purpose: read-only rows over a time window with export and optional live inserts
  state:
    window:
      type:
        record:
          from: timestamp
          to: timestamp
          preset: {enum: [this_month, last_30d, this_quarter, custom]}
      class: page_state
      store: url
      pinned: true
      default: {preset: last_30d}
    page: {type: integer, class: page_state, store: url, pinned: true, default: 1}
    size: {type: integer, class: page_state, store: url, pinned: true, default: 50}
  sections:
    - name: filters
      component: filter_bar
      binds: [state.window]
      window: {binds: state.window}
    - {name: list, component: collection}
  header: {title: from_page, total: list, filters: filters}
detail_page:
  purpose: one record and its related sections
  sections: [{name: summary, component: record}]
settings_page:
  purpose: one settings record edited in place
  sections: [{name: settings, component: form}]
dashboard_page:
  purpose: a board of widgets each with its own read or channel
  sections: [{name: board, component: board}]
editor_page:
  purpose: a graph or document edited as a whole
  sections: [{name: editor, component: graph_editor}]
form_page:
  purpose: a standalone form such as sign-in
  sections: [{name: form, component: form}]
static_page: {purpose: fixed content without reads, sections: [{name: body, component: record}]}
```

**Example**

```yaml
extends: list_page
purpose: rows of one entity with a drawer to create and edit and a confirm to delete
state: {tags: {type: {list: Tag}, class: page_state, store: url, pinned: true}}
overlays:
  create: {kind: drawer, component: form}
  edit: {kind: drawer, component: form}
  delete: {kind: dialog, component: confirm, confirm_label: Delete}
header:
  title: from_page
  total: list
  filters: filters
  actions: [{name: create, opens: create, label: New}]
```

### Section

A region of a page with one read and its own loading lifecycle.

The section, not the page, is the unit of loading. Each has at most one `reads`, its own states and optional live updates; a slow section never blocks its siblings. Use `load: on_visible` for partial loading and `depends_on` when a section needs another section's selection. `component` names a member of the composite union or a widget; its props are written inline beside the section's own fields, and a key that is neither is refused. `children` adds widgets or primitives rendered with the section (a caption, a button).

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` | yes |   | node name among the page's sections |
| `component` | one of: name of a [Composite](#composite) \| name of a [Widget](#widget) | yes |   | what the section renders |
| `reads` | [Reads](#reads) |   |   | the section's data; one per section; it is the composite's own `reads` |
| `live` | [Live](#live) |   |   | how channel events change the rows |
| `load` | one of: `eager` \| `on_visible` \| `on_demand` |   | `eager` | when the read starts |
| `depends_on` | name of a [Section](#section) |   |   | sibling whose selection the params use; loads after it |
| `visible` | `expr` |   |   | shows the section only when true |
| `states` | [SectionStates](#sectionstates) |   |   | how each lifecycle state renders |
| `state` | map of `name` → [State](#state) |   |   | state local to the section |
| `children` | list of [Node](#node) |   |   | extra named nodes rendered after the composite |
| `profile` | one of: `thin` \| `fat` |   |   | placement for this section in a hybrid document |
| `degrades` | [Degrades](#degrades) |   |   | fallbacks for renderers lacking a capability |
| `unmapped` | list of `string` |   |   | gaps found by a retrofit |

**Lifecycle**

```yaml
states: [idle, loading, ready, empty, refreshing, stale, failed, forbidden]
transitions:
  - {from: idle, to: loading, "on": mounted and dependencies satisfied}
  - {from: loading, to: ready, "on": the read returned rows}
  - {from: loading, to: empty, "on": the read returned zero rows}
  - {from: loading, to: failed, "on": refusal or transport error}
  - {from: loading, to: forbidden, "on": the actor lacks the grant for the view}
  - from: [ready, empty]
    to: refreshing
    "on": "params changed, a command outcome touched the view, or a poll tick"
  - {from: refreshing, to: [ready, empty, failed], "on": the read finished}
  - {from: ready, to: stale, "on": the feeding channel is down past stale_after}
  - {from: stale, to: refreshing, "on": the channel reconnected}
  - {from: failed, to: loading, "on": retry}
invariants:
  - {id: first_paint, rule: shell and page skeleton render before any read}
  - {id: stale_marked, rule: "a stale value is marked stale, never shown as current"}
```

**Example**

```yaml
name: list
component: collection
reads:
  view: activity.Page
  params: {from: state.window.from, to: state.window.to, types: state.types}
  paging: cursor
live:
  channel: activity
  effect: insert_top
  only_if: matches(params)
  when_paged_away: count_new
states:
  loading: skeleton
  empty: {message: Nothing happened in this window}
  stale: {mark: both}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `section_refs` | `**.header.total`, `**.header.filters`, `**.depends_on`, `pages.*.layout.columns[].sections[]`, `pages.*.layout.areas.place.*[]` | `{must_resolve_in: [page.sections]}` | error |
| `widget_expands` | `**.component naming a widget` | `{must: [resolve_widget, required_args_present, args_match_param_types, no_recursion], then: check_expanded_body}` | error |

### SectionStates

How a section renders each lifecycle state.

Declare what the user sees while loading, when there is nothing, when the read failed, when access is denied, and when live data went stale.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `loading` | one of: `skeleton` \| `spinner` \| `none` |   | `skeleton` | first load |
| `refreshing` | one of: `keep_rows` \| `skeleton` |   | `keep_rows` | later loads |
| `empty` | record \{ `message`: `string`, `action`: optional [Action](#action) \} |   |   | ready with no rows |
| `failed` | record \{ `message`: optional `string`, `retry`: `boolean` \} |   | `{retry: true}` | read refused or transport error |
| `forbidden` | record \{ `message`: optional `string` \} |   |   | actor lacks the grant |
| `stale` | record \{ `mark`: (one of: `badge` \| `dim` \| `both`) \} |   | `{mark: badge}` | live channel down |

**Example**

```yaml
loading: skeleton
empty: {message: No partners yet, action: {name: create, opens: create, label: New partner}}
```

## Composites, widgets and primitives

The composite union, header and overlay, app-defined widgets, the 9 primitives, and renderer fallbacks.

### collection

Rows of a view with columns, sorting, paging, selection and row actions.

Use for any list, table or card grid. Columns are fields of the rows; `as` picks a semantic rendering. Row actions run commands bound to the row or open overlays. `item` nests nodes per row (widgets, metrics, primitives); `expand` shows detail inline; `group_by` and `reorder` give a kanban. A TUI renders it as a scrollable list.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) |   |   | the rows |
| `columns` | one of: list of [Field](#field) \| record \{ `binds`: `expr`, `all`: list of [Field](#field) \} \| `string` |   |   | fixed columns, user-selectable columns, or an UNMAPPED string |
| `sort` | record \{ `by`: `name`, `dir`: optional (one of: `asc` \| `desc`), `allowed`: optional list of `name`, `mode`: optional (one of: `server` \| `client`) \} |   |   | default and allowed sort |
| `style` | one of: `table` \| `cards` \| `list` \| `tree` |   | `table` | presentation hint |
| `selection` | one of: `none` \| `single` \| `multiple` \| record \{ `mode`: (one of: `single` \| `multiple`), `enabled`: `expr` \} |   | `none` | row selection, optionally only in a mode |
| `row_actions` | list of [Action](#action) |   |   | actions per row |
| `bulk_actions` | list of [Action](#action) |   |   | actions on the selection |
| `actions` | list of [Action](#action) |   |   | actions on the whole collection |
| `expand` | [Node](#node) |   |   | detail shown when a row expands |
| `item` | list of [Node](#node) |   |   | nested named nodes per row, in order |
| `reorder` | record \{ `does`: name of an ESS `command`, `endpoint`: optional `string` \} |   |   | drag to reorder, saved by a command |
| `group_by` | `name` |   |   | field rows are grouped under |

**Example**

```yaml
component: collection
style: cards
selection: multiple
reads:
  view: partners.Page
  params: {q: state.search, tier: state.tier}
  paging: server
  debounce: 300ms
columns:
  - {field: name, sortable: true}
  - {field: tier, as: badge}
  - {field: revenue_ytd, as: number, sortable: true}
item: [{name: card, component: partner_card, args: {partner: row}}]
row_actions:
  - {name: open, navigate: {to: partners.detail, params: {id: row.id}}, label: Open}
  - {name: delete, opens: delete, as: icon}
```

### record

One row shown as labelled, read-only fields.

Use for detail views, KPI panels and read-only summaries.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) |   |   | the row |
| `fields` | list of [Field](#field) |   |   | fields to show |
| `tabs` | list of [Tab](#tab) |   |   | fields split into tabs |
| `item` | list of [Node](#node) |   |   | nested named nodes such as metrics or widgets, in order |
| `actions` | list of [Action](#action) |   |   | actions on the record |

**Example**

```yaml
component: record
reads: {view: partners.ById, params: {id: params.id}}
fields: [name, {field: tier, as: badge}, website, {field: created_at, as: date}]
item: [{name: revenue, component: money, args: {value: row.revenue_ytd}}]
```

### form

Inputs bound to one ESS command's input.

A form is a command's input made editable. Field types, required-ness, validation and error messages come from the command; the form chooses order, grouping and widgets. A refusal is shown on the form as the command's typed error. Settings pages use `groups` that save on change; typed editors use `variant_by` to generate one form per type.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `does` | name of an ESS `command` | yes |   | command the form submits |
| `loads` | [Reads](#reads) |   |   | initial values for edit forms |
| `fields` | list of [Field](#field) |   | `{from: does.input_fields, order: declaration}` | inputs shown |
| `groups` | list of [FormGroup](#formgroup) |   |   | independently saved groups |
| `tabs` | list of [Tab](#tab) |   |   | fields split into tabs |
| `parts` | list of [Node](#node) |   |   | nested named nodes inside the form, in order |
| `actions` | list of [Action](#action) |   |   | extra actions next to submit |
| `result` | [Node](#node) |   |   | shows the command's outcome |
| `record` | [Node](#node) |   |   | read-only fields above the inputs |
| `submit` | record \{ `label`: optional `string`, `closes`: optional `boolean`, `auto`: optional `boolean` \} |   |   | submit button; auto submits on open |
| `draft` | [State](#state) |   |   | the unsaved input (class draft) |
| `save` | one of: `on_submit` \| `on_change` \| `on_blur` |   | `on_submit` | when the command runs |
| `variant_by` | record \{ `field`: `name`, `forms`: map of `string` → `name` \} |   |   | one generated form per value of a field |
| `endpoint` | `string` |   |   | traceability |

**Example**

```yaml
component: form
does: deals.CreateDeal
tabs:
  - {name: basics, label: Basics, fields: [title, {field: value, as: number}]}
  - {name: schedule, label: Schedule, fields: [{field: close_date, as: date}]}
```

### choice

Pick one or many values from a view or a fixed list.

One composite replaces dropdowns, tag pickers, tree selects and checkbox lists. `style` is a hint; a TUI renders all styles as a checklist popup.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) |   |   | options from a view |
| `options` | one of: list of record \{ `value`: `json`, `label`: `string` \} \| list of `string` \| `name` |   |   | fixed options or a named enum type |
| `binds` | `expr` |   |   | state the value is written to |
| `multiple` | `boolean` |   | `false` | many values |
| `style` | one of: `dropdown` \| `tags` \| `tree` \| `grouped` \| `radio` \| `segmented` \| `checklist` |   | `dropdown` | presentation hint |
| `creatable` | record \{ `does`: name of an ESS `command` \} |   |   | typed values create new options |
| `visible` | `expr` |   |   | shows the choice only when true |
| `note` | `string` |   |   | author remark |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `options[]` | `string` | `{value: $value, label: $value}` |
| `options` | `name` | `{each_value_of_enum_type: $value, as: {value: $item, label: $item}}` |

**Example**

```yaml
component: choice
multiple: true
options: DealStage
```

### filter_bar

Search, choices, inputs and a time window bound to page state.

A filter bar writes page state; sections read it in their params. Because that state lives in the URL, a filtered view is a link.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `binds` | list of `expr` | yes |   | state the bar writes |
| `search` | record \{ `binds`: `expr`, `placeholder`: optional `string` \} |   |   | free-text search |
| `window` | record \{ `binds`: `expr`, `time`: optional `boolean` \} |   |   | date or date-time range |
| `choices` | list of [Node](#node) |   |   | choice composites, named, in order |
| `inputs` | list of [Field](#field) |   |   | free inputs each bound to state with binds |
| `actions` | list of [Action](#action) |   |   | actions next to the filters |
| `reset` | `boolean` |   | `false` | offers a reset-all button |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `choices[]` | record \{ `name`: `name`, `remove`: exactly `true` \} | `{remove_inherited: $value.name}` |

**Example**

```yaml
component: filter_bar
binds: [state.search, state.stage, state.min_value]
search: {binds: state.search}
choices: [{name: stage, component: choice, multiple: true, options: DealStage}]
inputs: [{field: min_value, as: number, binds: state.min_value}]
```

### header

Page title, count, primary actions, view switch, filters and live status.

Every page has one header, placed by position (`header`) rather than by `component`, so it is not a member of the composite union. `total` names the section whose total is shown; `live` lists channels whose connection state is shown, so a stale page is visibly stale.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `title` | one of: `string` \| exactly `from_page` |   |   | header title |
| `total` | name of a [Section](#section) |   |   | section whose total is shown |
| `actions` | list of [Action](#action) |   |   | primary actions |
| `switch` | list of `name` |   |   | pages from switch\_to, or modes of this page |
| `filters` | name of a [Section](#section) |   |   | filter bar rendered in the header |
| `live` | list of name of a [Channel](#channel) |   |   | channels whose lifecycle is shown |
| `metrics` | list of [Node](#node) |   |   | headline metrics, named, in order |
| `help` | record \{ `text`: optional `string`, `link`: optional `string` \} |   |   | help text or link |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `title` | exactly `from_page` | `$page.title` |

**Example**

```yaml
live: [tickets]
actions: [{name: create, opens: create, label: New ticket}]
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `channel_refs` | `**.live.channel`, `**.header.live[]`, `channel.<name> inside expr` | `{must_resolve: {ref: channel}}` | error |
| `section_refs` | `**.header.total`, `**.header.filters`, `**.depends_on`, `pages.*.layout.columns[].sections[]`, `pages.*.layout.areas.place.*[]` | `{must_resolve_in: [page.sections]}` | error |

### overlay

A drawer, dialog, fullscreen pane or popover shown on demand.

Overlays hold forms, confirms and detail views opened by actions. They belong to a page or a shell. An overlay is a frame around one member of the composite union (or a widget): the frame fields below sit beside the member's own props, and a key that is neither is refused. `same_as` reuses another overlay's definition, addressed as `<page>.<overlay>`; local props override.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `kind` | one of: `drawer` \| `dialog` \| `fullscreen` \| `popover` | yes |   | presentation hint; a TUI uses a full-screen pane |
| `component` | one of: name of a [Composite](#composite) \| name of a [Widget](#widget) | yes |   | what the overlay renders |
| `title` | `string` |   |   | overlay title; a confirm shows it as its question |
| `params` | map of `name` → `expr` |   |   | values passed by the opener |
| `state` | map of `name` → [State](#state) |   |   | overlay-local state |
| `visible` | `expr` |   |   | shows the overlay only when true |
| `degrades` | [Degrades](#degrades) |   |   | fallbacks for renderers lacking a capability |
| `same_as` | name of an [overlay](#overlay) |   |   | reuse another overlay; local props override |
| `unmapped` | list of `string` |   |   | gaps found by a retrofit |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `same_as` | name of an [overlay](#overlay) | `{merge_under: $value}` |

**Example**

```yaml
kind: drawer
component: record
title: Invoice
reads: {view: invoices.ById, params: {id: params.id}}
fields: [number, partner, {field: status, as: badge}, {field: due, as: date}]
actions: [{name: pdf, export: {reads: invoices.Pdf, as: zip}, label: Download PDF}]
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `same_as_resolves` | `**.same_as` | `{must_resolve: {ref: overlay}}` | error |
| `widget_expands` | `**.component naming a widget` | `{must: [resolve_widget, required_args_present, args_match_param_types, no_recursion], then: check_expanded_body}` | error |

### confirm

A yes/no step before a command, listing references and consequences.

Use before destructive commands, inside an overlay whose `title` is the question. `references` shows what still uses the record; `input` asks the user to type a value; `alternatives` offers a softer action.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `body` | `string` |   |   | explanation |
| `does` | name of an ESS `command` |   |   | command run on confirm |
| `references` | name of an ESS `view` |   |   | the used-by view shown first |
| `consequences` | list of `string` |   |   | what the command will do |
| `confirm_label` | `string` |   | `Delete` | confirm button text |
| `danger` | `boolean` |   | `true` | styles the confirm as destructive |
| `input` | record \{ `label`: `string`, `must_equal`: optional `expr` \} |   |   | type-to-confirm |
| `alternatives` | list of [Action](#action) |   |   | other actions offered |
| `endpoint` | `string` |   |   | traceability |

**Example**

```yaml
kind: dialog
component: confirm
title: Close ticket
does: tickets.CloseTicket
input: {label: Type CLOSE to confirm, must_equal: CLOSE}
alternatives: [{name: snooze, does: tickets.SnoozeTicket, label: Snooze for a day}]
```

### metric

One number from a view or channel, with a window and a stale mark.

For KPI tiles and per-row live numbers. With `from` a metric reads a channel field; when the channel is stale the number is marked stale, not shown as current.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) |   |   | value from a view |
| `from` | `expr` |   |   | value from a channel field |
| `match` | `name` |   |   | row field that picks a keyed value |
| `window` | `duration` |   |   | time window the value covers |
| `format` | one of: `number` \| `duration` \| `percent` \| `bytes` |   | `number` | display format |
| `label` | `string` |   |   | caption |

**Example**

```yaml
component: metric
from: channel.metrics.overdue_invoices
label: Overdue invoices
```

### chart

A series view drawn as a chart.

A renderer without charts shows the series as a table (degrade no\_charts).

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) | yes |   | the series |
| `chart` | one of: `line` \| `bar` \| `doughnut` \| `pie` \| `polar` \| `single_number` \| `list` \| `table` \| record \{ `binds`: `expr`, `options`: list of `string` \} | yes |   | chart kind, fixed or chosen by data |
| `x` | `name` |   |   | x-axis field |
| `series` | list of `name` |   |   | y fields |

**Example**

```yaml
component: chart
chart: bar
reads: {view: deals.StageTotals, params: {window: state.window}}
x: stage
series: [count, value]
degrades: {no_charts: table}
```

### board

A user-arranged grid of widgets, persisted.

A board reads a dashboard record; each row places a widget whose kind is chosen by `widget_by`. A renderer without free layout stacks them in layout order.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) | yes |   | the dashboard record |
| `widget_by` | `name` |   |   | row field choosing the widget kind |
| `widgets` | map of `name` → [Node](#node) |   |   | node per widget kind |
| `item_actions` | list of [Action](#action) |   |   | actions on each placed widget |
| `layout` | record \{ `persisted_by`: name of an ESS `command`, `editable_by`: optional name of an ESS `command`, `state`: optional `expr` \} |   |   | how the arrangement is saved |

**Example**

```yaml
component: board
reads: {view: dashboards.Mine}
widget_by: type
widgets:
  win_rate: {component: metric, reads: {view: deals.WinRate}, format: percent, label: Win rate}
layout: {persisted_by: dashboards.SaveLayout, state: state.layout}
degrades: {no_free_layout: stack}
```

### graph_editor

Nodes and edges of a model, editable on a canvas.

For flow and workflow editors. Nodes open an overlay to edit; edges carry actions such as inserting a step. A renderer without a canvas falls back to a collection of nodes.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) | yes |   | the graph |
| `nodes` | record \{ `kind_by`: `name`, `opens`: optional name of an [overlay](#overlay) \} |   |   | node kind field and edit overlay |
| `edges` | record \{ `from`: `name`, `to`: `name`, `kind_by`: optional `name` \} |   |   | edge endpoints and kind |
| `node_actions` | list of [Action](#action) |   |   | context menu of a node |
| `edge_actions` | list of [Action](#action) |   |   | context menu of an edge |
| `toolbar` | list of [Node](#node) |   |   | named nodes above the canvas, left to right |

**Example**

```yaml
component: graph_editor
reads: {view: workflows.Graph}
nodes: {kind_by: step_type, opens: edit_step}
edges: {from: from_step, to: to_step, kind_by: outcome}
edge_actions: [{name: insert, does: workflows.InsertStep, label: Insert step}]
toolbar: [{name: zoom, primitive: input, as: number, binds: state.zoom}]
degrades: {no_graph_editor: collection}
```

### rich_text

Text with expression completion or markup.

For message templates with \{\{expression\}\} completion, markup and JSON bodies.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `completes` | name of an ESS `view` |   |   | view of completable expressions |
| `syntax` | one of: `plain` \| `expression` \| `ssml` \| `json` \| `curl` |   | `expression` | language of the text |
| `binds` | `expr` |   |   | draft field the text is written to |

**Example**

```yaml
component: rich_text
completes: templates.Expressions
syntax: expression
binds: draft.body
```

### references

The "used by" list of a record.

Shown before destructive commands and in usage tabs so users see what depends on a record.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `reads` | [Reads](#reads) | yes |   | the referencing records |
| `columns` | list of [Field](#field) |   | `[label, type]` | columns shown |
| `navigates` | `boolean` |   | `true` | rows link to the referencing record |

**Example**

```yaml
component: references
reads: {view: partners.UsedBy, params: {id: params.id}}
```

### Widget

An app-defined composite with typed parameters, usable wherever a composite kind is.

Declare a widget under the document's `widgets` when the same arrangement of composites and primitives recurs — a partner card, a money value, a toned status. Its `params` are typed; its `body` is a list of named nodes that may read `args.<param>`. A widget is expanded at its use site and then checked like a built-in. A widget may not contain itself.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `summary` | `string` | yes |   | one line shown in pickers and docs |
| `doc` | `string` |   |   | longer description for authors |
| `params` | map of `name` → record \{ `type`: [Type](#type), `required`: optional `boolean`, `default`: optional `json`, `note`: `string` \} |   |   | typed parameters |
| `arrange` | one of: `row` \| `column` \| `grid` |   | `column` | how body nodes are arranged |
| `body` | list of [Node](#node) | yes |   | named nodes; each may use args |

**Example**

```yaml
summary: "A partner as a card with logo, tier and revenue."
params: {partner: {type: Partner, required: true, note: the partner row}}
arrange: column
body:
  - {name: logo, primitive: image, src: args.partner.logo_url, alt: Partner logo, fit: contain}
  - {name: title, primitive: text, text: args.partner.name, style: heading}
  - name: tier
    component: status_badge
    args:
      status: args.partner.tier
      tones: {registered: neutral, silver: info, gold: warning, platinum: success}
  - {name: revenue, component: money, args: {value: args.partner.revenue_ytd}}
  - name: open
    primitive: link
    text: Open
    to: {to: partners.detail, params: {id: args.partner.id}}
```

### WidgetInstance

A use of a widget, with arguments.

Written like a composite with `component` naming the widget and `args` supplying its params. Required params must be given; each arg must match the param type.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `component` | name of a [Widget](#widget) | yes |   | the widget used |
| `args` | map of `name` → `json` |   |   | one entry per param: an expression, or a literal such as a map |
| `name` | `name` |   |   | node name, required inside lists |
| `visible` | `expr` |   |   | shows the instance only when true |
| `body` | list of [Node](#node) |   |   | written by expansion, never by an author: the widget body with args substituted |

**Expansion**

```yaml
- {step: substitute, detail: args.<param> in the body is replaced by the bound expression}
- step: validate
  detail: the expanded nodes are checked like built-ins; findings are reported at <instance path>/body/<node name>
- step: bound
  detail: the expanded bodies of all uses in a document hold at most 100000 YAML values; the outermost use that passes the limit is refused (widget_expands)
```

**Example**

```yaml
component: status_badge
args:
  status: row.stage
  tones: {lead: neutral, qualified: info, proposal: warning, won: success, lost: danger}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `widget_expands` | `**.component naming a widget` | `{must: [resolve_widget, required_args_present, args_match_param_types, no_recursion], then: check_expanded_body}` | error |

### Node

Anything that can appear inside a section, composite or widget — a composite, a widget instance or a primitive.

Wherever a nested element is allowed (`item`, `children`, `body`, `parts`, `expand`, `choices`, `metrics`, `toolbar`), a Node is expected. In lists every node carries `name`; in the one map of nodes (board `widgets`) the key is its name. A `component` naming a member of the composite union is a composite; any other `component` names a widget.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (value) | one of: [Composite](#composite) \| [WidgetInstance](#widgetinstance) \| [Primitive](#primitive) |   |   | exactly one of component or primitive |

**Exactly one of**

`component`, `primitive`

**Example**

```yaml
name: send
primitive: button
label: Send
tone: primary
action: {name: send, does: tickets.PostReply}
```

### Field

One input of a form or one column of a collection.

Columns and form inputs are fields. `as` is a semantic widget, not a component; a bare name is enough when the defaults fit.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `field` | `name` | yes |   | field of the command input or view row |
| `name` | `name` |   | `{from: field}` | node name |
| `label` | `string` |   | `{from: model.field_label}` | caption |
| `as` | one of: `text` \| `number` \| `secret` \| `toggle` \| `choice` \| `tags` \| `time` \| `date` \| `date_range` \| `duration` \| `file` \| `rich_text` \| `json` \| `list` \| `map` \| `cron` \| `color` \| `badge` \| `audio` \| `link` \| `metric` |   | `text` | semantic widget |
| `choice` | [Node](#node) |   |   | the choice composite for `as: choice` |
| `sortable` | `boolean` |   | `false` | column can sort |
| `visible` | `expr` |   |   | shows the field only when true |
| `binds` | `expr` |   |   | bind to UI state instead of the command input |
| `note` | `string` |   |   | author remark |

**You may also write**

| Where | You write | It means |
|---|---|---|
| the value itself | `string` | `{field: $value, name: $value}` |
| `name` | nothing: leave it out | `{first_present: [field]}` |

**Example**

```yaml
field: tier
as: choice
choice: {component: choice, options: PartnerTier}
```

### Tab

One tab of a form, record or overlay.

A tab holds fields, or a nested node, or an action such as opening another overlay.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` | yes |   | node name |
| `label` | `string` |   |   | tab text |
| `visible` | `expr` |   |   | shows the tab only when true |
| `fields` | one of: list of [Field](#field) \| `string` |   |   | fields in the tab, or an UNMAPPED string |
| `form` | one of: [Node](#node) \| [Action](#action) |   |   | nested node or an action |

**Example**

```yaml
name: schedule
label: Schedule
fields: [{field: close_date, as: date}]
```

### FormGroup

A group of settings fields saved on its own.

Settings pages save each group independently, often on change.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` | yes |   | node name |
| `label` | `string` |   |   | group heading |
| `fields` | list of [Field](#field) | yes |   | fields in the group |
| `save` | one of: `with_form` \| `on_change` |   | `with_form` | when the group saves |
| `does` | name of an ESS `command` |   |   | command for this group if not the form's |
| `actions` | list of [Action](#action) |   |   | extra actions such as rotating a key |
| `endpoint` | `string` |   |   | traceability |

**Example**

```yaml
name: profile
label: Profile
save: on_change
fields: [name, website, {field: logo, as: file}]
```

### Composite

The composite union — one type whose member is chosen by `component`.

Every composite kind is a member of one union discriminated by `component`; the member's props follow inline beside the fields below, and a key the member does not declare is refused, naming the node's path. A composite written where a nested node is expected, a section and an overlay all hold one member. `header` and `overlay` are composites placed by position, so they are not members. A `component` that names no member names a widget. The bare-name shorthand accepts a member only: a widget is always written `{component: <widget>, args: …}`.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `component` | name of a [Composite](#composite) | yes |   | kind of the nested composite |
| `name` | `name` |   |   | node name, required inside lists |
| `state` | map of `name` → [State](#state) |   |   | state local to this composite |
| `visible` | `expr` |   |   | shows the composite only when true |
| `degrades` | [Degrades](#degrades) |   |   | fallbacks for this composite |
| `unmapped` | list of `string` |   |   | gaps found by a retrofit |

**You may also write**

| Where | You write | It means |
|---|---|---|
| the value itself | name of a [Composite](#composite) | `{component: $value}` |

**Union**

```yaml
tag: component
members:
  - collection
  - record
  - form
  - choice
  - filter_bar
  - confirm
  - metric
  - chart
  - board
  - graph_editor
  - rich_text
  - references
other_tag_values: {ref: widget}
```

**Example**

```yaml
component: metric
from: channel.metrics.open_deals
label: Open deals
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `widget_expands` | `**.component naming a widget` | `{must: [resolve_widget, required_args_present, args_match_param_types, no_recursion], then: check_expanded_body}` | error |

### Degrades

Fallbacks for renderers that lack a capability.

A renderer profile lists the capabilities it lacks. A construct declares a fallback per capability; otherwise the capability table's first fallback applies, and `refuse` stops generation.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (capability) | one of: `no_free_layout` \| `no_charts` \| `no_graph_editor` \| `no_rich_text` \| `no_drawer` \| `no_drag` \| `no_audio` \| `no_file_upload` \| `no_live` \| `no_iframe` \| `no_columns` \| `no_areas` |   |   | key of the degrades map; the value is a fallback name |

**Rule**

```yaml
when: {target_lacks: $capability, construct_uses: $capability}
use: [construct.degrades.$capability, "capabilities.$capability.fallbacks[0]"]
if_none: refuse
```

**Capabilities**

```yaml
no_free_layout: {fallbacks: [stack], applies_to: [board]}
no_charts: {fallbacks: [table, metric], applies_to: [chart, board]}
no_graph_editor: {fallbacks: [collection, refuse], applies_to: [graph_editor]}
no_rich_text: {fallbacks: [plain], applies_to: [rich_text, form]}
no_drawer: {fallbacks: [dialog, fullscreen], applies_to: [overlay]}
no_drag: {fallbacks: [move_buttons], applies_to: [collection, board, graph_editor]}
no_audio: {fallbacks: [link], applies_to: [collection, record]}
no_file_upload: {fallbacks: [refuse], applies_to: [form, Action]}
no_live: {fallbacks: [poll, refuse], applies_to: [Live, Channel]}
no_iframe: {fallbacks: [link], applies_to: [board, record]}
no_columns: {fallbacks: [stack], applies_to: [PageLayout], order: section_declaration}
no_areas: {fallbacks: [stack], applies_to: [PageLayout], order: section_declaration}
```

**Example**

```yaml
no_graph_editor: collection
no_drag: move_buttons
```

### Primitive

The closed set of nine renderer-neutral leaves.

Composites are the normal level of a spec. Primitives exist for the small pieces inside widgets, item templates and section children: a caption, a toned badge, a button, an image. Every primitive has `primitive` (its kind) and `name`; the kinds and their props are the constructs text, badge, icon, button, link, input, toggle, image and divider.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `primitive` | one of: `text` \| `badge` \| `icon` \| `button` \| `link` \| `input` \| `toggle` \| `image` \| `divider` | yes |   | kind of primitive |
| `name` | `name` |   |   | node name, required inside lists |
| `visible` | `expr` |   |   | shows the primitive only when true |
| `state` | map of `name` → [State](#state) |   |   | state local to the primitive |
| `degrades` | [Degrades](#degrades) |   |   | fallbacks for renderers lacking a capability |

**Tone**

```yaml
enum: [neutral, info, success, warning, danger]
```

**Example**

```yaml
name: hint
primitive: text
text: "Tip: drop a CSV here to import partners"
style: caption
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `primitive_props` | `**.primitive` | `{must: props_of_kind_only}` | error |

### text

A run of text, literal or from a field, with a style and a value format.

Use for captions, headings and formatted values inside widgets and templates.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `text` | `expr` |   |   | literal or expression |
| `field` | `name` |   |   | row field shown |
| `style` | one of: `body` \| `caption` \| `heading` \| `mono` |   | `body` | typographic role |
| `format` | one of: `plain` \| `number` \| `currency` \| `percent` \| `date` \| `time` \| `duration` |   | `plain` | value formatting |
| `currency` | `expr` |   |   | currency code for format currency |

**Exactly one of**

`text`, `field`

**Example**

```yaml
name: amount
primitive: text
text: args.value.amount
format: currency
currency: args.value.currency
```

### badge

A short value in a toned pill.

For statuses and tiers. `tone` is fixed; `tone_by` picks a tone from the value.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `text` | `expr` |   |   | literal or expression |
| `field` | `name` |   |   | row field shown |
| `tone` | one of: `neutral` \| `info` \| `success` \| `warning` \| `danger` |   | `neutral` | fixed tone |
| `tone_by` | record \{ `value`: `expr`, `map`: map of `string` → (one of: `neutral` \| `info` \| `success` \| `warning` \| `danger`) \} |   |   | tone per value |

**Exactly one of**

`text`, `field`

**Example**

```yaml
name: badge
primitive: badge
text: args.status
tone_by: {value: args.status, map: args.tones}
```

### icon

A semantic icon with an accessible label.

The renderer maps the semantic name to its icon set; a TUI shows a glyph or the label.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `icon` | `string` | yes |   | semantic icon name |
| `tone` | one of: `neutral` \| `info` \| `success` \| `warning` \| `danger` |   | `neutral` | colour role |
| `label` | `string` | yes |   | accessible text |

**Example**

```yaml
name: typing
primitive: icon
icon: typing
tone: info
label: Someone is typing
visible: channel.ticket_chat.typing
```

### button

A button that runs one action.

Use inside widgets and section children; page-level actions belong in the header.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `label` | `string` | yes |   | button text |
| `action` | [Action](#action) | yes |   | what the button does |
| `tone` | one of: `primary` \| `secondary` \| `danger` \| `ghost` |   | `secondary` | emphasis |
| `icon` | `string` |   |   | optional semantic icon |

**Example**

```yaml
name: send
primitive: button
label: Send
tone: primary
action: {name: send, does: tickets.PostReply}
```

### link

Text that navigates to a page or an external address.

Prefer `to` (a page of this app); `href` is for external addresses.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `text` | `expr` | yes |   | link text |
| `to` | record \{ `to`: name of a [Page](#page), `params`: optional map of `name` → `expr` \} |   |   | page to open |
| `href` | `expr` |   |   | external address |

**Exactly one of**

`to`, `href`

**Example**

```yaml
name: open
primitive: link
text: Open
to: {to: partners.detail, params: {id: args.partner.id}}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `page_refs` | `**.navigate.to`, `**.to.to`, `pages.*.switch_to[]`, `shells.*.guards[].then.redirect` | `{must_resolve: {ref: page}}` | error |

### input

A single free input bound to state or a draft field.

For inputs outside a form's field list — a settings search, an inline value.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `as` | one of: `text` \| `number` \| `search` \| `date` \| `time` \| `secret` |   | `text` | input type |
| `binds` | `expr` | yes |   | state or draft field written |
| `placeholder` | `string` |   |   | hint text |

**Example**

```yaml
name: api_help
primitive: input
as: search
placeholder: Search settings
binds: state.settings_search
```

### toggle

An on/off switch bound to state, or running an action.

Bind to UI state with `binds`, or run a command with `action`.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `label` | `string` | yes |   | switch text |
| `binds` | `expr` |   |   | boolean written |
| `action` | [Action](#action) |   |   | command run on change |

**Exactly one of**

`binds`, `action`

**Example**

```yaml
name: api_toggle
primitive: toggle
label: Allow API access
binds: draft.api_enabled
```

### image

An image with required alternative text.

A renderer without images (a TUI) shows the alt text.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `src` | `expr` | yes |   | image address |
| `alt` | `string` | yes |   | alternative text |
| `fit` | one of: `cover` \| `contain` |   | `contain` | scaling |

**Example**

```yaml
name: logo
primitive: image
src: args.partner.logo_url
alt: Partner logo
fit: contain
```

### divider

A visual separator.

Separates groups of nodes inside widgets and children.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `orientation` | one of: `horizontal` \| `vertical` |   | `horizontal` | direction |

**Example**

```yaml
name: rule
primitive: divider
```

## Reads, actions and fixtures

How the UI reads ESS views, runs ESS commands, and runs without a backend.

### Reads

The ESS view a section or composite reads — or, while designing, a named placeholder backed by a fixture.

Every piece of data on screen comes from an ESS view. Params bind page state; `paging` says who pages. While a screen is designed before its model exists, write `placeholder` with a view name and a `fixture` file instead of `view`; renderers read the fixture, validators report the placeholder as a warning until it is bound. `endpoint` and `derived` are traceability for retrofits.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `view` | name of an ESS `view` |   |   | ESS view name |
| `placeholder` | `name` |   |   | a view name not yet bound to the model |
| `fixture` | `string` |   |   | fixture file answering the placeholder |
| `params` | map of `name` → `expr` |   |   | view params bound to state |
| `paging` | one of: `server` \| `client` \| `cursor` \| `append` \| `none` |   | `none` | who pages |
| `debounce` | `duration` |   |   | coalesce param changes before reading |
| `refresh` | one of: `duration` \| `expr` |   |   | poll interval |
| `cache` | record \{ `store`: [Store](#store), `ttl`: optional `duration` \} |   | `{store: {resolve: {class: view_cache}}}` | where the result is cached |
| `endpoint` | `string` |   |   | traceability to the HTTP call replaced |
| `derived` | `string` |   |   | traceability when data is computed client-side today |

**You may also write**

| Where | You write | It means |
|---|---|---|
| the value itself | name of an ESS `view` | `{view: $value}` |

**Exactly one of**

`view`, `placeholder`

**Constraints**

```yaml
- {when: {paging: client}, requires: {view.bounded: true}, else: refuse}
- {when: {placeholder: present}, requires: {fixture: present}, else: refuse}
```

**Example**

```yaml
placeholder: forecast.Quarterly
fixture: fixtures/forecast.yaml
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `unbound_placeholder` | `**.reads.placeholder` | `{must: bind_to_view}` | warning |
| `fixture_per_view` | `**.reads.view` | `{must_be_in: [fixtures.views, fixtures.derived]}` | warning |

### Action

One user-triggered effect: run a command (`does`), open an overlay, navigate, export, upload, copy or set UI state.

`does` names an ESS command; its input is bound from the row, selection or state, and its typed errors are shown where the action happened. There is no reload to declare: the command's outcome names the entities it changed, and every section reading a view over them is patched or re-read. Visibility follows actor grants; `visible` adds a UI condition.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `name` | `name` |   |   | node name among sibling actions; derived when absent, except for a `copy` action, whose expression is no name |
| `does` | name of an ESS `command` |   |   | ESS command to run |
| `bind` | map of `name` → `expr` |   |   | command input from row, selection or state |
| `opens` | name of an [overlay](#overlay) |   |   | overlay to open |
| `navigate` | record \{ `to`: name of a [Page](#page), `params`: optional map of `name` → `expr` \} |   |   | page to go to |
| `export` | record \{ `reads`: name of an ESS `view`, `as`: (one of: `csv` \| `json` \| `zip` \| `png`), `params`: optional `expr` \} |   |   | download a view |
| `upload` | record \{ `accept`: list of `string`, `does`: name of an ESS `command` \} |   |   | file picker feeding a command |
| `copy` | `expr` |   |   | value copied to the clipboard |
| `sets` | map of `expr` → (one of: `expr` \| exactly `toggle`) |   |   | UI state change only, never business data |
| `label` | `string` |   |   | button or menu text |
| `as` | one of: `button` \| `icon` \| `toggle` \| `choice` \| `menu_item` \| `link` |   | `button` | presentation hint |
| `choice` | [Node](#node) |   |   | options for as choice |
| `loads` | [Reads](#reads) |   |   | current value for a header toggle or choice |
| `confirm` | one of: name of an [overlay](#overlay) \| record \{ `title`: `string`, `show`: optional `expr`, `confirm_label`: optional `string` \} |   |   | confirm first |
| `optimistic` | `boolean` |   | `false` | apply the expected outcome at once and revert on refusal; requires: `{does.outcome: unique_for_input}` |
| `bulk` | `boolean` |   | `false` | applies to the collection's selection |
| `visible` | `expr` |   |   | UI condition beyond grants |
| `endpoint` | `string` |   |   | traceability |

**You may also write**

| Where | You write | It means |
|---|---|---|
| `sets.*` | exactly `toggle` | `{expr: not $key}` |
| `confirm` | record \{ `title`: `string`, `show`: optional `expr`, `confirm_label`: optional `string` \} | `{overlay: {kind: dialog, component: confirm, title: $value.title, does: $action.does, confirm_label: $value.confirm_label}, show: $value.show}` |
| `name` | nothing: leave it out | `{first_present: [opens, does.last_segment_snake_case, navigate.to, export.reads, upload.does, sets.first_key]}` |

**Exactly one of**

`does`, `opens`, `navigate`, `export`, `upload`, `copy`, `sets`

**Example**

```yaml
- name: move
  does: deals.MoveDeal
  bind: {deal_id: row.id}
  as: choice
  choice: {component: choice, options: DealStage}
  optimistic: true
- {name: import, upload: {accept: [text/csv], does: partners.ImportPartners}, label: Import CSV}
- name: export
  export: {reads: invoices.Page, as: csv, params: same_as(list)}
  label: Export CSV
- name: remind
  does: invoices.SendReminder
  bind: {invoice_id: row.id}
  visible: row.status == overdue
  confirm: {title: Send payment reminder}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `opens_resolves` | `**.opens` | `{must_resolve_in: [page.overlays, page.kind.overlays, shell.overlays]}` | error |
| `page_refs` | `**.navigate.to`, `**.to.to`, `pages.*.switch_to[]`, `shells.*.guards[].then.redirect` | `{must_resolve: {ref: page}}` | error |

### FixtureIndex

Sample data per view and event scripts per channel, so renderers run without a backend.

In fixture mode a renderer answers every read from a fixture file and plays each channel's script in a loop, honouring delivery and coalesce. A file holds one view or several; `derived` answers ById views from a list fixture. Placeholders name their fixture on the read itself.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `dir` | `string` |   |   | fixture directory relative to the document |
| `index` | `string` |   |   | file holding views, derived and scripts |
| `views` | map of name of an ESS `view` → `string` |   |   | view to fixture file |
| `derived` | map of name of an ESS `view` → (one of: record \{ `by_id_from`: name of an ESS `view`, `key`: `name` \} \| record \{ `same_as`: name of an ESS `view` \}) |   |   | views answered from another fixture |
| `scripts` | map of name of a [Channel](#channel) → `string` |   |   | channel to event script file |

**Script entry**

```yaml
type:
  one_of:
    - {record: {at: duration, event: {ref: event}, payload: json}}
    - record:
        at: duration
        lifecycle: {enum: [connecting, loading, live, reconnecting, stale, closed]}
        payload: {optional: json}
note: "an event to deliver, or a connection lifecycle change to simulate"
```

**View file**

```yaml
type:
  one_of:
    - record:
        view: {ref: view}
        total: {optional: integer}
        by_params: {optional: json}
        rows: {list: json}
    - record:
        views:
          map:
            key: {ref: view}
            value:
              record:
                total: {optional: integer}
                by_params: {optional: json}
                rows: {list: json}
note: "one view per file, or several; by_params records which params the rows answer"
```

**Example**

```yaml
dir: fixtures
views: {partners.Page: portal.yaml, deals.Page: portal.yaml}
derived: {partners.ById: {by_id_from: partners.Page, key: id}}
scripts: {activity: scripts/activity.yaml}
```

## Live data

Channels and how sections apply their events.

### Channel

A live source of ESS events or a live view, with delivery and resume semantics.

A channel says what it carries and in which direction; the renderer picks the transport. `delivery: latest_value` may drop intermediate values (metrics); `every_event` may not (an activity feed). `resume` says what happens after a reconnect. The connection lifecycle is UI state, and a stale value is marked. Scope is decided by the ESS model.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `carries` | one of: record \{ `events`: list of name of an ESS `event` \} \| record \{ `view`: name of an ESS `view` \} | yes |   | events (deltas) or a live view (replacing value) |
| `direction` | one of: `server_to_client` \| `both` | yes |   | both when the client also sends commands on it |
| `sends` | list of name of an ESS `command` |   |   | commands sent over the channel |
| `delivery` | one of: `latest_value` \| `every_event` | yes |   | whether values may be dropped |
| `resume` | one of: `from_last_seen` \| `refetch` | yes |   | behaviour after reconnect |
| `scope` | one of: `account` \| `user` \| `session` \| `param` |   | `account` | which events reach the actor |
| `session` | record \{ `per`: `name` \} |   |   | one channel instance per value |
| `lifecycle` | list of (one of: `connecting` \| `loading` \| `live` \| `reconnecting` \| `stale` \| `closed`) |   |   | connection states shown to the user |
| `reconnect` | record \{ `backoff`: record \{ `from`: `duration`, `to`: `duration` \} \} |   |   | reconnect backoff |
| `stale_after` | `duration` |   |   | down time after which fed sections are stale |
| `buffer` | [State](#state) |   |   | class channel\_buffer: unapplied events and last-seen cursor |
| `fields` | map of `name` → `string` |   |   | named fields readable as channel.\<name\>.\<field\> |
| `transport_today` | `string` |   |   | traceability only; never read by a renderer |
| `source` | list of `string` |   |   | traceability |
| `unmapped` | list of `string` |   |   | gaps found by a retrofit |

**Transport choice**

```yaml
- declared: direction server_to_client
  renderer_picks: "SSE, or long polling where SSE is unavailable"
- {declared: direction both, renderer_picks: WebSocket}
- {declared: no channel and a read with refresh, renderer_picks: polling}
```

**Example**

```yaml
carries: {events: [tickets.ReplyPosted, tickets.TypingStarted]}
direction: both
sends: [tickets.PostReply, tickets.StartTyping]
delivery: every_event
resume: from_last_seen
scope: param
session: {per: ticket_id}
buffer: {type: {list: ChatEvent}, class: channel_buffer}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `script_per_channel` | `channels.*` | `{must_be_in: [fixtures.scripts]}` | warning |

### Live

How a section applies a channel's events to its rows.

`effect` decides what an event does: patch a row, insert or patch, insert at the top of a feed, remove, replace, or re-read. `only_if: matches(params)` drops live rows outside the section's filters; `when_paged_away: count_new` shows "12 new" instead of shifting rows.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `channel` | name of a [Channel](#channel) | yes |   | channel to consume |
| `on` | list of name of an ESS `event` |   |   | subset of the channel's events |
| `effect` | one of: `patch_row` \| `insert_or_patch` \| `insert_top` \| `remove_row` \| `replace` \| `refetch` | yes |   | what an event does to the rows |
| `match` | `name` |   | `id` | row identity field |
| `only_if` | `expr` |   |   | drop events that fail the condition |
| `coalesce` | `duration` |   |   | batch bursts into one render |
| `when_paged_away` | one of: `count_new` \| `ignore` \| `insert` |   | `count_new` | behaviour when the reader is not on page one |
| `max_rows` | `integer` |   |   | cap for insert\_top feeds |

**Example**

```yaml
channel: tickets
effect: insert_or_patch
only_if: matches(params)
when_paged_away: count_new
coalesce: 500ms
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `channel_refs` | `**.live.channel`, `**.header.live[]`, `channel.<name> inside expr` | `{must_resolve: {ref: channel}}` | error |

## State placement

Where every piece of UI state lives, and how a profile moves it.

### State

One piece of UI state, its class and where it is stored.

Every piece of state — filters, a selection, an open drawer, a draft, a cached view, a channel buffer, a preference, a token — is declared with a `class`. Its `store` is explicit or resolved from the placement profile, so one document deploys as a thin or a fat client. `pinned` keeps a placement under any profile; `sensitive` forbids browser storage and the URL.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `type` | [Type](#type) | yes |   | structural type of the value |
| `class` | [StateClass](#stateclass) | yes |   | what kind of state this is |
| `store` | [Store](#store) |   | `{resolve: PlacementProfile.resolution}` | explicit placement |
| `pinned` | `boolean` |   | `false` | no profile may move it |
| `default` | `json` |   |   | initial value |
| `scope` | one of: `tab` \| `device` \| `session` \| `user` \| `account` \| `string` |   | `{by_store: {memory: tab, url: tab, session_storage: tab, local_storage: device, server_session: session, server: user}}` | sharing (string only for an UNMAPPED marker) |
| `sensitive` | `boolean` |   | `false` | forbids url, session\_storage and local\_storage |
| `ttl` | `duration` |   |   | expiry |
| `clear_on` | list of (one of: `signout` \| `account_switch` \| `submit` \| `close`) |   | `[signout, account_switch]` | events that clear it |
| `fallback` | record \{ `when`: `expr`, `store`: [Store](#store) \} |   |   | second placement used when the condition holds |
| `endpoint` | `string` |   |   | traceability for store server |
| `note` | `string` |   |   | author remark |

**Example**

```yaml
layout:
  type: {list: {record: {widget: string, x: integer, "y": integer, w: integer, h: integer}}}
  class: preference
  store: server
  scope: user
  fallback: {when: not actor.may(dashboards.SaveLayout), store: local_storage}
```

**Checks**

| Check | Subject | Rule | Severity |
|---|---|---|---|
| `state_resolves` | `**.state.*` | `{must: resolve_to_store, refusals: PlacementProfile.resolution.refusals}` | error |

### StateClass

The kind of a state — the key the placement profile uses.

A profile maps each class to a default store; the class also decides what reload and sign-out do.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (value) | one of: `page_state` \| `selection` \| `component_state` \| `draft` \| `view_cache` \| `channel_buffer` \| `preference` \| `credential` |   |   | one of the classes below |

**Meaning**

```yaml
page_state: "filters, search, paging, sort, mode — what a link should reproduce"
selection: rows or a record the user picked; drives dependent sections and overlays
component_state: "open/closed, expanded rows, active tab, scroll, tickers"
draft: unsaved form input
view_cache: "the last result of a read (rows, total, cursor)"
channel_buffer: "events received but not applied, plus the last-seen cursor"
preference: "per-user choices that outlive a page (theme, nav expanded, columns, layouts, positions)"
credential: tokens held for the actor; always sensitive
```

**Example**

```yaml
type: {list: PartnerId, unique: true}
class: selection
```

### Store

Where a state lives, with its durability, sharing, and reload/reconnect behaviour.

Six stores from most local to authoritative. memory is lost on unmount; url travels with links; session\_storage survives a reload in one tab; local\_storage survives restarts on one device; server\_session lives on the server for one login session (for example in a key-value store) and is shared by its tabs; server is an ESS entity or view, durable and changed only by commands.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| (value) | one of: `memory` \| `url` \| `session_storage` \| `local_storage` \| `server_session` \| `server` |   |   | one of the stores below |

**Meaning**

```yaml
memory:
  durability: "lost on unmount, reload and tab close"
  sharing: one component instance in one tab
  on_reload: gone; sections re-read and drafts are lost
  on_reconnect: kept; the channel's resume decides replay or re-read
url:
  durability: "survives reload, back/forward and bookmarks"
  sharing: whoever holds the link
  on_reload: restored exactly
  on_reconnect: unaffected
  constraints:
    serializable: true
    max_bytes_per_page: 2048
    allows_sensitive: false
    sent_with_each_request_in_thin: true
session_storage:
  durability: survives reload; lost when the tab closes; a duplicated tab gets a copy
  sharing: one tab
  on_reload: restored before the first read
  on_reconnect: unaffected
  constraints:
    allows_sensitive: false
    max_bytes_per_app: 1048576
    key: [origin, actor.user_id, actor.account_id]
local_storage:
  durability: survives reload and browser restart until the profile is cleared or clear_on fires
  sharing: "one device (browser profile), all its tabs, synced by the storage event"
  on_reload: restored before first paint
  on_reconnect: unaffected
  constraints: {allows_sensitive: false, key: [origin, actor.user_id, actor.account_id]}
server_session:
  durability: "survives reload, tab close and reconnect; lost on sign-out, account switch or session expiry"
  sharing: every tab of the same login session; not other devices or users
  on_reload: the server restores it and sends the rendered view model
  on_reconnect: restored; buffered events let the server replay what the client missed
  constraints: {bounded_per_session: true, authoritative: false}
server:
  durability: durable
  sharing: by scope — user (a preference entity) or account (a shared record); other sessions see changes through channels
  on_reload: re-read
  on_reconnect: re-read or replayed by the channel's resume
  constraints: {writes_via: ess_command}
```

**Matrix**

```yaml
- store: memory
  survives_reload: false
  survives_tab_close: false
  survives_signout: false
  across_tabs: false
  across_devices: false
  across_users: false
- store: url
  survives_reload: true
  survives_tab_close: via_link
  survives_signout: via_link
  across_tabs: via_link
  across_devices: via_link
  across_users: via_link
- store: session_storage
  survives_reload: true
  survives_tab_close: false
  survives_signout: false
  across_tabs: false
  across_devices: false
  across_users: false
- store: local_storage
  survives_reload: true
  survives_tab_close: true
  survives_signout: unless_cleared
  across_tabs: true
  across_devices: false
  across_users: false
- store: server_session
  survives_reload: true
  survives_tab_close: true
  survives_signout: false
  across_tabs: true
  across_devices: false
  across_users: false
- store: server
  survives_reload: true
  survives_tab_close: true
  survives_signout: true
  across_tabs: true
  across_devices: true
  across_users: when_scope_account
```

**Example**

```yaml
type: {enum: [light, dark]}
class: preference
store: local_storage
pinned: true
default: light
```

### PlacementProfile

A default store per state class — thin, fat or hybrid.

The profile is how one document deploys as a thin or a fat client. thin keeps everything but the URL on the server: a presenter executes reads, holds drafts and applies channel events, and the client paints view models. fat keeps UI state in the browser and talks to ESS views, commands and channels directly. hybrid lets each page or section pick.

**Properties**

| Property | Type | Required | Default | Note |
|---|---|---|---|---|
| `idea` | `string` |   |   | one line on the deployment shape |
| `reads` | `string` |   |   | who executes reads |
| `channels` | `string` |   |   | who applies channel events |
| `defaults` | map of [StateClass](#stateclass) → [Store](#store) |   |   | default store per class |

**Profiles**

```yaml
thin:
  idea: the server holds all state except what the URL carries; the client renders view models and forwards input
  reads: a server-side presenter
  channels: "the server subscribes, applies events to the session's view cache and pushes rendered patches"
  defaults:
    page_state: url
    selection: server_session
    component_state: server_session
    draft: server_session
    view_cache: server_session
    channel_buffer: server_session
    preference: server
    credential: server_session
fat:
  idea: the client holds and computes UI state; the server is only the ESS model
  reads: "the client, against ESS views"
  channels: the client subscribes and applies events itself
  defaults:
    page_state: url
    selection: memory
    component_state: memory
    draft: session_storage
    view_cache: memory
    channel_buffer: memory
    preference: local_storage
    credential: memory
hybrid:
  idea: each page or section picks thin or fat with profile
  reads: per section
  channels: "server-applied when every fed section is thin, else delivered to the client"
  defaults: {}
  unresolved: refuse
```

**Resolution**

```yaml
order:
  - {step: explicit_store, source: state.store}
  - {step: pinned, source: state.store, when: {state.pinned: true}, effect: profile_ignored}
  - {step: section_profile, source: "profiles[section.profile].defaults[state.class]"}
  - {step: page_profile, source: "profiles[page.profile].defaults[state.class]"}
  - {step: document_defaults, source: "document.placement_defaults[state.class]"}
  - step: profile_defaults
    source: "profiles[document.placement_profile].defaults[state.class]"
refusals:
  - when: {state.sensitive: true, store: [url, session_storage, local_storage]}
    result: refuse
  - {when: {state.class: credential, store_not_in: [memory, server_session]}, result: refuse}
  - {when: {store: url, value_serializable: false}, result: refuse}
  - {when: {store: url, value_bytes_over: 2048}, result: refuse}
  - {when: {renderer.thin_only: true, resolved_by_profile: fat}, result: refuse}
```

**Reload and reconnect**

```yaml
reload:
  memory: the section re-enters loading; a renderer warns on unload while a memory draft is dirty
  url: page state restored; sections re-read with the same params
  session_storage: restored before the first read; drafts refill their forms
  local_storage: "restored before first paint; a cached view paints stale-marked, then refreshes"
  server_session: the server renders the page from session state
  server: re-read
reconnect:
  memory: "kept; resume refetch re-reads, from_last_seen replays from the buffer cursor"
  server_session: "the server replays buffered events or re-reads, then pushes one patch"
  server: unaffected
```

**Example**

```yaml
placement_profile: hybrid
placement_defaults:
  page_state: url
  selection: memory
  draft: session_storage
  preference: server
  credential: server_session
pages:
  overview: {kind: dashboard_page, profile: fat}
  invoices.list: {kind: report_page, profile: thin}
```

## Checks

What a validator checks in a document, and the construct each check applies to. Every finding names the failing node by its [NodePath](#nodepath).

| Check | Applies to | Subject | Rule | Severity |
|---|---|---|---|---|
| `names_unique` | every node | `every node` | `{must: unique_name_among_siblings}` | error |
| `nav_resolves` | [Navigation](#navigation) | `navigation.sections[].pages[]`, `navigation.hidden[]`, `navigation.home` | `{must_resolve: {ref: page}}` | error |
| `page_reachable` | [Page](#page) | `pages.*` | `{must_be_in: ["navigation.sections[].pages[]", "navigation.hidden[]", "navigation.sections[].pages.page"]}` | error |
| `opens_resolves` | [Action](#action) | `**.opens` | `{must_resolve_in: [page.overlays, page.kind.overlays, shell.overlays]}` | error |
| `same_as_resolves` | [overlay](#overlay) | `**.same_as` | `{must_resolve: {ref: overlay}}` | error |
| `page_refs` | [Action](#action), [link](#link), [Page](#page), [Guard](#guard) | `**.navigate.to`, `**.to.to`, `pages.*.switch_to[]`, `shells.*.guards[].then.redirect` | `{must_resolve: {ref: page}}` | error |
| `channel_refs` | [Live](#live), [header](#header) | `**.live.channel`, `**.header.live[]`, `channel.<name> inside expr` | `{must_resolve: {ref: channel}}` | error |
| `section_refs` | [header](#header), [Section](#section), [PageLayout](#pagelayout) | `**.header.total`, `**.header.filters`, `**.depends_on`, `pages.*.layout.columns[].sections[]`, `pages.*.layout.areas.place.*[]` | `{must_resolve_in: [page.sections]}` | error |
| `layout_complete` | [PageLayout](#pagelayout) | `pages.*.layout` | `{must: place_every_section_once}` | warning |
| `widget_expands` | [Section](#section), [overlay](#overlay), [WidgetInstance](#widgetinstance), [Composite](#composite) | `**.component naming a widget` | `{must: [resolve_widget, required_args_present, args_match_param_types, no_recursion], then: check_expanded_body}` | error |
| `primitive_props` | [Primitive](#primitive) | `**.primitive` | `{must: props_of_kind_only}` | error |
| `unbound_placeholder` | [Reads](#reads) | `**.reads.placeholder` | `{must: bind_to_view}` | warning |
| `fixture_per_view` | [Reads](#reads) | `**.reads.view` | `{must_be_in: [fixtures.views, fixtures.derived]}` | warning |
| `script_per_channel` | [Channel](#channel) | `channels.*` | `{must_be_in: [fixtures.scripts]}` | warning |
| `state_resolves` | [State](#state) | `**.state.*` | `{must: resolve_to_store, refusals: PlacementProfile.resolution.refusals}` | error |
| `types_structural` | [Type](#type) | `**.type` | `{must_match: type_rule, forbidden: string_holding_a_type_expression}` | error |
| `unmapped_reported` | every node | `**` | `{matches: unmapped_marker.pattern, action: report}` | warning |

## Retrofits and lowering

How a document records what a retrofit could not determine, where it came from, and what each construct becomes in the ESS model.

### Unmapped marker

How a retrofit marks a value it could not determine.

```yaml
example: {endpoint: "UNMAPPED: the submit handler is not wired to an API call"}
pattern: "^UNMAPPED: .+$"
accepted_by:
  types: [string, expr, name, duration, "{enum: …}", "{ref: …}"]
  note: "a field of any other type (boolean, a number, a record, a list or a map) refuses the marker"
at_reads: a marker written as a `reads` or `loads` is no read at all; the node records it in its `unmapped` list
validator: {action: report, severity: warning}
renderer: {action: treat_as_absent}
```

### Traceability

| Key | Carried |
|---|---|
| `endpoint` | on Reads, Action, forms and overlays — the HTTP call of a retrofitted app |
| `source` | on Page, Shell and Channel — files the construct was read from |
| `transport_today` | on Channel — topic or URL used today |
| `roles_today` | on NavEntry — role list used today |
| `unmapped` | every value matching unmapped\_marker.pattern, and every unmapped list entry |

### Lowering

| Construct | Lowers to |
|---|---|
| reads (view, params) | ESS view with params, filter and paging |
| reads placeholder | nothing until bound; reported as a warning |
| does | ESS command; the form is its input, refusals are its typed errors |
| visibility and shown actions | actor grants |
| live and channel events | ESS events published by the model's components |
| channel delivery and resume | component reach and binding delivery guarantees; streaming delivery is new |
| state in store server | an ESS entity (a preference entity for class preference) and its view |
| state in store server\_session | a presentation domain in ESS (an entity per session) or host session storage |
| state in memory, url, session\_storage, local\_storage | nothing; renderer-owned |
| shell, pages, layouts, sections, composites, widgets, primitives, states, degrades | UiIr (new) |

## Quick reference

Every construct with its one-line summary, chapter by chapter.

| Construct | Chapter | Summary |
|---|---|---|
| [Document](#document) | [The document](#the-document) | The root of a ess-ui/1 file describing one application. |
| [Type](#type) | [The document](#the-document) | A named value type declared once in the document's `types` map. |
| [NodePath](#nodepath) | [The document](#the-document) | The stable address of any node — shell region, page, section, overlay, composite, widget instance or primitive. |
| [Shell](#shell) | [Shell and navigation](#shell-and-navigation) | An application frame with regions, preloads, guards and shell-wide state. |
| [Region](#region) | [Shell and navigation](#shell-and-navigation) | One named area of a shell. |
| [Preload](#preload) | [Shell and navigation](#shell-and-navigation) | Views a shell reads before or alongside the first page. |
| [Guard](#guard) | [Shell and navigation](#shell-and-navigation) | A condition checked before a page opens, with a redirect, refusal or state change. |
| [Navigation](#navigation) | [Shell and navigation](#shell-and-navigation) | The menu — sections of pages, the home page, hidden routes and search. |
| [NavSection](#navsection) | [Shell and navigation](#shell-and-navigation) | One group of the menu. |
| [DynamicNavEntries](#dynamicnaventries) | [Shell and navigation](#shell-and-navigation) | Menu entries generated from the rows of a view. |
| [NavEntry](#naventry) | [Shell and navigation](#shell-and-navigation) | How a page appears in the menu, declared on the page as `nav`. |
| [Page](#page) | [Pages and sections](#pages-and-sections) | One route — its state, layout, header, sections and overlays. |
| [PageLayout](#pagelayout) | [Pages and sections](#pages-and-sections) | A renderer-neutral arrangement of a page's sections. |
| [PageKind](#pagekind) | [Pages and sections](#pages-and-sections) | A page template that fills the same sections, state, header and overlays. |
| [Section](#section) | [Pages and sections](#pages-and-sections) | A region of a page with one read and its own loading lifecycle. |
| [SectionStates](#sectionstates) | [Pages and sections](#pages-and-sections) | How a section renders each lifecycle state. |
| [collection](#collection) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Rows of a view with columns, sorting, paging, selection and row actions. |
| [record](#record) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | One row shown as labelled, read-only fields. |
| [form](#form) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Inputs bound to one ESS command's input. |
| [choice](#choice) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Pick one or many values from a view or a fixed list. |
| [filter\_bar](#filter_bar) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Search, choices, inputs and a time window bound to page state. |
| [header](#header) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Page title, count, primary actions, view switch, filters and live status. |
| [overlay](#overlay) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A drawer, dialog, fullscreen pane or popover shown on demand. |
| [confirm](#confirm) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A yes/no step before a command, listing references and consequences. |
| [metric](#metric) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | One number from a view or channel, with a window and a stale mark. |
| [chart](#chart) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A series view drawn as a chart. |
| [board](#board) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A user-arranged grid of widgets, persisted. |
| [graph\_editor](#graph_editor) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Nodes and edges of a model, editable on a canvas. |
| [rich\_text](#rich_text) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Text with expression completion or markup. |
| [references](#references) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | The "used by" list of a record. |
| [Widget](#widget) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | An app-defined composite with typed parameters, usable wherever a composite kind is. |
| [WidgetInstance](#widgetinstance) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A use of a widget, with arguments. |
| [Node](#node) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Anything that can appear inside a section, composite or widget — a composite, a widget instance or a primitive. |
| [Field](#field) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | One input of a form or one column of a collection. |
| [Tab](#tab) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | One tab of a form, record or overlay. |
| [FormGroup](#formgroup) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A group of settings fields saved on its own. |
| [Composite](#composite) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | The composite union — one type whose member is chosen by `component`. |
| [Degrades](#degrades) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Fallbacks for renderers that lack a capability. |
| [Primitive](#primitive) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | The closed set of nine renderer-neutral leaves. |
| [text](#text) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A run of text, literal or from a field, with a style and a value format. |
| [badge](#badge) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A short value in a toned pill. |
| [icon](#icon) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A semantic icon with an accessible label. |
| [button](#button) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A button that runs one action. |
| [link](#link) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | Text that navigates to a page or an external address. |
| [input](#input) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A single free input bound to state or a draft field. |
| [toggle](#toggle) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | An on/off switch bound to state, or running an action. |
| [image](#image) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | An image with required alternative text. |
| [divider](#divider) | [Composites, widgets and primitives](#composites-widgets-and-primitives) | A visual separator. |
| [Reads](#reads) | [Reads, actions and fixtures](#reads-actions-and-fixtures) | The ESS view a section or composite reads — or, while designing, a named placeholder backed by a fixture. |
| [Action](#action) | [Reads, actions and fixtures](#reads-actions-and-fixtures) | One user-triggered effect: run a command (`does`), open an overlay, navigate, export, upload, copy or set UI state. |
| [FixtureIndex](#fixtureindex) | [Reads, actions and fixtures](#reads-actions-and-fixtures) | Sample data per view and event scripts per channel, so renderers run without a backend. |
| [Channel](#channel) | [Live data](#live-data) | A live source of ESS events or a live view, with delivery and resume semantics. |
| [Live](#live) | [Live data](#live-data) | How a section applies a channel's events to its rows. |
| [State](#state) | [State placement](#state-placement) | One piece of UI state, its class and where it is stored. |
| [StateClass](#stateclass) | [State placement](#state-placement) | The kind of a state — the key the placement profile uses. |
| [Store](#store) | [State placement](#state-placement) | Where a state lives, with its durability, sharing, and reload/reconnect behaviour. |
| [PlacementProfile](#placementprofile) | [State placement](#state-placement) | A default store per state class — thin, fat or hybrid. |
