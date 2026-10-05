---
title: Changing selected records
sidebar_position: 4
description: "An outcome that changes, moves or deletes every record a filter selects; `moves:`, `updates:` and, from ess/23, `deletes:` take `instances:`."
---

# An outcome can change every record a filter selects

From `format: ess/16`, a `moves:` or `updates:` outcome can act on every stored record a filter
selects instead of the one an input names. Write `instances: {where: <predicate>}` in place of
`instance:`:

```yaml
- name: ended
  moves: demo.desk.Session.end
  instances: {where: team == input.team}
  emits: [demo.desk.TeamEnded]
  payload:
    demo.desk.TeamEnded: {team: input.team, ended: {count: changed}}
  sets: {note: input.note}
```

The predicate is the stored-field grammar of `when_subject:`: the entity's fields, compared with
literals or with the command's input as `input.<field>`. A `moves:` changes the selected records
resting in the transition's `from` states and skips the others; no selected record at all is an
accepted answer. `sets:` applies to every changed record and takes a literal, `input.<field>`,
`{input: …, else: …}`, `{generated: true}` or `{cleared: true}`; a source reading one record
(`{subject: …}`, `{related: …}`, `{increment: …}`) or the caller is refused by name.
`{count: changed}` fills an `Integer` payload field with the number of records the outcome changed,
and is refused anywhere but a `payload:` field of such an outcome. `instance:` beside `instances:`,
and `instances:` on `creates:` or `preserves:`, are refused. A set outcome accepts, and is selected
by `when:` or as the default.

An outcome with one existing subject can also change other records, with `affects:`:

```yaml
- name: invited
  updates: demo.desk.Session
  instance: session_id
  emits: [demo.desk.Invited]
  payload:
    demo.desk.Invited: {session_id: input.session_id}
  sets: {on_hold: false}
  affects:
    - entity: demo.desk.Session
      where: team == subject.team
      sets: {on_hold: true}
```

Each entry changes every record of `entity` its `where:` selects. `where:` reads that entity's
fields, `input.<field>` and `subject.<field>` — the subject as it was before the outcome. Where
`entity` is the subject's own, the subject itself is not among the records. `sets:` takes the
sources `instances:` does. `affects:` sits beside `moves:`, `updates:` or, from `ess/23`, `deletes:`
with `instance:`, never beside `instances:`.

From `ess/22` an entry may also move the records it selects (beyond10x/ess#229): deactivating a
user ends that user's live sessions.

```yaml
- name: deactivated
  moves: demo.users.User.deactivate
  instance: user_id
  emits: [demo.users.UserDeactivated]
  payload:
    demo.users.UserDeactivated: {user_id: input.user_id}
  affects:
    - entity: demo.users.Session
      where: user_id == subject.user_id
      moves: demo.users.Session.end
      sets: {revoked: true}
```

`moves:` names a transition of the entry's own `entity`, written `<Entity>.<transition>`, and
`sets:` beside it is optional. A selected record resting in one of the transition's `from` states
takes it and comes to hold what `sets:` writes; one resting elsewhere is left as it is, as under
`instances:`. The move counts as the transition's cause, but a state only it reaches is not one the
suite can arrange for another scenario. Below `ess/22` the move is refused, naming `ess/22`. One
outcome moves the records of one entity at most once: a second entry with `moves:` over the same
entity is refused. Entries that only set fields may sit beside it, and apply in the order written.
A record whose filter reads an `Optional<…>` field holding nothing is not selected.

The suite arranges, for each, three records the filter selects, one record per conjunct of the
filter that fails only that conjunct (or one failing the whole filter), and — for a `moves:` — one
it selects resting outside the transition's `from` states; runs the command; and reads every record
back from an immediate, unfiltered view that publishes the identity, the state and every field the
effect writes: the changed ones in their new state with what `sets:` wrote, the others as they
were, and `{count: changed}` equal to the records changed. It then sends the command again with an
input the filter selects no record by, and requires the same outcome, a count of 0 and no record
changed. Where no such view exists the scenario is refused by name. A `sets:` entry writing the
entity's identity is refused. No new
suite step is used. Every generated code target (Rust, Go, Web, Clap) refuses both constructs by
name, and Entity Runtime lowering refuses them with `SetEffectUnsupported`. Below `ess/16` both are
refused with `unsupported_format_version`. Atomicity, partial failure and the order in which records
change are not part of either.

## Delete every record a filter selects

From `format: ess/23` (beyond10x/ess#452), `deletes:` takes `instances:` as `moves:` and `updates:`
do, and removes every stored record the filter selects:

```yaml
- name: revoked
  deletes: demo.auth.Token
  instances: {where: {all: [user_id == input.user_id, scope == input.scope]}}
  emits: [demo.auth.TokensRevoked]
  payload:
    demo.auth.TokensRevoked: {user_id: input.user_id, revoked: {count: changed}}
```

No selected record at all is an accepted answer, and `{count: changed}` is the number of records
removed. `sets:` beside it is refused, as on a single `deletes:`: a removed record holds nothing to
set.

An `affects:` entry may delete too, written `deletes: <Entity>` naming the entry's own `entity`, and
`affects:` sits beside a `deletes:` subject, which `subject.<field>` reads as it was before the
outcome: deleting a user removes its tokens.

```yaml
- name: deleted
  deletes: demo.auth.User
  instance: user_id
  emits: [demo.auth.UserDeleted]
  payload:
    demo.auth.UserDeleted: {user_id: input.user_id}
  affects:
    - entity: demo.auth.Token
      where: user_id == subject.user_id
      deletes: demo.auth.Token
```

Another entity than the entry's, `sets:` or `moves:` in a deleting entry, a deleting entry beside
any other entry over the same entity, and a deleting entry over an entity another domain owns are
refused; over different entities of the outcome's domain they may sit side by side. The suite reads every removed record absent from each immediate view of its entity and
every other record as arranged; for `instances:` it then sends the command again selecting nothing,
and requires a count of 0. Below `ess/23` each form keeps the refusal it had, naming `ess/23`.

## Removal in other domains is one binding per domain

A deleting `affects:` entry stays inside the domain of its outcome: one over an entity another
domain owns is refused, naming this idiom. To remove records another domain owns when a
record here is deleted, the deleting command emits one event, and each receiving domain gets one
binding on it: one binding per receiving domain, each with its own `delivery:` and `on_failure:`
([bindings](bindings-and-components.md)), invoking a command of that domain that deletes its own
records with `instances:`. ESS states no order between bindings of one event, so no receiving
domain may rely on another having run first; there is no order between bindings to declare. A
failure in one is that binding's `on_failure:` to answer, and atomicity across domains is not part
of the specification.

