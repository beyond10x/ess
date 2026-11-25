---
format: aep.planning-md/3
id: story:a-specification-declares-how-a-document-is-read-into-its-entities-and-the-generated-decoder-applies-it
kind: story
status: draft
title: A specification declares how a document is read into its entities (an absent member's default, an entity keyed by its identity, root and container fields, a document root) and the generated decoder applies it
summary: 'A source format step: rules a to d of a consumer''s document format have no declaration today; e to g need none (TD-D9 to TD-D11)'
tags:
- external-request
- format-step
- gap-reports-2026-11-24
- synthesize-rust
relations:
- serves: vision:O2
- depends_on: story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table
- informed_by: story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own
revision: 1
---
## Need

A specification declares how a document is read into its entities, and the generated typed decoder applies the declaration, so a
consumer whose document format has conventions of its own (an absent member that means a default, an entity written as an entry of
a map under its identity, a field written once at the root) writes no reader between the file and the entities. Gap 5 of a consumer's
request of 2026-11-25, rules a to g of seven; this story holds a to d, which need a source format step, and records e to g, which do not.

## Reported case

From a consumer, 2026-11-25; the consumer's own text and rule list are not copied. The seven rules, in neutral words:

- a. a default for an absent member: an absent enum member reads as one declared variant; an absent required list reads as the empty list.
- b. an entity written as an entry of a map of its kind, its identity being the key and not a member.
- c. a field of every entity written once at the root of the document, and a field of a child given by its container (the child's
  identity being built from the container's and its own).
- d. a root that is a document and not an entity: a format marker, the identity, one map per kind of entity; several files form one
  document set with unique ids across them.
- e. a `Decimal` written as a number token and read as the text of that token, never through a float; an exponent form and a leading
  `+` refused; the quoted text accepted too.
- f. a `Date` or `Timestamp` member read as the text of its scalar.
- g. a decoder that reads a value tree the consumer built from YAML, or a YAML reader in the crate, which ess decides.

The consumer's twelfth rule (a second kind of root with a placeholder syntax) stays hand-written and is not covered here.

Measured on a 0.99.1 build, 2026-11-25 03:22 CET: the `ess/37` source format has no key for a, b, c or d on a type, field or
entity (a search of `crates/specify/ess-domain/src/types.rs` for a field default or a document-form key finds none); the strict wire
decoder of `ess-synth` (`rust/wire.rs`, `decode_primitive`, 1038-1051) reads a `Decimal` from a string.

## Findings covered

- Gap 5 of one consumer's request of 2026-11-25, rules a to d. Rules e, f and g are recorded in the Fit review and in TD-D9 to TD-D10.

## Fit review

1. **Need.** The generator leaves no hand-written model code only if each rule is declared once, in the specification, and applied by
   the decoder: a rule kept in the consumer's README is a rule two readers can disagree about.
2. **Class.** A feature with a source format step: a to d add declarations to the authored specification, so a new `ess/N`, its
   reference and diagnostics, and a design page before code (`AGENTS.md` § Where work is tracked: a construct is a design page before
   it is code).
3. **Which rules need a format step.** a, b, c, d: yes, each is a declaration that does not exist. e: no format step, a decision about
   the wire contract (TD-D9). f: no format step and, to be confirmed by the first decoder test, no change: a YAML 1.2 core-schema
   reader gives a string for a date, and the strict decoder reads `Date` and `Timestamp` from a string. g: no format step, a decision
   about a dependency (TD-D10).
4. **Fit.** Rules a to d are the document form of a model, a layer between a file and the entities that today has no home in the
   specification. A normalization recipe (`crates/generate/schema-contract/src/realize/normalize.rs`) maps one document to another and
   may be the place; whether it can carry a to d is a decision for the design page, not claimed here.
5. **Second adopter.** Any specification whose instances live in files written by hand: configuration, catalog and manifest formats.
6. **Cost.** A format version, a reference page, diagnostics, a design page; generated decoder bytes change only for a model that
   declares a rule. Release train: needs a format step.
7. **Existing stories.** None covers a to d. `story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own`
   borders rule a (a default is a plain value in the model; that story is about the UI loader's defaults, not a data member's).
   `story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table`
   is the decoder this story's rules extend (depends_on).

## Decisions

- TD-D9, rule e, a `Decimal` as a number token. A: the strict decoder stays string-only; the number-token reading is a document-form
  reading declared with a to d (so e joins this story's format step). B: the strict decoder accepts a number token and the published
  schema follows (a schema projection change, byte pins move). C: the consumer's value-tree builder quotes the token; ess changes
  nothing. Recommended A. Wire contract: the owner's.
- TD-D10, rule g, who reads YAML. A: the consumer brings the reader and hands over `json::Value` (public in the crate); ess changes
  nothing. B: the crate carries a YAML 1.2 core-schema reader, emitted and dependency-free like `json.rs` (a second fixed
  module in every crate that asks for it). C: the crate takes a third-party YAML crate (architecture: the owner's; it drops the
  zero-third-party rule). Recommended A.
- TD-D11, how a to d are declared. A: on the type, field and entity declarations of the specification (a new `ess/N`). B: in a
  separate document-form file beside the specification, a format of its own. C: through a normalization recipe. Recommended A, to be
  confirmed by the design page; the choice is architecture: the owner's.

## Acceptance

- an_absent_member_reads_as_its_default: with rule a declared, a document without the member decodes to the declared variant or the
  empty list; without the declaration, it is refused as today.
- an_entity_is_read_from_a_map_entry: with rule b declared, `{<id>: {...}}` decodes to entities whose identity is the key, and a
  document that also holds the identity as a member is refused.
- a_root_field_and_a_container_field_reach_the_children: with rule c declared, a field written once at the root and a field given by
  the container are present on every entity, and an entity that writes them itself is refused.
- a_document_set_has_unique_ids: with rule d declared, two files of one set decode to one document; a repeated id is refused naming
  both files.
- the_format_step_is_complete: the reference page, the diagnostics, the format table and the design page exist before any decoder code.

## Scope

- cited: `crates/specify/ess-domain/src/types.rs`, `crates/specify/ess-compiler/src/ir.rs`, `crates/generate/ess-synth/src/rust/wire.rs`
- inferred: `docs/design/` (a new page), `website/docs/reference/formats.md`, `CHANGELOG.md`
