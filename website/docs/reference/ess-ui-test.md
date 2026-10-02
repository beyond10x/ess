---
title: "ess-ui-test/1 reference"
sidebar_label: "ess-ui-test/1"
description: "UI behaviour tests that select nodes by their document path, run headless against the terminal renderer and as a Playwright spec."
---

# ess-ui-test/1 reference

An `ess-ui-test/1` file tests the behaviour of an [`ess-ui/1`](./ess-ui.md) document. A test
selects nodes by their canonical node path, declares backend state as fixtures, plays live events
from the fixture scripts and moves a virtual clock. The same file runs headless against the
terminal renderer and is emitted as a Playwright spec for the generated React project.

```bash
ess ui test --path ui.yaml tests/*.yaml
ess ui test --path ui.yaml tests/*.yaml --format json
ess ui test --path ui.yaml tests/*.yaml --playwright e2e/ui.spec.ts
```

The first form runs every test and prints one line per test; the second prints an
`ess-ui-test-report/1` report. Either exits 1 when a test fails. The third writes the Playwright spec
and runs nothing.

## A test file

```yaml
format: ess-ui-test/1
document: ../ui.yaml
tests:
  - name: the search narrows the partner list
    steps:
      - open: partners.list
      - expect: {at: pages/partners.list/sections/list, rows: 6}
      - type: {at: pages/partners.list/sections/filters, text: cedar}
      - expect: {at: pages/partners.list/sections/list, rows: [pt-003]}
      - expect: {at: pages/partners.list/sections/list/rows/pt-003, text: Cedar Partners}

  - name: deleting a partner asks first and sends the command
    steps:
      - open: partners.list
      - act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete
      - expect: {at: pages/partners.list/overlays/delete, text: Delete partner}
      - act: pages/partners.list/overlays/delete
      - expect_command: {command: partners.DeletePartner, input: {id: pt-003}}
```

| Key | Meaning |
|---|---|
| `format` | Always `ess-ui-test/1`. |
| `document` | The document under test, relative to the test file. `--path` must name the same file. |
| `tests` | The tests, run in order, each from a freshly opened application. |
| `tests[].name` | The name the report uses. |
| `tests[].fixtures` | Optional. Replaces fixture data for this test only (see [Fixtures](#fixtures)). |
| `tests[].latency` | Optional duration. How long every read takes on the virtual clock; `loading` is visible until it passes. |
| `tests[].steps` | The steps. The first step that does not hold fails the test, and later steps do not run. |

The example application's own tests are in `examples/partner-portal/tests/`.

## Node paths

Every step that names a node uses its canonical node path: the chain of container keys and names
from the document root, joined with `/`. It is the path `ess ui check` reports a finding at, and
the path the generated React project renders as `data-ui-path`. A path holds no list index, so
adding, removing or reordering a sibling leaves every other path unchanged.

An item of a collection is its collection's path, `rows`, and the row's key: the value of the
field the section's `live.match` names, else `id`. A node inside a row is addressed under the row:

| Path | Names |
|---|---|
| `pages/partners.list` | a page |
| `pages/partners.list/sections/list` | a section |
| `pages/partners.list/sections/filters/choices/tier` | a choice in a filter bar |
| `pages/partners.list/sections/list/rows/pt-003` | the row whose key is `pt-003` |
| `pages/partners.list/sections/list/rows/pt-003/row_actions/delete` | that row's `delete` action |
| `pages/partners.list/header/actions/create` | a header action |
| `pages/invoices.list/sections/list/rows/in-03/row_actions/remind/confirm/overlay` | the inline confirm `remind` opened on row `in-03` |
| `pages/partners.list/overlays/create/fields/name` | a form field in an overlay |

An inline confirm (`confirm: {title: …}`) is the overlay at its action's path and
`confirm/overlay`; a row action's is addressed under the row that opened it. Confirming runs
the action that opened the confirm, inline or `confirm: <overlay>`, whether or not the confirm
declares `does`.

A path that names no node fails its step with `no node at <path>`; a row key the collection does
not hold fails with `no row <key> in <collection>`. A path on a page other than the one shown fails
naming both pages.

## Steps

A step is a map with one key, its keyword. Most steps fit on one line.

### open

Opens a page, with its params.

```yaml
- open: partners.list
- open: {page: partners.detail, params: {id: pt-001}}
```

### select

Focuses a section, or moves to a row (paging to it). Selecting the page closes an open overlay.

```yaml
- select: pages/partners.list/sections/list/rows/pt-005
```

### type

Types text into a filter bar's search, a collection's filter or a form field. Typing adds to what
the field holds.

```yaml
- type: {at: pages/partners.list/overlays/create/fields/name, text: Gum Tree Ltd}
```

### choose

Picks an option of a choice, by its value or its label: a filter bar's choice, or a form field drawn
`as: choice` (at the field or at its `choice` node). In a multiple choice a second pick of the same
option removes it.

```yaml
- choose: {at: pages/tickets.list/sections/filters/choices/priority, option: urgent}
```

### act

Runs an action: a row's action, a header action, a section's action. On an open overlay's own path
it runs the overlay's primary action — a confirm confirms, a form submits.

```yaml
- act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete
```

### page

Moves a collection to a page, counting from 1. The page is absolute: `to: 1` after `to: 2` goes
back to the first page.

```yaml
- page: {at: pages/partners.list/sections/list, to: 2}
```

### expect

Checks a node. Every key given must hold.

```yaml
- expect: {at: pages/partners.list/sections/list, rows: [pt-001, pt-005], not_text: Birch Channel}
```

| Key | Holds when |
|---|---|
| `text` | the node shows the text on its own cells: a page the screen, a section its box, an overlay its pane, a row its line, a row's column its cell, a column its header, a row's or the header's action its label, a section's child its own place |
| `not_text` | the node does not show the text |
| `rows` | a count: the collection shows that many rows; a list: it shows the rows with these keys, in this order |
| `state` | the section is `loading`, `empty`, `stale`, `failed`, `ready` or `not_loaded` |

`text` never reads a wider area than the node's own. A node the terminal does not draw on cells of
its own — a field inside an overlay, a node inside a row's `item` — fails the step naming its
path, rather than being satisfied by what a neighbour shows. A row, a cell, a column header and an
action are read whole, as the browser shows them, even where the terminal's screen cuts the value
at the column's width.

A number of four or more digits in `text` or `not_text` compares by value, not by spelling: the
terminal prints `1840`, the browser groups it by its locale (`1,840`), and either spelling in the
test matches either on the screen. A comma, a no-break space or a narrow no-break space between
groups of three digits is grouping; a full stop is not, since it is also a decimal point.

`stale` is the badge a section carries while a channel feeding it is stale; it outranks `ready` and
`empty`. `ready` means shown and fresh.

### play

Runs the clock to the next moment the fixture scripts play an event or a lifecycle beat. `with`
narrows to an event whose payload has these fields; a lifecycle beat names its channel.

```yaml
- play: {event: tickets.TicketOpened, with: {id: tk-07}}
```

```yaml
- play: {channel: metrics, lifecycle: stale}
```

A beat no script plays after the current time fails the step.

### advance

Moves the virtual clock: reads with latency answer, coalesced live updates land, `stale_after`
runs out, and every script entry now due plays.

```yaml
- advance: 500ms
```

A duration that does not fit is refused when the file is read. A move that would pass the end of
the clock, or play a looping script more than 10000 cycles in one step
(`ess_ui_test::MAX_CYCLES_PER_ADVANCE`), fails the step.

### expect_command

Holds when the command has been sent, with at least the input fields given.

```yaml
- expect_command: {command: partners.DeletePartner, input: {id: pt-003}}
```

Input values compare as JSON values with their types: `7500` holds only for the number, `'7500'`
only for the string. An object holds when every field it names holds, at any depth; a list holds
item by item. A dotted field name reaches into nested objects: `limits.cents: 7500` is
`limits: {cents: 7500}`.

A command never sent fails naming the commands that were; a command sent with other input fails
showing the input it was sent with and, for each send, the first field that differs with both
values and their types (`at website it sent string "7500", expected number 7500`).

## Fixtures

A test starts from the document's fixtures. `fixtures.views` replaces a view's data, in the shape a
fixture file holds it; views derived from it (`same_as`, `by_id_from`) follow.
`fixtures.scripts` replaces a channel's event script.

```yaml
format: ess-ui-test/1
document: ../ui.yaml
tests:
  - name: a longer list pages
    fixtures:
      views:
        partners.Page:
          total: 12
          rows:
            - {id: pt-101, name: Partner 01, tier: silver}
            - {id: pt-102, name: Partner 02, tier: silver}
      scripts:
        tickets:
          events:
            - {at: 2s, event: tickets.TicketOpened, payload: {id: tk-08, subject: Billing export, priority: urgent}}
    steps:
      - open: partners.list
      - expect: {at: pages/partners.list/sections/list, rows: 2}
```

## Time and live events

Both renderers play every channel's fixture script on one clock that starts at zero when the
application opens; anything due at zero has played before the first step. `advance` moves the
clock; `play` moves it to the moment a beat plays, which also plays everything due before it.
Looping scripts repeat. A test therefore names an event by what it is, not by when it happens,
and runs in no real time.

## The report

`--format json` prints `ess-ui-test-report/1`:

```json
{
  "format": "ess-ui-test-report/1",
  "document": "examples/partner-portal/ui.yaml",
  "tests": [
    {"name": "the search narrows the partner list", "status": "passed"},
    {
      "name": "a wrong count",
      "status": "failed",
      "step": 2,
      "message": "expect at pages/partners.list/sections/list: expected 5 rows at pages/partners.list/sections/list; it shows 6 [pt-001, pt-002, pt-003, pt-004, pt-005, pt-006]"
    }
  ]
}
```

`tests` is in file order. `step` counts from 1 and is present only on a failed test whose step
failed; a test whose fixtures do not load fails without one.

## In the terminal

`ess ui test` runs each test headless against the terminal renderer, drawn into a test backend.
Every step goes through what a reader of the terminal has: key presses and the drawn screen.

| Step | In the terminal |
|---|---|
| `select` a row | focus the section, move the cursor (`j`, `k`, `n`, `p`) until the highlighted line is the row |
| `page` | back to the first page (`p`), then `n` to the page |
| `type` | `/` and the text in a filter bar or collection; `enter`, the text, `enter` on a form field |
| `choose` | in a filter bar, move to the choice (`h`, `l`), to the option (`j`), `space`; in a form, move to the field (`k`, `j`), then `space` until it shows the option |
| `act` | the key the hint line offers for the action; the palette (`:`) for a header action; `y` or `ctrl-s` on an overlay |
| `expect` | the cells the renderer drew the node on: the screen, the section's box, the overlay, the row's line, the cell |

The terminal renderer records where each frame drew each node, by path (`ess_ui_tui::App::regions`):
the page header and its actions, section boxes and their children, the open overlay, and for a
section's or an overlay's collection a table's column headers, each row line keyed by the row's
key, each cell (not for cards, a list or a tree with an `item`), and each row action a row offers.
A region also carries the node's whole text where the screen cuts it (`Region::text`).
Rows are counted and found by those keys, so rows that show the same values are still told apart.

## One verdict in both renderers

A test file means the same in the terminal and in the browser. **Wherever the terminal refuses a
step, the Playwright spec for that test is `test.fixme` with the same reason**, at that step: the
two renderers never give one file different verdicts silently. Both read the refusals from one
place (`ess-ui-test`'s parity rules and clock), so the lists cannot drift apart. The terminal
refuses:

| Step | Refused because |
|---|---|
| any step at a column cell of a cards, list or tree collection with an `item` | the generated app renders the item there, not the cells |
| any step at a column header of a collection that is not a table | the generated app draws no header for it |
| any step inside the rows of a references list | the generated app does not address its rows |
| any step at a row action's inline confirm not under a row | the generated app draws it under the row that opened it |
| `text` or `not_text` at a node the terminal does not draw on cells of its own | its text cannot be told from its neighbours' |
| `text` or `not_text` at an icon action, or an action drawn as a choice | the browser shows no label there |
| `choose` at a form choice field with `multiple: true` | the terminal sets one value in a form's choice field |
| `choose` at a field `as: choice` without a `choice` node | it offers no options in either renderer |
| `advance` or `play` past the end of the clock | the move does not fit |
| `advance` or `play` over more than `MAX_CYCLES_PER_ADVANCE` cycles of a looping script | every cycle's events are delivered one by one |

A path that names no node is not a refusal: it fails in both renderers.

## In the browser

`--playwright <out>` writes the same tests as a Playwright spec for the project
`ess generate ui --target react` generates. Every node is `page.locator('[data-ui-path="<path>"]')`,
with row paths exactly as the test file writes them. The page clock is installed at zero before the
page loads, and `advance` and `play` become `page.clock.runFor(…)`. `page.goto` opens a page by
its path, so the spec's `baseURL` is the generated project served by `npm run dev` or
`npm run preview`, which answer every page path with `www/index.html`.

| Step | In the spec |
|---|---|
| `open` | `page.goto` of the page's route |
| `select` a row | a `click` event dispatched on the row element, then a check that it is the focused row (`ui-focused`); `test.fixme` for a collection with `expand`, which also expands the row it focuses |
| `page` | `Previous` until the pager shows page 1, then `Next` to the page, each checked on the pager |
| `type` | `pressSequentially` into the search input or the field's input |
| `choose` | the option's button, or `selectOption` for a dropdown |
| `act` | `click`; on an overlay, its primary button |
| `expect` | `toContainText`, a `RegExp` that allows digit grouping where the text holds a number of four or more digits, `toHaveCount` and `data-ui-path` of each `.ui-row`, `data-status`, `.ui-stale-badge` |
| `expect_command` | commands captured by routing `/commands/…` |

`select` does not click the row with the pointer: a click lands on the row's centre, which may be a
link or a button in a cell, and would navigate where the terminal only moves its cursor.

A test that replaces view fixtures routes exactly `/views/<view>` (not every view whose name starts
with it), and the views derived from a replaced view (`same_as`, `by_id_from`) answer from the
replacement as they do in the terminal. One that expects commands routes
`/commands/<command>`: run the app with `setDataAdapter(httpAdapter(<base>))` for these. A test that
replaces a channel script is emitted as `test.fixme`, because the generated app plays its own
scripts.

## Design notes

**Why node paths, not selectors.** A browser test usually finds an element by a data attribute, a CSS class or visible text, and
each ties the test to something the test is not about:

| Selector | What it couples the test to | What breaks it |
|---|---|---|
| A data attribute written for tests (`data-testid="delete-btn"`) | an identifier someone chose and has to keep | a renamed or duplicated id; a new component that forgets one; ids that differ per renderer |
| A CSS class or structure (`.card:nth-child(3) button.danger`) | the markup and styling | a restyle, a wrapper element, a reorder of siblings, a different component library |
| Visible text (`getByText("Delete")`) | the copy, in one language | a reworded label, a translation, two elements with the same text |

A node path is none of these. It is the node's address in the document, derived from names the
document already gives its sections, fields and actions. Nothing is added for the test's sake,
there is nothing to forget, and every renderer computes the same path: the terminal renderer from
the document, the React project as `data-ui-path`. What it removes:

- **Restyling and re-layout do not break tests.** The path holds no index and no markup; a section
  moved to another column keeps its path.
- **Rewording does not break tests.** Steps address nodes, not labels; only `expect: {text}` looks
  at copy, and only when copy is what is being tested.
- **One test, every renderer.** The same file drives the terminal headless and a browser through
  the generated spec.
- **Rows by identity, not position.** `rows/<key>` names a row by its data key, so a new row at the
  top, a sort or a live insert does not move the test onto the wrong row.
- **A broken path is found as a broken path.** A path is resolved against the document before a key
  is pressed; a renamed section fails with `no node at <path>`, not with a timeout.
- **Backend state and time are declared.** Fixtures replace views per test and the clock is
  virtual, so live updates, staleness and loading are tested without a server or real waiting.

**Limits.** The terminal runner addresses rows of a section's collection, types into filter bars,
collection filters and form fields, and chooses in filter-bar choices and form choice fields; other targets fail naming the
step. `expect text` at a node the terminal does not draw on cells of its own fails naming the path.
The browser
spec is generated and not run by `ess ui test`; its fixture and command routes assume the HTTP
data adapter.
