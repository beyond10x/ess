---
title: Generate contracts and documentation
sidebar_position: 5
description: Derive repository docs, static-site source, JSON Schema, OpenAPI and AsyncAPI from a specification, keep the committed output drift-checked, and know what a projection cannot carry.
---

# Generate contracts and documentation

Once a specification validates, five projections derive from it. Everything on this page is
deterministic: the same source produces byte-identical output, so generated artifacts can be
committed, reviewed and drift-checked like source.

## The five projections

```shell-session
$ ess generate --path examples/billing --kind openapi
openapi/email-service.yaml — 4608 byte(s)
openapi/invoice-service.yaml — 21077 byte(s)
2 artifact(s), nothing written
```

| `--kind` | Output | Why it exists |
|---|---|---|
| `docs` | Markdown with Mermaid diagrams (lifecycles as state diagrams, bindings as flowcharts) | the cheapest completeness check: a construct with no rendering is a hole in a page a person reads |
| `site` | the same Markdown with YAML frontmatter, plus `sidebar.json` | static-site-ready source without inventing presentation or hosting choices |
| `schema` | JSON Schema for command inputs, messages, named types, and entities | the type system projected without losing its distinctions — newtypes stay separate definitions |
| `openapi` | one OpenAPI 3.1 document per component | the specification *is* the HTTP contract, not a document beside it |
| `asyncapi` | one AsyncAPI 3.0 document per component | the same for messaging, including what happens when a binding fails |

Omit `--kind` and all five are produced together. The CLI prints the exact artifact inventory; it
does not hide a projection that produced nothing.

Without `--out` you get a listing and nothing is written — a command that looks read-only does not
write into whatever directory you happened to be in. **`--out` names the root of the tree, not one
projection's directory**: each artifact's path already begins with its projection, so the committed
output of the whole set is one command.

```shell-session
$ ess generate --path examples/billing --out target/projections
```

Every artifact carries provenance: specification version, the digest of the resolved model, a
separate digest of the model slice it derives from, and the regeneration command. The model digest
is over the *model*, not the source files, so it does not move when a comment does — a digest that
moves for no reason is one every reader learns to ignore.

See [the worked example](../examples/specification-to-contracts.md) for one command's source next to
each generated document.

## What `site` means

`site` was introduced in ESS `0.4.0`. It wraps the `docs` projection rather than creating a second
description of the model: each `.md` page gains a title and deterministic sidebar position in YAML
frontmatter, and `sidebar.json` lists the page ids in one stable order. For example:

```shell-session
$ ess generate --path examples/billing --kind site --out target/projections
```

produces `target/projections/site/index.md`, the domain pages, the interaction and topology pages,
and `target/projections/site/sidebar.json`.

This is **specification to documentation**, not documentation to specification. ESS still accepts a
typed ESS YAML document or directory as input; it does not parse prose or Markdown into model
semantics. “Ready for a static site” means Markdown, frontmatter, links, and sidebar data. The
projection emits no HTML, CSS, theme, navigation shell, deployment, or hosted site. A documentation
system such as Docusaurus consumes these files and owns those presentation decisions.

## The graph, without generating a tree

`ess graph` prints the actor/command/event picture the generated docs open with:

| `--format` | Output |
|---|---|
| `dot` | Graphviz, for `dot -Tsvg` |
| `mermaid` (default) | a `flowchart`, unfenced — redirect into a Markdown file or paste into a PR |
| `json`, `yaml` | the nodes, edges and groups themselves — 13 nodes, 7 edges and 3 groups for `examples/billing` |

One renderer produces both the CLI's diagram and the documentation's: `ess graph --path
examples/billing --format mermaid` emits exactly the bytes fenced under *The system as a graph* in
`generated/docs/index.md`, and a test compares them, so the two cannot drift.

## Drift-checking in CI

Commit the generated output and regenerate in CI:

```shell-session
$ cargo xtask generate --check    # committed projections still match the specification?
projections are up to date
```

The check runs the public `ess generate` command, compares every owned file byte for byte, and also
fails on a committed file no projection produces any more. Run `cargo xtask generate` to reconcile
the tree, review the resulting diff, and commit it with the specification change. Structural
synthesis, conformance, schema, and infrastructure fixtures are exercised by their Rust tests in
`task check`; this repository does not advertise repository commands it does not ship.

## What a projection can quietly destroy

Two questions to ask of any generated artifact, answered honestly for these:

* **A newtype collapses on the wire.** `billing.invoice.Email` and `billing.email.EmailAddress`
  stay separate schema definitions — each carries `"x-ess-kind": "newtype"` and its own name — so
  code generators emit two types. But both are `"type": "string"` on the wire, and a payload with
  the two values swapped validates clean. JSON Schema constrains structure; it cannot carry nominal
  identity.
* **A command's HTTP path is a convention.** The model has no `exposures:` construct, so
  `/invoices/commands/create-invoice` is a shape the generator chose — written into the generated
  document's own `info.description` rather than left for a reader to infer.

And one check that is scoped rather than total, stated per projection:

| projection | what is checked | what is not |
|---|---|---|
| `schema` | every document is validated against the real JSON Schema 2020-12 meta-schema and built into a validator | — |
| `openapi` | every **embedded** schema is validated against the same meta-schema, because OpenAPI 3.1's dialect *is* 2020-12 | the envelope, checked against an enumerated list by hand |
| `asyncapi` | the envelope is checked as a skeleton: version, `info`, `channels`, `operations`, and every operation's `action` | the payloads. They are AsyncAPI Schema Objects and declare no `schemaFormat`, so validating them against 2020-12 would assert a dialect the document does not claim |

What closes the two gaps is vendoring the OpenAPI 3.1 and AsyncAPI 3.0 meta-schemas, which is an
open decision rather than an oversight: neither ships with anything here, and a test may not fetch
one — the validator is built with `default-features = false` and has no retriever, so it could not
reach the network if it tried.
