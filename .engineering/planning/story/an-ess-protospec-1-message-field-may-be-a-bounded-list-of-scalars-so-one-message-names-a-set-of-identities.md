---
format: aep.planning-md/3
id: story:an-ess-protospec-1-message-field-may-be-a-bounded-list-of-scalars-so-one-message-names-a-set-of-identities
kind: story
status: draft
title: An ess-protospec/1 message field may be a bounded list of scalars, so one message names a set of identities
summary: A protocol message field is a String, Boolean or Integer only, so one message cannot name a set of identities (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

One protocol message names a set of identities: a broadcast to the members `[a, b, c]`, a
subscription to several topics, a batch acknowledgement. The field is a list of text values, and a
transition may react to its length or to a member.

Reproduction (installed `ess`, a copy of the committed `lost-ack` example with `{name: members, type: "List<String>"}`
on one message):

```console
$ ess specify protocol validate --path list.yaml
error: messages.ack.members [UnsupportedProtocolType]: List<String>: ess-protospec/1 supports String, Boolean and Integer only
```

The field type is `Field { type_ref: TypeRef }`, and "this
experimental profile admits String, Boolean and Integer only" (`crates/specify/ess-domain/src/protocol.rs:68-71`,
`admits` at 336). Runtime values are `Node::Text`, `Bool` and `Number` only.

The model workaround is one message per identity (a member is a `String` field), which changes the
protocol: the send is no longer one message, so ordering, loss and capacity are checked for another
protocol.

## Outcome

A message field may be `List<String>`, `List<Integer>` or `List<Boolean>`. A transition guard may test
that the list is empty, that it has a member equal to a literal or to another field, and read its
length; a `set` effect may copy a list field. Values are ordered and finite (the document's `max`
length is a bound, `bounds.max_list_len`).

## Acceptance

- A document with a `List<String>` field validates and compiles; a nested list, a `List` of a
  non-scalar type, a `value:` that is not a list, and a missing `bounds.max_list_len` each refuse with a
  string code and its path.
- Exploration branches over the lists a delivered message may carry (the bound is `max_list_len`) and a
  transition guarded on a member is enabled for exactly the lists that hold it.
- A recorded run that delivers `members: ["a","b"]` takes the same transition as a model delivery of
  that list; a list longer than `max_list_len` fails the monitor.
- The committed examples keep their digests and verdicts (a new key and a new type are additive in
  `/1` under PR-D3 A).
- Each test runs within 60 s and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Field` 68-71, `admits` 336), `crates/specify/ess-compiler/src/protocol.rs`,
  `crates/verify/ess-conformance/src/protocol/engine.rs`, `monitor.rs`, `docs/design/protocol-specification.md`.
- Size: M, about 400 changed lines. Edits the files of the other open `ess-protospec/1` stories; one at a time.

## Fit review

1. **Need.** A message that carries a set of identities as one send. The reporter's syntax is the
   `List` field type (theirs, and the one ESS already spells for entity fields).
2. **Class: gap** within the experimental profile.
3. **Already expressible?** One message per member, which changes ordering and capacity (fix-by-idiom
   fails).
4. **Fit.** Reuses ESS's `List` type vocabulary; the profile's restriction is stated in the code and in the
   design page as experimental. Coverage: validate, compile, explore, monitor, target run.
5. **Second adopter.** A pub/sub model whose subscribe message names topics.
6. **Cost.** One type constructor, one bound, about four codes. Format per PR-D3.
7. **Alternatives.** Change nothing; a `Set` type (no gain, a list is ordered and the check dedups
   nothing); a `Json` field (refused by the profile's determinism rule).

## Decisions

AC9-D5: how a list value enters exploration. A: the lists up to `max_list_len` over the values a guard or another message names, plus the empty list. B: every list over a declared alphabet (may exhaust `max_states`). C: decline, one message per identity. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as redesigned (a bound is part of the field).
