---
format: aep.planning-md/3
id: story:via-and-instance-may-name-a-member-of-a-struct-input
kind: story
status: draft
title: via:, all:, instance:, each.in and {missing:} accept an input path into a struct input, a List<Id> member included
summary: 'via: and instance: accept only top-level inputs (ESS-COMMAND-001) (adopter item 3, beyond10x/a2a); gap, needs ess/31.'
tags:
- adopter-report
- ess-0.71.0
- feature-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#640
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 7
---
## Need

A command whose reference sits inside a struct input cannot name it where a row is addressed. Four
positions are affected:

- `when_related: {via: …}`, including a `List<Id>` member and the per-element existence of
  beyond10x/ess#558;
- `instance:`;
- `affects: [{each: {in: …}}]`;
- the `{missing: …}` payload source.

`sets:` and `payload:` read `input.<path>` from `ess/22`. Flattening the input changes a wire shape
a published protocol fixes.

Minimal reproduction (0.92.0, `ess/36`). `example.g.Settings {parents: List<example.g.ParentId>,
primary: example.g.ParentId, entries: List<example.g.Entry>}`, and command
`example.g.Make(settings: example.g.Settings)`:

```yaml
      - name: unknown-parent
        when_related: {via: input.settings.parents, exists: false}
        error: example.g.ParentNotFound
      - name: made
        creates: example.g.Child
        instance: child_id
        sets: {parents: input.settings.parents}
```

```text
error[ESS-COMMAND-001]: `settings.parents` is not an input of `example.g.Make`
  help: declared input: `settings`
```

The same holds for a scalar member, `via: input.settings.primary`. `instance: settings.primary` on an
`updates:` branch gives ESS-COMMAND-001, ``acts on the instance named by `settings.primary`, which is
no input field of it``. `each: {in: input.settings.entries, as: e}` gives ESS-COMMAND-009, ``each:
walks a list of the command's input, written `input.<field>` ``. The top-level control
`via: input.parents` validates, synthesizes 3 scenarios and passes on the interpreted target.

## Acceptance

At the new source format, `ess specify validate` exits 0 on each of the following, and on
`all: [{via: input.settings.primary, predicate: …}, …]`:

- `via: input.settings.parents, exists: false`, where the list member has the per-element meaning
  of `ess/28`;
- `via: input.settings.primary` with `exists: false` and with `predicate:`;
- `instance: settings.primary` on `updates:`, `moves:`, `deletes:` and the `unknown_instance:` /
  `existing_instance:` branches that answer it;
- `each: {in: input.settings.entries, as: e}`;
- `{missing: input.settings.parents}` in an error payload.

Refusals and compatibility:

- Below the new format, each of these is refused as `unsupported_format_version` naming it, instead
  of ESS-COMMAND-001 or ESS-COMMAND-009.
- A path to a member that does not exist is refused as `undeclared_reference` naming the struct's
  members.
- A member that is no entity identity (or a list of one), and a path crossing a list or a union, are
  each refused as `type_mismatch` naming the member's type.
- A path crossing an `Optional` member reads as an Optional reference: `ess/22`'s "checked only when
  present" rule applies, and `instance:` over it is refused as today for an Optional input.
- The compiled IR of a model that uses no input path keeps its bytes.
- `docs/design/expression-family-source22.md` (input paths) and `website/docs/reference/predicates.md`
  list the new positions.

## Scope

- cited: crates/specify/ess-domain/src/command/related_guard.rs
- cited: crates/specify/ess-domain/src/command/input_path.rs
- cited: crates/specify/ess-domain/src/command/set_effects.rs
- cited: crates/specify/ess-domain/src/entity.rs
- cited: crates/specify/ess-domain/src/command.rs
- cited: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/specify/ess-domain/src/command/related_list.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-domain/tests/input_path_addressing.rs
- inferred: docs/design/expression-family-source22.md
- inferred: website/docs/reference/predicates.md
confidence: medium

## Fit review

1. **Need.** A command must address the row it acts on, a related row or a list of rows through a
   reference nested in a struct input. Minimal reproduction above (triage run: `list-member`,
   `scalar-member`, `instance-member` and `each-member` exit 1; `top-level-list` exits 0). The
   requester's proposed syntax: `via: input.settings.parents`. An earlier report on the same story
   proposed `instance: message.thread_id`.
2. **Class.** Gap. Each of these refuses a member path:
   - the related-row check looks up a top-level input only
     (`crates/specify/ess-domain/src/command/related_guard.rs:1665-1678`,
     `crates/specify/ess-domain/src/command.rs:2891`, `input_field`);
   - so does the instance check (`crates/specify/ess-domain/src/entity.rs:1493`);
   - the `each:` check refuses anything but `input.<field>`
     (`crates/specify/ess-domain/src/command/set_effects.rs:980`);
   - the IR's `ResolvedRelatedVia::Input` holds one field name
     (`crates/specify/ess-compiler/src/ir.rs:1845-1853`).
3. **Already expressible?** Only by a duplicate top-level input. That changes the wire, and nothing
   checks it against the nested value. It is no idiom under policy 1
   (`docs/design/generic-language-features.md:588-592`).
4. **Fit.**
   - Grid R input column: the target admits the input "in every command position"
     (`docs/design/language-grids.md:46`), and G1 spells every input read `input.f`
     (`docs/design/generic-language-features.md:146-154`).
   - The path rule exists (`crates/specify/ess-domain/src/command/input_path.rs:1-11`: declared
     struct members, through `Optional` and newtype; lists, maps and unions stop a path).
   - The closing rule opens every addressing position at once (`via:`, `all:` elements,
     `instance:`, `each.in`, `{missing:}`), not `via:` alone (`generic-language-features.md:211-216`).
   - The list member reuses `ess/28`'s per-element meaning (`generic-language-features.md` § G2,
     "The same format quantifies over a list of references").
   - No new key: the existing keys take the existing path syntax.
5. **Second adopter.** A payment API, `Capture(request: {payment_id, amount})`, guarding on the
   payment's state through `request.payment_id`. A message API, `Reply(message: {thread_id, text})`,
   addressing the thread.
6. **Cost.**
   - One source format, the next one open (`ess/37` if no other decision allocates it first). An
     earlier draft of this story named `ess/31` and then `ess/33`, both shipped without it.
   - No new keyword and no new diagnostic code.
   - An IR change to carry a path in `ResolvedRelatedVia::Input` and in the instance source, left
     out for one segment so that existing IR keeps its bytes.
   - `ess verify diff` reports a moved path as the existing `outcome-condition-changed`.
7. **Alternatives.** (a) Change nothing: adopters duplicate the reference at top level, which
   policy 1 rejects. (b) A new key, `via_path:`: two spellings for one idea, refused as a red flag.
   (c) Open `via:` only, as the issue asks: refused by the closing rule. (d) The existing keys
   accept the `sets:` input path in every addressing position: chosen.

## Decisions

accept, redesigned. `via:`, `all:` elements, `instance:`, `each: {in:}` and `{missing:}` accept the
`input.<path>` that `sets:` accepts, a `List<Id>` member included, under the next source format. The
request named `via:` alone; the closing rule widens it to the whole row. Follow-ups:

- `story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member`
- `story:generated-targets-read-a-reference-nested-in-a-struct-input`

## Source

github:beyond10x/ess#640
