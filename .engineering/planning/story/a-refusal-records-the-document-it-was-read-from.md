---
format: aep.planning-md/2
id: story:a-refusal-records-the-document-it-was-read-from
kind: story
status: draft
title: A refusal records the document it was read from
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
revision: 3
---
# A refusal records the document it was read from

`ValidationError` (`crates/specify/ess-primitives/src/error.rs:663`) carries a code, a location
string, a message, a hint and an optional site. It does **not** carry the document the refusal was
read from. `Locator` (`crates/specify/ess-compiler/src/resolve.rs:427`) recovers the file by
searching the whole specification for the declaration's name, and reports `<document>` — the whole
specification, not a file — whenever that search is not unique.

`story:enum-variant-in-an-entity-invariant`'s acceptance clause 1 asks for "a stable refusal code,
the file it was read from, and the variants that are declared". The first and third are structural.
The second is a name search, and the search fails on the plainest case the compiler is built to
reach: one entity declared in two files, which ESS itself refuses with `DuplicateDeclaration`.
`whole_name` does not help — both occurrences are whole names.

Measured by the wave-24 adversary at `adversary_locator_pass1.rs:252`, exit 101: all three refusals
in that run cited `("<document>", None)`.

## Acceptance

A refusal produced while reading a multi-file specification names the file its declaration was read
from, without searching for the name, including when the same name is declared in two files.

## Scope

- `crates/specify/ess-primitives/src/error.rs` — `cited`
- `crates/specify/ess-compiler/src/resolve.rs` — `cited`
- every construction site of `ValidationError` — `inferred`, count not established

Blast radius is larger than one package. Not a wave candidate until it is scoped.
