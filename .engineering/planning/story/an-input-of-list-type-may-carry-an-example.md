---
format: aep.planning-md/3
id: story:an-input-of-list-type-may-carry-an-example
kind: story
status: draft
title: An input of List type may carry an example that synthesis builds the input from
summary: 'example: is refused on a List input as not a scalar, so a list input cannot carry a witness value and synthesis sends [] or a first-variant list; same on 0.67.0 and 0.68.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 6
---
## Need

Reported by an adopter on 2026-10-22 (item 4 of the second batch, relayed by the orchestrator; adopter names kept out). A
pub/sub protocol contract has a `Handshake` command whose input `supported_connection_types` is a `List` of an enum or of a
String newtype. Synthesis builds that input with no authored value, and a real implementation accepts only lists that name a
member it knows, so every success scenario of the command fails against it. `example:` on a scalar input already fixes the same
problem for scalars (ess/11).

Needed: an input of `List` type may carry an `example:` (a list of admitted element values), and synthesis builds the input from
it, as it does for a scalar.

## Reproduction

Brand-free fixture, `format: ess/29`: one domain `demo.server`, an enum `ConnectionType [long-polling, websocket]`, one entity
`Client`, one command `Handshake` creating it, with the input

```yaml
- {name: supported_connection_types, type: "List<demo.server.ConnectionType>", example: [long-polling]}
```

`ess specify validate --path spec.yaml`, exit 1 on 0.68.0 and 0.67.0, identical text:

```text
- [type_mismatch] command.demo.server.Handshake.input.supported_connection_types.example: `List<demo.server.ConnectionType>` is not a scalar (hint: an example sits on a scalar input: a primitive other than Binary64, or an enum, through any newtype or Optional)
- error[ESS-COMMAND-002]: `List<demo.server.ConnectionType>` is not a scalar
```

The adopter's text was `unknown field `example`` (format not relayed); on ess/29 the refusal is the typed one above. Without
`example:`, synthesis sends `["long-polling"]` for this enum list (the first variant); for a String-newtype element type the
adopter reports `[]` or `["<field>.0"]`, not re-probed here.

## Where it lives

- `crates/specify/ess-domain/src/command.rs:5864` (`example_admitted`), called from `validate_examples` (:5839): an example is
  admitted only on a scalar input, through any newtype or `Optional`.
- How synthesis consumes a scalar example was not traced; the fix extends it to a list of element examples, each held to the
  element type.

## Regression status

Not a 0.68.0 regression: 0.67.0 refuses the same fixture with byte-identical diagnostics. Gap, present since examples landed
(ess/11). Does not block 0.68.0.

## Further report

Source: a downstream specification, public repository beyond10x/patchback, commit 69e87fd, first reported as "`input example:` is ignored for list inputs" (no count). Corrected by the downstream session's repro.

- Corrected: a list `example:` is refused, not ignored. On 0.67.0 and on 0.68.0, adding `example:` to `LoadPatch.nodes` and `.edges` (`List<NodeDecl>`, `List<EdgeDecl>`) gives, on both releases:

```text
[type_mismatch] command.patchback.patch.LoadPatch.input.nodes.example: `List<patchback.patch.NodeDecl>` is not a scalar (hint: an example sits on a scalar input: a primitive other than Binary64, or an enum, through any newtype or Optional)
error[ESS-COMMAND-002]: `List<patchback.patch.EdgeDecl>` is not a scalar
```

- The real gap for this report: a list input cannot carry a witness value. Synthesis gives `LoadPatch` `nodes: []` and one edge between a node and itself (written suite, 0.68.0: `"nodes":{"kind":"literal","value":[]}`), so no synthesized scenario passes `ValidatePatch` and `Compiled` is never reached. That is the root case of `story:seeded-state-reaches-related-rows` (62 of its 65 ESS-SYNTH-004 refusals).
- Where list witnesses are chosen: `crates/verify/ess-conformance/src/witness.rs:993` (`ResolvedTypeRef::List` is `Node::Seq(Vec::new())` in the default value) and `:5355-5372` (`list`: unless the path is expanded by a guard or copied, the witness is the empty sequence; a copied list holds one element). The scalar example is consulted at `:5307` (`self.examples.get(path)`, filled from the command at :5229); a list path never has one because `example_admitted` (`crates/specify/ess-domain/src/command.rs:5864`) refuses it (traced).
- A struct-element list (`NodeDecl`) needs more than an `example:` of scalars: a witness element needs a value for each member, so the fix may be an `example:` holding a list of records, or a way to name the shape of a minimal element. A design choice for the story, not decided here.
- The cited commit holds no `example:` on a List input (checked `spec/domains/*.yaml`), consistent with the construct being refused; the repro added it.
- Not needed from the downstream repository any more; the repro files are in hand.
