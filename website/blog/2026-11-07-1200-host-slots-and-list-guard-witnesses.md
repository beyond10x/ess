---
title: "0.90 — UI documents declare host slots, and synthesis witnesses guards over lists of identities"
description: >
  0.90.0 lets an ess-ui/1 document declare host slots, the places the host application draws with
  its own component, and checks them. Conformance synthesis witnesses a guard over a list of
  identities beside an updates subject, in a creating command and over several literals, and a
  null entry of a tags field draws no chip.
slug: host-slots-and-list-guard-witnesses
tags: [release, ess]
date: 2026-11-07T12:00:00+01:00
release_tag: "0.90.0"
---

0.90.0 changes the `ess-ui/1` document and its checker, conformance synthesis and the tags field
of the React and terminal renderers. No source format moves, and `ess-ui/1` stays `/1`: a host slot
is an optional node, so a document without one loads, checks and renders as before. The synthesis
fixes close the gaps [#761](https://github.com/beyond10x/ess/issues/761) describes. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change.

{/* truncate */}

## Before you upgrade

- An `ess` before 0.90.0 refuses a document that declares a host slot, naming `host` as an unknown
  key.
- `ess generate ui --target react` and the terminal renderer refuse a document holding a host slot,
  naming its path, until host slots are generated in 0.91.0. Declare slots now to check them;
  generate from such a document once 0.91.0 is out.

## Host slots

- A node carrying `host` is a host slot: a place the host application draws with its own component
  and ESS does not draw. It names the component the host registers (`host`), why the generator
  cannot draw it (`reason`), what it stands in for (`stands_in_for`), and optionally the view and
  commands the component uses (`reads`, `commands`).
- A slot stands at a page, a section, an overlay, a tab or a field; its node path is its scope.
- `ess ui check` refuses a slot without a reason or without `stands_in_for`, two slots of one host
  name, and a slot beside `component` or `primitive`. With `--hosts <file>` it refuses a registered
  name no slot declares.
- `stands_in_for` is a composite, a primitive or `none`. The slots standing in for `none` are
  listed as the generator's backlog, in the text report and in the `ess-ui-check/1` JSON report.

## Guards over lists of identities

- A refusal over a list of identities beside a branch acting on a row the input names is sent for
  a row that branch arranges, so the not-found answer no longer decides its scenario first.
- A branch whose own guard the empty list refutes is sent the list its witness holds, each element
  a row created under it. Where that branch is the only route to a row, it is sent a list that
  satisfies the guard instead of being refused.
- A guard that needs several literals in one list, `all: [exists a: …, exists b: …]`, is witnessed
  by one list holding every literal; before, it was refused `ESS-SYNTH-003`.
- Known limit: the negated side of such a guard, `not exists b: …`, is not yet tested, so a target
  that ignores the negation passes the suite. A story tracks it.

## Tags fields

- A null entry of an `as: tags` field draws no chip in the React target or the terminal, and the
  tags input neither shows, compares nor writes back a held null.
