---
format: aep.planning-md/3
id: story:via-and-instance-may-name-a-member-of-a-struct-input
kind: story
status: draft
title: 'A when_related via: and an instance: may name a member of a struct input, as sets: already may'
summary: 'via: and instance: accept only top-level inputs (ESS-COMMAND-001) (adopter item 3, beyond10x/a2a); gap, needs ess/31.'
tags:
- adopter-report
- ess-0.71.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 3 of 9, 2026-10-22). A
command whose identifying reference sits inside a struct input (`message.taskId`) cannot name it
in `when_related: {via: …}` or in `instance:`. Validation refuses with `ESS-COMMAND-001`
(`COMMAND_UNDECLARED_REFERENCE`, `crates/specify/ess-compiler/src/resolve.rs:590`), message format
from `crates/specify/ess-domain/src/command/related_guard.rs:1658-1670`:

```text
`{via}` is not an input of `{command}`   (hint: declared input: …)
```

(Format string quoted from the code; the adopter's rendered line was not captured.)

Minimal reproduction (brand-free): `Reply(message: Message)` where `Message { thread_id: Ref<Thread>,
text }`, with `when_related: {via: message.thread_id, …}` or `instance: message.thread_id`.

## Where it lives

- `crates/specify/ess-domain/src/command/related_guard.rs:307-308` keeps `via` as one string;
  refusal at `:1658-1670`; `crates/specify/ess-domain/src/command.rs:2784` (`input_field`) looks up
  top-level inputs only.
- `crates/specify/ess-compiler/src/resolve.rs:3538` (related_read), `:4077`, `:4153-4177`
  (instance refusal).
- `sets:` already resolves nested paths through `crates/specify/ess-domain/src/command/input_path.rs`
  (ess/22, `story:feature-request-233`, `docs/design/expression-family-source22.md:369-414`).

## 0.68.0

No.

## Cause

Not a defect: the language has no input path in these two positions. The nearest story,
`story:related-guard-on-a-captured-struct-input-member` (implemented), covers a predicate reading a
member, not `via:` or `instance:`.

## Fit review

1. **Need.** A command must be able to address the row it acts on, or a related row, through a
   reference nested in a struct input. Requester's wording: "when_related.via and instance: accept
   only top-level inputs; nested message.taskId".
2. **Class: gap.** The domain fact (the reference lives in a sub-structure of the request) cannot
   be stated; flattening the input changes the wire shape a published protocol fixes.
3. **Already expressible?** Only by a duplicate top-level input, which changes the wire and is not
   checked against the nested value. Not an idiom under policy 1
   (`docs/design/generic-language-features.md` § Policy).
4. **Fit.** Grid R row "input member readable in position X" (`docs/design/language-grids.md`):
   `sets:` and guard predicates already read an input path; `via:` and `instance:` are the two
   cells of the row still closed. Open the row: both take the same `input_path` syntax and
   resolution `sets:` uses, so one spelling everywhere. Every target reads the resolved reference,
   not the path, so Rust, Go, TypeScript, the interpreter and synthesis see a resolved field
   (inferred; the implementor confirms each lowers from the resolved form or refuses by name).
5. **Second adopter.** A payments API whose `Capture(request: {payment_id, amount})` guards on
   the payment's state through `request.payment_id`.
6. **Cost.** A new format version: `via:`/`instance:` accepting a path is a new meaning under an
   existing key, so it lands in ess/31 (the next format) and older headers refuse it as
   `unsupported_format_version`. No new keyword, no new diagnostic code; `ess verify diff`
   classification of a moved via path to be checked.
7. **Alternatives.** (a) Change nothing: adopters duplicate the reference at top level. (b) A new
   key such as `via_path:`: two spellings for one idea, refused as a red flag. (c) Chosen: the
   existing keys accept the `sets:` input path.

## Decisions

**accept, redesigned.** Not a new key: `via:` and `instance:` accept the input path `sets:`
already accepts, under the next format version (ess/31).

## Acceptance

- The minimal reproduction validates and compiles under ess/31, and its conformance suite
  exercises the guard and the addressed instance through the nested reference.
- The same document under ess/30 is refused as `unsupported_format_version` naming ess/31.
- A path naming a non-reference member, or a member that does not exist, is refused by name.

## Schedule

Gap; not tagged for 0.69.0. Needs a new format version (ess/31).

## Format retarget (2026-10-28)

`ess/31` shipped in 0.69.0 and `ess/32` in 0.70.0 without this story (`FORMAT_HISTORY` in
`crates/specify/ess-domain/src/system.rs`). Its planned format gate is now the next one, `ess/33`,
opened once for the open-issue batch that carries the `ess/33` language changes
(`docs/plan/wave-ess-issues-1.md` § Batches). Read `ess/31` above as `ess/33`, and `ess/30` as
"below `ess/33`". Store drift fixed under D136 (`approval-record:d136-synthesis-defects-and-store-drift`).
