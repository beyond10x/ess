# An update that writes the identity re-keys the record (`ess/23`)

Status: implemented (beyond10x/ess#429, `story:feature-request-429`). Source format `ess/23`, which
this story introduces. No new keyword, no suite format, no delta format.

## The gap and the defect

A command addresses a stored record by its identity and leaves it answering to a new one, every
other field carried over: a secret renamed, a file moved, a username changed. Before `ess/23` the
construct that says so already parsed and validated —

```yaml
- name: renamed
  updates: demo.vault.Secret
  instance: name
  sets: {name: input.new_name}
```

— and the synthesized suite then required the opposite of a rename: the row under the *old*
identity, holding the new one in no column a view reads. A target that renamed failed it and a
target that ignored the write passed it. Nothing could say that the old identity is gone.

## The meaning

From `ess/23`, an `updates:` with `instance:` whose `sets:` writes the entity's identity **re-keys
the record**:

- the row is read under the identity `instance:` names, as for every update;
- it comes to rest under the identity `sets:` writes, holding every field `sets:` does not name as
  it held it before;
- the old identity names nothing afterwards: a view shows no row under it, and the same request
  sent again takes the command's unknown-instance answer.

Guards read the row as it was before the outcome (`when:`, `when_subject*`), `payload:` reads
`input.*` as today, and `wrong_state`/unknown-instance answers on the old identity are unchanged. A
struct identity is written whole (`sets: {address: input.new_address}`); a dotted write into one
member of an identity is not in this cut.

The meaning is derived, not carried: `ResolvedOutcome::identity_write` reads it off the subject and
`sets:` the IR always held, so no model's IR bytes move.

## The collision answer

A re-key onto an identity another record carries would leave two rows under one key. The command
must declare its answer, a refusal guarded by exactly

```yaml
- name: taken
  when_related:
    entity: demo.vault.Secret
    where: name == input.new_name
    exists: true
  error: demo.vault.NameTaken
```

over the input the identity is written from (`ess/22`'s row-set form, `predicates.md`). Its rows
are the store just before the branch, so the addressed record's own row is one of them: **a rename
to the identity the record already carries is the collision.** The guard is recognised only in that
exact shape — a narrower selector (`all: [name == input.new_name, …]`), an input guard beside it, a
`count` or `exists: false` test, or an accepting branch would leave a collision unanswered.

Precedence is the row-set refusal's own: after the input-guarded refusals and the addressed row's
existence, before every accepting branch. An old identity no row carries is answered as unknown
even when the new one is carried.

## Validation

| rule | code | location |
|---|---|---|
| the identity write under a source below `ess/23` | `unsupported_format_version`, the only error the branch earns | `…outcomes.<o>.sets.<identity>` |
| the identity write beside `compensates: true` | `unsupported_construct` | `…outcomes.<o>.sets.<identity>` |
| the identity write on the updating branch of a create-or-update pair (`unknown_instance:` on a creation) | `unsupported_construct` | `…outcomes.<o>.sets.<identity>` |
| the identity write on an entity a declared `owns` or `references` relation carries | `unsupported_construct` | `…outcomes.<o>.sets.<identity>` |
| the identity write with no collision answer, or written from anything but `input.<field>` | `missing_declaration` | `…outcomes.<o>` |
| the identity written by `affects:` or `instances:` | `conflicting_declaration`, unchanged (`set_effects::identity_set`) | `…sets.<identity>` |

The compensating refusal and the create-or-update pair read `instance:` as the identity that
persists, and a re-key leaves it naming nothing. A relation's carrier would keep naming the old
identity; cascading the new key to carriers is out of scope. A `moves:` that writes the identity is
not given this meaning in this cut and keeps its meaning and its admission.

Below `ess/23` the identity write was admitted and is now refused naming `ess/23`. That breaks any
document that wrote it, and every such document carried a suite that contradicted it.

## Conformance

No new step and no new suite format: the scenarios use `expect_view contains`,
`expect_subject_absent` (`ess-conformance/22`, `deletes:`'s absence check) and
`snapshot_view`/`expect_view_unchanged` (`ess-conformance/22`, `accepts: nothing`).

- **The re-key's own scenario** (`<command>/outcome/<re-key>`): the record is arranged, the command
  sent with a fresh new identity, and every immediate view read whole requires the row under the
  new identity carrying the arranged fields, no row under the old one (`expect_subject_absent`), and
  the same request again answered by the unknown-instance answer with no event
  (`synthesize::deletion_witness`, which a re-key now shares with `deletes:`).
- **The collision's scenario** (`<command>/outcome/<refusal>`): the row-set arrangement arranges a
  second record under the new identity; every immediate view of the entity is snapshotted before
  the send and required unchanged after it; then the same request is sent with the new identity the
  addressed record's own, and the same refusal, no event and unchanged views are required again.

Arranging a row by its identity needs the selector's identity to be read: a row-set selector over
the identity binds the row's key (`subject_fact::evaluate_row`), a row named by a literal identity
reads it (`row_set::Reading`), an input naming an arranged instance is read as that instance's
identity, and a creation that publishes its identity from an input is steered through that input
(`arrange_toward_bound`). Each applies only where the selector reads the identity, so no other
model's suite moves.

## Targets

| target | answer |
|---|---|
| interpreter | re-keys: the row is removed under the old identity and inserted under the new one; a re-key onto a carried identity the model left undecided is `NotInterpreted` |
| generated Rust | the collision refusal looks the addressed identity and the written identity up through the entity's storage port; the branch writes the identity into the snapshot, deletes the old row and puts the new one |
| Go, Web, Clap | refused by name: `MissingRepresentation` at `commands.<c>.outcomes.<o>.sets.<identity>` |
| Entity Runtime | refused by name: `IdentityChangeUnsupported` at `<c>.<o>.sets.<identity>` |
| `ess verify diff` | the `outcome-sets-changed` entry writing the identity reads `<identity> <- …, re-keying the record`, and its line says the outcome now (or no longer) re-keys the record |
| generated documentation | "It re-keys a `<Entity>`: the record comes to rest under the identity written to `<identity>`…" |

## Not in this cut

A dotted write into a member of a struct identity; a `moves:` that writes the identity; cascading a
new key to the records a relation carries it in; a collision answer written in any other shape than
the exact selector above; Go, Web, Clap and Entity Runtime realisations.
