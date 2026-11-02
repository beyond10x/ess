---
title: "0.83 — Optional list appends, binding refusals that check event identities, and refusals that tell states apart"
description: >
  0.83.0 adds no format: the interpreted target appends null for an absent optional list element,
  binding refusal scenarios compare their trigger event's identities, a row-set guard beside named
  rows is witnessed on every refusal order, and no-view input refusals tell the record's state apart.
slug: optional-appends-and-refusals-that-tell-states-apart
tags: [release, ess]
date: 2026-11-02T14:00:00+01:00
release_tag: "0.83.0"
---

0.83.0 adds no source, scenario or suite format, and every change in it is a fix. One is in the
interpreted target, and the others are in the conformance suites synthesis writes. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change; this note
covers each of them.

{/* truncate */}

## Appending an absent optional element

- The interpreted target appends `null` for an absent or `null` element of
  `{append: input.<field>}` over a `List<Optional<T>>`, as generated Rust and Go already append
  `None` and `nil`. Synthesis expects that `null` in the view row.
- Before, the interpreted target refused the command as unsupported.
- An element over a list of required elements is unchanged.

## Binding refusal scenarios compare event identities

- A synthesized binding refusal scenario covers a refusal the binding's policy drops, escalates or
  retries a bounded number of times. It now compares the identity fields of the trigger's event with
  the instances its invocation was sent, as the other binding scenarios do since 0.82.0.
- It also compares them with each id the invocation was sent as a literal for a field of an
  entity's identity type, such as the id a trigger naming no related row is sent. That holds where
  it sends another publisher of the event, or a trigger naming a row arranged so the predicate
  refuses it.
- Before, it expected that event by shape alone, so a target publishing a wrong id there passed.
- The suite bytes of seven fixture models move; the changelog names them.

## A row set beside named rows, on every refusal order

- A command guarding on a row set beside rows its input names (`ess/32`) is witnessed on every
  refusal order.
- A refusal over a named row that also carries a `when:` is held beside the row set's rows with a
  send that holds that `when:`. Where the `when:` equates two inputs, that send names one row twice.
  Where no such send stands, the refusal is witnessed on its related row alone, so its row-set
  branch is no longer refused.
- A row-set refusal is held beside a later refusal whose `when:` compares the named rows. Where only
  a creator sent one row twice lays the selected row, that is how it is laid.
- A `forall` on the accepting branch is held on a selected row the `forall` passes, and its default
  refusal on a selected row naming the send's own rows. A target refusing on any selected row now
  fails, at `ess/31` too.

## No-view input refusals tell the record's state apart

- These are input refusals on a command that addresses an existing record, on a model where no
  identity view shows the row.
- The arranged half of such a refusal tells the arranged state apart from every other state. Where
  the first branch addressing the record from that state also runs from other states, the scenario
  sends the shortest sequence a bounded search finds that does tell them apart. Where the bound
  stops the search first, that sequence is extended one send at a time past it. It is drawn from
  the command's branches and from other commands whose input names only that record, with a
  `wrong_state:` refusal among the answers. A free-text input beside the identity names no record.
- A refusal beside held-state branches reads the record back after each held-state send, through
  sends from that state that tell it apart from every other state. The command's own branches come
  first, its held-state refusals among them.
- A target that moves the record on refusal and publishes nothing now fails both. Before, they
  required only the error and no event, and such a target passed.
- Where the sends tried leave states untold, the scenario is withdrawn and refused as
  `ESS-SYNTH-001`, naming the states left. Where no send has a valid input, it names every branch
  tried.
- A model with an identity view keeps its suite bytes. The test model `occurrence-clock-offers.yaml`,
  which has none, moves its suite bytes: its `AcceptOffer/outcome/malformed` scenario reads the
  offer back in each of its three states, with its scenario count and refusals unchanged.
