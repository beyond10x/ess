# A created subject is created under an owner that exists — Design v0.1

> **Repository:** `beyond10x/ess`
> **Status:** Decided and implemented. `ess-conformance`'s synthesizer, plus one lookup on `EssIr`.
> **Audience:** Implementors of `ess-conformance`, and authors of a model whose entities own one
> another.
> **Relationship to existing work:** Additive, and it consumes a construct that already exists —
> [`ess-entity-relations-design-v0.1.md`](./ess-entity-relations-design-v0.1.md). A model that
> declares no `relations:` synthesises byte-for-byte what it did before.

---

## 1. The defect

`prepare` used to say this, and the comment was the whole of the reasoning:

```rust
// Empty for a branch that changes no entity, and for one that *creates* its own subject: a
// created instance is the scenario's own doing, so there is nothing to arrange first.
ResolvedEffect::Creates => return Ok(Setup { after: Some(initial), ..Setup::none() }),
```

For a root that is exactly right. For an entity another one declares it **owns** it is false, and
`owns` is what makes it false: §1 of the relations design says the far side *does not stand on its
own*. A scenario that creates one row without the row it belongs to arranges a world the
specification says cannot exist.

**Measured, on an adopter's model.** `aiagent.agent.Agent` owns `aiagent.agent.AgentDraft`. Every
route into a draft starts at `SaveDraft`, every generated `SaveDraft` began by saving a draft
against a fabricated agent id, and the implementation requires the agent to be there
(`store.rs`: "Requires the agent to exist"). Fifteen scenarios — every draft scenario the model
had — reported `unsupported`. That is not a failure and it is not a pass: it is **no information
about the implementation**, produced by a suite asking a question that has no answer.

## 2. What is arranged

Before the command that creates an owned subject runs, the owner is brought into existence and
bound, and the creating command's input is pointed at it:

```
execute_command   ledger.book.OpenAccount
expect_outcome    ledger.book.OpenAccount/opened
capture_instance  account  <- ledger.book.AccountOpened.account_id
execute_command   ledger.book.PostEntry  { account_id: instance(account), memo: "memo" }
```

Recursively, so a three-deep chain is three commands, and at the **initial** state of each owner and
no further: the relation says the owner must exist and says nothing about what state it must be in.
Driving it anywhere else would be the synthesizer inventing a requirement the model does not have.

## 3. The link is `sets:`, and nothing else

A relation names the **entity field** that carries the ownership. What an arrangement needs is the
**command input** that fills that field, and the one place a model says which input fills which
field is the creating branch's `sets:`:

```yaml
outcomes:
  - name: posted
    creates: ledger.book.Entry
    sets:
      account_id: input.account_id   # <- the field `owns` is carried by
```

Two alternatives were available and both are rejected:

| rejected | why |
|---|---|
| **the input spelled like the `via` field** | `Invoice.account_id` filled by `input.account_id` is the common case and the reason the heuristic looks safe. It is the inference the relations design refuses for declaring a relation (§1: *nothing is inferred from a name*), and it is worse here than there: a wrong guess does not fail to compile, it points a scenario at somebody else's row and asserts against it |
| **the one input typed as the owner's identity** | Better than a name and still a guess — two inputs of that type make it ambiguous, one of them makes it *unambiguously wrong* where the model meant the other. And in the motivating model the owned entity's own identity has the same type as its owner's, so "typed as the owner's identity" does not even pick out one field |

A model that has not written the `sets:` line has not said which input names the owner. The repair is
one line in the model, and it is a line worth having for its own sake: `sets:` is also what makes the
field assertable (§5).

## 4. Four ways there is no owner to arrange, and none of them refuses

Falling back means the suite is the suite it was before: the creating command runs with a synthesised
literal, exactly as it did.

| | |
|---|---|
| nothing owns this entity | it is a root. Refusing one would make every aggregate root an error — §3 of the relations design |
| `sets:` does not determine the carrying field | the model has not said which input names the owner, per §3 above |
| nothing creates the owner | **`examples/billing/` is this case.** `Account` is declared, it owns `Invoice`, and no command in the model brings an account into existence. An arrangement cannot create what the specification never says how to create |
| the owner is already being arranged | two entities owning each other. `validate_relations` does not refuse it, because neither declaration is wrong on its own — it refuses only *two* entities claiming one target |

**A refusal was the alternative, and it is the wrong answer.** `ESS-SYNTH` refusals are how this
crate reports a scenario it cannot honestly build, and the case for using one here is real: a suite
that creates an invoice under an account nobody created is arranging the impossible world §1 is
about. But billing's invoice scenarios *pass*, against a reference implementation that does not
require the account, and refusing would delete them — a lower `total` and a lower `passed`, in
exchange for a truth nobody can act on. What is lost by falling back is visible in the suite itself:
the arrangement has no command creating an owner in it.

The guard is the **chain** — the entities this one is being arranged for — and not a depth count. A
number would have to be right about how deep is deep enough; the chain is right because what makes an
owner unarrangeable is precisely that arranging it is already in progress.

## 5. What the scenario then asserts

`settled` used to keep a `sets:` source only where the invocation supplied a **literal**, so a field
filled from an input that names an arranged row was dropped and nothing was said about it. It now
keeps the reference, and the assertion that follows is one a generated suite could not previously
make:

```
expect_view  ledger.book.EntryById  contains { account_id: instance(account) }
```

The entry holds the id of the account **this scenario opened** — a value neither the suite nor the
specification can spell, and the assertion an implementation that files the new row under a
different owner fails.

## 6. Out of scope

- **Deletion and cascade.** Unchanged and still out of scope, per §6 of the relations design. What
  a delete does to an owned row is a command outcome; nothing here says it.
- **`references`.** The far side of a reference stands on its own and outlives the link, so it is
  not something a row's existence depends on. `EssIr::owner_of` excludes it, and that exclusion is
  the content of the function.
- **A required owner *state*.** `owns` says the owner exists. A model that needs the owner to be in
  some particular state before a child may be created is stating a precondition, which is a
  different construct and an open question elsewhere.
