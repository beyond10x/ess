---
format: aep.planning-md/3
id: architecture-decision-record:a-ui-document-declares-the-host-components-it-does-not-generate
kind: architecture-decision-record
status: proposed
title: A UI document declares the host components it does not generate
summary: 'Proposed: ess-ui/1 host slots (declared, own coverage count, binding-only access, named stand-in); Decider: operator'
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
Status: proposed. Decider: operator. Nothing is built until this record is accepted.

## Context

An `ess-ui/1` document names every node it draws. A section, an overlay and a nested node each hold
one member of the composite union, discriminated by `component` (`crates/ui/ess-ui/src/model.rs:1042-1070`),
or a `primitive`; the union has twelve members (`COMPOSITE_KINDS`, `model.rs:15-28`). `ess ui check`
checks every node, and with `--model` checks each read and command against the specification and
its `ess-http/1` binding (`crates/ui/ess-ui-check/src/model.rs`). The generators draw every node
(`ess generate ui --target react|tui`).

An adopter that moves an existing application onto a generated UI meets parts the union cannot
draw yet: a bespoke editor, a vendor widget, a visualisation with its own interaction. Today such a
part has two homes, both outside the document:

- a patched copy of a generated runtime file, which the next generation overwrites;
- a list kept beside the document (one adopter keeps entries of page, section, component and
  reason), which no ESS command reads, so `ess ui check` reports such a page as fully covered by the
  specification while part of it is hand-written.

Either way the document stops being the whole truth about the page, and a reviewer cannot tell
generated coverage from hand-written coverage.

## Decision

Add a **host slot** to `ess-ui/1`: a declared place in the document where the host application
supplies its own component, inside the generated page and under the generated page's rules. It is
an extension point outside the specification: ESS does not draw it, and does not claim it is
covered by the specification.

### 1. Every host slot is declared in the document

A slot stands where a node may stand, at one of five scopes: a page, a section, an overlay, a tab, or
a field. It declares:

- `host: <name>`, the component name the host registers;
- the scope, by its position in the document (the node path it occupies);
- `reason:`, why the generator cannot draw it (required, non-empty);
- `stands_in_for:`, see 4.

`ess ui check` refuses a host component that no slot declares: a generated project registers only
the declared names, and a name registered by the host that no slot declares is an error, as an
undeclared view or command is today. A slot whose name the host never registers is a build error
of the generated project, not a blank region.

### 2. Host-covered rows are their own count

`ess ui check` and every coverage ESS reports over a UI document (the operations, reads and fields
the document reaches) count a row a host slot reaches as **host-covered**, a third count beside
covered and uncovered. It is never added to the covered count. The text and JSON reports name each
slot with its path and reason.

### 3. The host gets what a generated node gets, and nothing more

A host component receives the same adapter and state access a generated node at that position
receives: the data adapter (`DataAdapter`, `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:58`)
through the slot's declared reads and commands, and the page and scope state of that position. It
declares its reads and commands on the slot as a generated node does, and `ess ui check --model`
checks them against the specification and binding as it checks a generated node's. It reads and
writes only through the binding of what it declares; a host component never calls the server
around the binding, and a read or command it does not declare is not available to it.

### 4. Each slot names the generator item it stands in for

`stands_in_for:` names the composite or primitive the slot replaces (one of `COMPOSITE_KINDS`, or a
primitive), so the gap is visible in the generator's own terms and a later generator release can
take the slot back. Where no generator item fits, `stands_in_for: none` with `reason:` saying why;
that list is the input to the generator's backlog.

## Options

| | what it is | cost |
|---|---|---|
| A | **Host slots in the document** (this record) | One construct in `ess-ui/1` with check rules, a third coverage count, a registration seam in the React project and a placeholder region in the terminal renderer. |
| B | **A side file** (`host-components.yaml`, entries of page, section, component, reason) read by `ess ui check` | No document change; the slot's reads and commands are not checked against the binding, and the file and the document drift apart, since nothing ties an entry to a node path. |
| C | **Change nothing** | Patched runtime copies and private lists stay; coverage keeps counting hand-written parts as generated. |

## Recommendation

A. It satisfies all four constraints with the document as the one source; B satisfies 1, 2 and 4
only by convention and cannot satisfy 3. An adopter's existing side file migrates one entry per
slot: page and section become the node path, component becomes `host:`, reason stays.

## Effect

- **Format:** one optional node shape in `ess-ui/1`, refused by an older reader as an unknown key
  by name; the schema (`schemas/ui/ess-ui.schema.yaml`) and the reference page gain it.
- **Check:** the undeclared-component refusal, the reads and commands of a slot, the host-covered
  count.
- **Generators:** React registers declared slots and passes the adapter and scope; the terminal
  renderer draws a labelled placeholder region at the slot's path, so `ess-ui-test/1` can address
  it; a test step inside a host slot is refused by name in the terminal.
- **Specification:** none. A host slot reads and commands what the specification already serves.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`; the design page and
stories follow it.
