---
format: aep.planning-md/3
id: story:an-action-a-confirm-and-a-form-name-the-notice-text-of-each-outcome-of-their-command-with-the-row-s-fields-at-hand
kind: story
status: draft
title: An action, a confirm and a form name the notice text of each outcome of their command, with the row's fields at hand
summary: The generated runtime writes the command's qualified name as the notice of every outcome; ess-ui/1 has no key for the text of a notice (downstream report 2026-11-26, need 1 of 2)
tags:
- ess-ui
- external-request
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:a-ui-expression-joins-texts
- informed_by: story:ui-binding-contract
revision: 1
---
## Need

A downstream application reported on 0.99.2 (received 2026-11-26) that a generated UI cannot say, in words of the document, what happened when a command ran. Each outcome of a command ends in a notice, and the generated runtime writes the command's qualified name into it: `<command> done`, `<command> sent`, `<command> failed: <reason>`, and in fixture mode `<command> done in fixture mode; its effect was not applied: <reason>`. The report counts 28 of 44 scenario runs of its test document that show a qualified command name to a user.

Facts, read on `main` at `c0b5f560cb`:

- React: `runCommand` writes the notices in `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl` (lines 96-150): `${command} sent`, `${command} done`, the fixture line, `${command}: ${error.display}` after a committed unfinished answer, and `${command} failed: ${message}`.
- Terminal: the same words at `crates/ui/ess-ui-tui/src/app.rs:4364` and `crates/ui/ess-ui-tui/src/http.rs:480`.
- `Action`, `Confirm` and `Form` (`crates/ui/ess-ui/src/model.rs`: `Form` 1673, `Confirm` 2085, `Action` 3215) hold no key for a notice. `Form.result` shows the command's outcome as a node, which is a place on the page and not the notice.
- A typed refusal is shown on a form as the command's error text (`story:ui-binding-contract`); the notice for a refusal of an action or a confirm is still the status text with the qualified name.
- No store story names a notice key. `story:fixture-mode-applies-a-confirmed-commands-effect` records the fixture notice as it is.

Minimal reproduction (fresh, not run here): a document with one entity `Request`, a command `ApproveRequest` with outcomes `approved` and `not-pending`, a page with a `collection` over a view of requests and the row action `{name: approve, does: desk.requests.ApproveRequest, label: Approve}`. After the action on row `r1` the notice reads `desk.requests.ApproveRequest done`, and after the refusal `desk.requests.ApproveRequest failed: ...`.

The requester's proposal, restated and labelled theirs: a key per outcome on the action, the confirm and the form, naming the notice text, with the row's fields available to it.

## Outcome

An `ess-ui/1` action, confirm or form names the notice text of each outcome of its command, and the text can read the row's fields (and the command's input). An outcome with no text keeps today's notice, so a document without the key draws and generates the same bytes. Both renderers write the same text for the same outcome, and `ess ui test` reads it the way a person sees it.

## Acceptance

1. Red first: `ess ui check` accepts the key beside `does:` on an action, a confirm and a form, and with `--model` refuses, by node path, an outcome name the command does not declare and a text that reads a path outside the row, the input and the state.
2. After the action of the reproduction on row `r1`, the notice reads the text written for the outcome `approved` with the row's fields filled in, in the generated React project (a test in `crates/ui/ess-ui-react/tests/`) and in the terminal (a test in `crates/ui/ess-ui-tui/tests/`). For the refusal `not-pending` it reads the text written for that outcome.
3. An outcome without a text writes the notice as before, byte for byte, in both renderers; a document without the key generates a React project identical to the one before.
4. Fixture mode: the line "done in fixture mode; its effect was not applied" stays a separate, fixed notice, so a person never reads a success text for an effect that did not happen (decision NW-D3).
5. An `ess ui test` `text` check sees the written notice, as it sees the notice today (`website/docs/reference/ess-ui-test.md:347` names the failed-command notice as shown text).
6. The `ess-ui/1` reference (schema and `website/docs/reference/ess-ui.md`) documents the key, its outcome names and the fields at hand.
7. Every new test runs in 60 s or less, the step in 120 s or less (`task test-xtask` and the reference regeneration included).

## Fit review

Read-only, against `origin/main` at `c0b5f560cb`.

1. **Need.** A document says, per outcome, what the user is told after a command ran: success, each refusal, with the row's values in the sentence. Reproduction above. The proposed syntax is the requester's and is one option below.
2. **Class.** *Gap*. The text of a notice is not expressible at all: no key exists (`model.rs`, `Action` 3215, `Confirm` 2085, `Form` 1673), and the generated runtime owns the words (`actions.tsx.tmpl:96-150`). A host slot is no answer: no check reaches it.
3. **Already expressible?** No. `Form.result` draws the outcome as a node on the page, which a confirm and a row action do not have; `label` and `confirm_label` name a button; `confirm.body` explains before the command and reads no outcome.
4. **Fit.** No grid applies (`docs/design/language-grids.md` has no cell for UI text). Vocabulary: the outcome names are the ones the command declares, as `ess ui check --model` already resolves outcomes of a command (`crates/ui/ess-ui-check/src/model.rs:2089-2165`). The text reads the row, as `bind` and `copy` read it. A text built from parts needs `story:a-ui-expression-joins-texts` (draft) or a template form; this story states which it takes in NW-D1. Siblings: action, confirm and form all run commands, so the key is one shape on all three, not on one of them. Targets: React and the terminal draw it; `ess ui check`, generated documentation and the schema carry it; a target that cannot draw it refuses it by node path.
5. **Second adopter.** A ticket desk shows `Ticket 4711 closed` after the close action and `Ticket 4711 is already closed` after its refusal, instead of two lines with the command's qualified name.
6. **Cost.** One optional key in `ess-ui/1`, additive: a document without it generates the same bytes, and an older `ess` refuses the key by name (the precedent is `unit` gaining values within `ess-ui/1`). New findings for an unknown outcome name and for a path the text may not read. No ESS source format, no migration.
7. **Considered.** (a) Change nothing: every notice names a command. (b) A host slot that draws the notice: no check reaches it, and the text leaves the document. (c) The requester's key per outcome with the row at hand, on the three nodes (recommended below). (d) One text for every outcome: it cannot tell a success from a refusal.

## Decisions

Accept, redesigned in the details below; the need is taken, the spelling is open.

- **NW-D1, spelling.** A: one key `notice` on the action, the confirm and the form, a map from an outcome name to a text; the text reads `row.<field>`, `input.<field>` and `state.<key>` by the grammar the text expressions already use (recommended: reuses outcome names and the existing roots, one shape on the three nodes). Cost: a text built from parts needs `story:a-ui-expression-joins-texts`, so the first unit reads one value per text or waits for that story. B: one key per outcome class (`on_done`, `on_refused`, `on_failed`) holding a text. Cost: three keys and a class the model does not name; a command with two refusals gets one text for both. C: a key on the command in the model (`ess/<n>`) so every renderer and every node uses the same words. Cost: a model format key, the widest change, and a UI word inside the specification.
- **NW-D2, an outcome the text does not list.** A: it keeps today's notice (recommended: nothing changes for a document that adds one text). B: it is refused by `ess ui check --model` until every outcome has a text. Cost: a longer document and a new refusal for every command change.
- **NW-D3, fixture mode.** A: the fixture notice stays fixed and says the effect was not applied (recommended: it is a statement about the runner, not about the command). B: the written success text is used and a second line says fixture mode. Cost: a user reads success for an effect that did not happen.
- Recommended: A, A, A. Not architecture and not a security boundary; taken on default if unanswered.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`Action`, `Confirm`, `Form`), `schemas/ui/ess-ui.schema.yaml`
- cited: `crates/ui/ess-ui-check/src/model.rs` (outcome names of a command), `crates/ui/ess-ui-check/src/rules.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl` (96-150), `crates/ui/ess-ui-react/src/emit.rs`
- cited: `crates/ui/ess-ui-tui/src/app.rs` (4364), `crates/ui/ess-ui-tui/src/http.rs` (480)
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- tests: `crates/ui/ess-ui-react/tests/`, `crates/ui/ess-ui-tui/tests/`, `crates/ui/ess-ui-check/tests/`

## Source

A downstream application, 2026-11-26, 0.99.2, a generated React app (need 1 of two UI needs).
