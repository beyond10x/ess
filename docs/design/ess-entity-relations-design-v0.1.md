# ESS Entity Relations — Design v0.1

> **Repository:** `beyond10x/ess`
> **Status:** Decided. `decision-blocker:relation-vocabulary` was cleared on 2026-09-03 with the
> default it named; this page is where that decision is written down.
> **Audience:** Implementors of `ess-domain`, `ess-compiler`, `ess-gen`, `ess-synth`, and of the
> tools that read an ESS specification back out of a projection (`aep reverse openapi`).
> **Relationship to existing work:** Additive. A specification with no `relations:` key parses,
> validates and projects exactly as it did before.

---

## 1. What a relation is, and why it is a construct

An entity today has `name`, `identity`, `fields`, `lifecycle`, `invariants` and `naming`. *An
account owns many commercial clients* is therefore written as a typed id field on the child plus an
invariant somebody remembers to write — and the second half is the half that goes missing. The
adopter case that motivated this construct lost a plan on exactly that: the relationship between an
account and a commercial-client object was inconsistent across passes, and nothing in either stack
was able to refuse the inconsistency, because nothing in either stack held the relation as a fact.

A relation is that fact:

> **entity S** has a relation named **r**, of kind **owns** or **references**, to **entity T**,
> with **one** or **many** of T on the target side, carried by the field **via**.

It is declared on **S only**. Nothing is declared on T, and nothing is inferred from a name.

## 2. The shape

```yaml
entities:
  - name: crm.account.Account
    identity: { name: account_id, type: crm.account.AccountId }
    relations:
      - name: clients
        kind: owns            # owns | references
        target: crm.client.CommercialClient
        cardinality: many     # one | many
        via: account_id
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
```

`via` names **the field that carries the relation**, and which entity that field is on depends on
the kind:

| kind | `via` is a field on | and its type is |
| --- | --- | --- |
| `owns` | the **target** | the source's `identity.type`, unwrapped |
| `references`, `cardinality: one` | the **source** | the target's `identity.type`, or `Optional<…>` of it |
| `references`, `cardinality: many` | the **source** | `List<…>` of the target's `identity.type` |

An `owns` relation's carrier is on the target because that is where the foreign key lives in every
representation this repository projects to, and because the rule that makes `owns` mean anything —
*at most one owner* — is a rule about that one field. Its type is never wrapped: a child has exactly
one owner whether the owner has one child or a thousand, so `cardinality` says how many children the
owner has and says nothing about the child's field.

### 2.1 One example of each kind

**`owns` — the relation the billing example carries.** `billing.invoice.Account` owns many
`billing.invoice.Invoice`; the carrying field is on the invoice.

```yaml
entities:
  - name: billing.invoice.Account
    identity: { name: account_id, type: billing.invoice.AccountId }
    relations:
      - name: invoices
        kind: owns
        target: billing.invoice.Invoice
        cardinality: many
        via: account_id

  - name: billing.invoice.Invoice
    identity: { name: invoice_id, type: billing.invoice.InvoiceId }
    fields:
      - name: account_id
        type: billing.invoice.AccountId     # the carrier: the owner's identity type
```

**`references` — a link that claims no ownership.** An invoice references at most one preceding
invoice it replaces; the carrying field is on the invoice itself, and it is optional because the
cardinality is `one` and most invoices replace nothing.

```yaml
entities:
  - name: billing.invoice.Invoice
    identity: { name: invoice_id, type: billing.invoice.InvoiceId }
    fields:
      - name: replaces
        type: Optional<billing.invoice.InvoiceId>
    relations:
      - name: replaced
        kind: references
        target: billing.invoice.Invoice
        cardinality: one
        via: replaces
```

## 3. What is refused

Five rules, checked in `ess-domain` beside `validate_lifecycle_causes`, because a relation — like a
command's causation of a lifecycle move — is a fact about two members that neither of them can check
alone. Each refusal reuses an existing `ValidationCode`: the codes classify *what kind of mistake*
was made, and a relation makes no new kind. Each carries a hint.

| # | rule | code | hint says |
| --- | --- | --- | --- |
| 1 | the target is a declared entity | `undeclared_reference` | the declared entities |
| 2 | `via` names a declared field of the carrying entity | `missing_declaration` | that entity's fields |
| 3 | the `via` field's type is the one §2 requires | `type_mismatch` | the type the field must have |
| 4 | at most one `owns` relation targets any entity | `conflicting_declaration` | the other owner |
| 5 | at most one relation is carried by any one field | `duplicate_declaration` | the relation that claimed it |

### 3.1 One refused example per rule

**1 — a target nothing declares.** `undeclared_reference`.

```yaml
relations:
  - { name: clients, kind: owns, target: crm.client.CommercialCleint, cardinality: many, via: account_id }
#                                        ^ nothing declares this entity
```

**2 — a `via` field that does not exist.** `missing_declaration`. The target here is real and has
no `account_id`, so the relation names a carrier nobody can read.

```yaml
relations:
  - { name: clients, kind: owns, target: crm.client.CommercialClient, cardinality: many, via: acount_id }
```

**3 — a `via` field of the wrong type.** `type_mismatch`. `CommercialClient.account_id` is a
`String`, and the source's identity type is `crm.account.AccountId`; the two are both strings
underneath, which is exactly the confusion a typed model exists to refuse.

```yaml
- name: crm.client.CommercialClient
  fields:
    - { name: account_id, type: String }        # must be `crm.account.AccountId`
```

The same rule refuses a `references`/`many` relation carried by an unwrapped field: `List<…>` is
what `many` means on the source side, and a single id cannot hold many.

**4 — two owners for one entity.** `conflicting_declaration`, reported on the second owner in name
order, naming the first.

```yaml
- name: crm.account.Account
  relations:
    - { name: clients, kind: owns, target: crm.client.CommercialClient, cardinality: many, via: account_id }
- name: crm.partner.Partner
  relations:
    - { name: clients, kind: owns, target: crm.client.CommercialClient, cardinality: many, via: account_id }
#     ^ `crm.client.CommercialClient` is already owned by `crm.account.Account`
```

**5 — one field, two relations.** `duplicate_declaration`. Each of these is well formed alone —
the account owns the invoice, and the invoice names the account it belongs to — and together they
give `account_id` two meanings, which is one meaning more than a projection can annotate it with.
Rule 4 is a special case of this one and is reported instead of it, so two entities owning one
target is one refusal rather than two.

```yaml
- name: billing.invoice.Invoice
  relations:
    - { name: account, kind: references, target: billing.invoice.Account, cardinality: one, via: account_id }
#     ^ `billing.invoice.Account`'s `invoices` already carries `Invoice.account_id`
```

An unowned entity is **not** refused. Ownership is a claim somebody makes, not a duty every entity
has, and refusing an entity nobody owns would make a root aggregate an error.

## 4. Projection: one key, `x-ess-relation`

Every projection carries the relation under exactly one extension key, `x-ess-relation`, **on the
carrying property** — the property that renders the `via` field. It carries the whole relation, so a
reader of one property never has to find the declaration it came from:

```yaml
x-ess-relation:
  name: invoices
  kind: owns
  source: billing.invoice.Account
  target: billing.invoice.Invoice
  cardinality: many
  via: account_id
```

`source` is in the payload because for an `owns` relation the carrying property is on the *target*,
so without it the annotation would name one end of a two-ended fact.

### 4.1 JSON Schema

The schema projection publishes one document per entity, `entities/<name>.schema.json`: the
identity, every declared field, and `state`. That document exists because of this design — an entity
was previously projected by the documentation and by nothing that a tool reads — and it is where the
carrying property lives:

```json
"account_id": {
  "$ref": "#/$defs/billing.invoice.AccountId",
  "x-ess-relation": { "name": "invoices", "kind": "owns", "source": "billing.invoice.Account",
                      "target": "billing.invoice.Invoice", "cardinality": "many", "via": "account_id" }
}
```

There is **no `$ref` to the target** here. A schema document is self-contained and carries under
`$defs` only the named types it reaches; a pointer to another entity's document would be a
cross-file `$ref`, which is the resolution mode this projection refuses everywhere else.

### 4.2 OpenAPI

The same entity schemas are published, under a root extension `x-ess-entities`, keyed by qualified
name and holding the named types they reach. The same key sits on the same property — plus a `$ref`,
because this document *does* have a schema to point at. The `$ref` names **what the property's value
identifies**, which is the other end from the carrier: a field holding an owner's id identifies the
owner. Pointing at `target` unconditionally would point a property of the invoice at the invoice.

```yaml
x-ess-entities:
  billing.invoice.Invoice:
    x-ess-kind: entity
    properties:
      account_id:
        $ref: '#/x-ess-entities/billing.invoice.AccountId'
        x-ess-relation:
          name: invoices
          kind: owns
          source: billing.invoice.Account
          target: billing.invoice.Invoice
          cardinality: many
          via: account_id
          $ref: '#/x-ess-entities/billing.invoice.Account'
```

**An extension and not `components.schemas`, and the reason is measured rather than stylistic.**
`components.schemas` is the half of the document `ess import openapi` reads back — the subset this
repository guarantees a semantic round-trip for — and an entity's shape reaches constructs that
subset does not carry. Putting the billing example's `Invoice` there made this repository's own
adapter refuse the document it had just generated:

```console
$ ess import openapi --path generated/openapi/invoice-service.yaml
refused: /components/schemas/billing.invoice.Payee: schema must declare `$ref` or one supported `type`
refused: /components/schemas/billing.invoice.Invoice/properties/metadata/additionalProperties: supported objects must be closed with `additionalProperties: false`
```

A `Map` field and a tagged union, both legal in the model and both outside the adapter's subset. An
unknown root extension is ignored by every reader that does not know it — verified against the
adapter — so the contract stays byte-for-byte what it was, and the model view sits beside it.
Publishing it adds **no path, no method and no query parameter**: nothing here says an entity is
addressable.

Rejected here, rather than in §5, because it is a decision about this projection only: growing the
importer to accept maps and unions would change `ess-service-interface/1`, which is a persisted
format, and a format version is not something a relation gets to spend.

### 4.3 Rust

The synthesised entity data struct already carries one doc comment per field. The carrying field's
comment gains the relation, as a sentence naming both ends:

```rust
/// `account_id` — `billing.invoice.AccountId`.
///
/// Carries `invoices`: `billing.invoice.Account` owns many `billing.invoice.Invoice`.
pub account_id: AccountId,
```

A doc attribute rather than a typed reference, deliberately. A typed `Account` field on `InvoiceData`
would make the child hold the parent — a navigation decision the runtime owns, not the specification
— and `ess synthesize` generates no store to navigate with.

## 5. Rejected alternatives

| alternative | why it was rejected |
| --- | --- |
| **An invariant-only encoding** — no construct; write `account_id exists` and let a reviewer read the relation out of it. | It is what the model has today, and the adopter case is the evidence: an invariant is prose to every consumer. Nothing can refuse a second owner, no projection can carry the relation, and a reverse tool has nothing to emit. |
| **A separate `relations.yaml` document.** | A relation is a property of an entity. In its own file it can name an entity that no longer exists, and every reader of an entity has to load a second document to know what the entity is part of. The model already refuses this shape for lifecycles, which §4.7 could have put in a file of their own and did not. |
| **Declared on both ends, with a consistency check.** | Two declarations of one fact are two declarations that can disagree, and the check that reconciles them is a rule that only exists because of the redundancy. The reverse direction is derivable: every `owns` target has at most one owner, so *who owns me* is a lookup, not a declaration. |
| **`min`/`max` on both sides instead of `one \| many`.** | `0..1` and `1..1` differ only in whether the carrying field is `Optional`, which the field's own type already says, in the place a projection already reads. `min`/`max` would be a second, weaker statement of the same fact. |
| **`on_delete: cascade \| restrict \| detach` in v0.1.** | ESS describes; the entity runtime decides. A deletion policy in the specification is a promise no projection here can keep — nothing generated by `ess synthesize` deletes anything — and a policy nothing enforces is worse than no policy. Out of scope for v0.1, reconsidered when a projector establishes the semantics. |
| **A `relation` *type kind* beside `newtype`, `struct`, `enum`, `union`.** | A relation is not a value. Nothing carries one on the wire; what crosses the wire is the id field, which is already typed. |
| **A document-level `x-ess-relations` array in each projection.** | A reader looking at `account_id` would not find it. The whole point of the key is that the property carrying the relation says so. |

## 6. Out of scope for v0.1

- **Navigation.** No generated `account.invoices()`, no traversal verb on the CLI. Projections
  describe; runtimes navigate.
- **Deletion, cascade and orphan semantics**, per §5.
- **Relations to value types.** `via` resolves against entities only; a struct is not an identity.
- **Relations across specifications.** A target is resolved inside one specification, like every
  other reference.
