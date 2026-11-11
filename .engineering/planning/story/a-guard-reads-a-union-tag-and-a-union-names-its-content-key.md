---
format: aep.planning-md/3
id: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
kind: story
status: active
title: Design page, grid O union column, validation and prover for union variant guards and a declared content key
summary: 'Adopter report G2: A guard cannot read a union''s tag, and a union cannot name its content key (ess#637).'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#637
- provider: github
  reference: beyond10x/ess#764
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/union_variant_guards.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/union-unit-variants.md
- confidence: inferred
  path: docs/design/union-variant-guards-and-content-key.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:34:15Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:34:16Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Two facts about an adjacently tagged union cannot be stated.

1. A guard cannot select on which variant a union value holds. The union compares as an aggregate,
   so `any_of`, `==` and a path through the tag are all refused.
2. A union cannot name its content member. The projections fix it as `value` (`content` when the
   tag is `value`), so the wire shape `{type, config}` cannot be declared.

Minimal reproduction (0.92.0, `ess/36`):

```yaml
  - name: example.g.Kind
    kind: union
    tag: type
    variants:
      text:
      web_crawl: example.g.CrawlConfig
...
  - name: example.g.Add
    input: [{name: kind, type: example.g.Kind}]
    outcomes:
      - name: fetched
        when:
          kind: {any_of: [web_crawl]}
        creates: example.g.Source
```

```text
error[ESS-COMMAND-002]: `kind in [web_crawl]`: operator `AnyOf` does not admit `example.g.Kind` (aggregate)
error[ESS-COMMAND-002]: `kind == web_crawl`: operator `==` does not admit `example.g.Kind` (aggregate) and `Text literal `web_crawl`` (Text)
error[ESS-COMMAND-003]: `kind.type` cannot select `type` from `example.g.Kind` (resolved prefix type `example.g.Kind`)
```

With `content: config` beside `tag: type`: ``domains/g.yaml: unknown field `content`, expected `tag` or `variants` ``.

This story writes the design and admits both in the source language. Synthesis, the targets and the
projections follow in the stories named under Decisions.

## Acceptance

- A design page `docs/design/union-variant-guards-and-content-key.md` exists, and grid O in
  `docs/design/language-grids.md` gains a union column (`==`, `!=`, `in`, `not in` over variant
  labels: yes; ordering, arithmetic, text: no, with the reason).
- At the new source format, `ess specify validate` exits 0 on the reproduction, on `when: kind ==
  web_crawl` and on `when: kind not in [text]`. The same comparison over a stored union field in
  `when_subject:`, a `when_related:` predicate, a row-set `where:` and a view `filter:` validates
  too.
- A label that is not a variant of the union is refused as `type_mismatch` naming the declared
  variants. A comparison of a union with a field, an ordering and a path into a variant's payload
  (`kind.url`) are each refused by name.
- At the new format, `content: config` beside `tag: type` validates. `content:` equal to `tag:` is
  refused as `conflicting_declaration`. Below the new format, both the guard and `content:` are
  refused as `unsupported_format_version` naming it.
- The finite prover treats a union as the set of its variant labels: `fetched` guarded on
  `[web_crawl]` and `stored` guarded on `[text]` are proved exhaustive with no default branch.
- The compiled IR of a union without `content:` and of a model with no union guard keeps its bytes
  (`ess specify compile` output unchanged for `examples/`).

## Scope

- inferred: docs/design/union-variant-guards-and-content-key.md
- cited: docs/design/language-grids.md
- cited: docs/design/union-unit-variants.md
- cited: crates/specify/ess-domain/src/types.rs
- cited: crates/specify/ess-domain/src/expression.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-domain/src/command/finite.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-domain/tests/union_variant_guards.rs
- inferred: website/docs/reference/predicates.md
- inferred: website/docs/reference/spec-versions.md
- inferred: schemas/generated/ess.schema.json
confidence: medium

## Fit review

1. **Need.** (a) One command branch per kind of a tagged input, where the kind is the union's
   variant. (b) A wire shape whose payload member is not named `value`. Minimal reproduction above.
   The triage run reproduces it (`ess specify validate --path guard`, `compact-eq`, `tag-path` and
   `content-key`, each exit 1). The requester's proposed syntax: `kind: {any_of: [web_crawl]}`, and
   `content: config` beside `tag:`.
2. **Class.** Gap, both halves. (a) "The model has no predicate or `match` over union variants"
   (`docs/design/union-unit-variants.md:90-91`), and the checker types a union as an aggregate
   (`crates/specify/ess-domain/src/expression.rs:2404`). (b) The content key is a function of the
   tag alone (`crates/generate/ess-gen/src/types.rs:1083-1090`). The raw union declares only `tag`
   and `variants` (`crates/specify/ess-domain/src/types.rs:1863-1871`).
3. **Already expressible?** (a) Only by sending an enum of the tag values beside the union
   (`repro/enum-idiom` validates and synthesizes). That idiom changes the wire shape: it adds a
   member. It also leaves the two members free to disagree, which no scenario checks. So it is no
   answer under policy 1 (`docs/design/generic-language-features.md:588-592`). (b) No idiom: the
   wire member name is the fact.
4. **Fit.**
   - Grid O (`docs/design/language-grids.md:65-82`) has no union column. A union of unit variants
     and an enum "both name one of a fixed set of labels; they differ only on the wire"
     (`union-unit-variants.md:33-35`). So the union column takes the enum column's comparison and
     membership cells, spelled as for an enum (`status in [A, B]`, `status == A`), and refuses the
     rest. That opens the whole column, not one cell (closing rule,
     `generic-language-features.md:211-216`).
   - Grid R: every guard position that reads a union-typed fact (input, stored field, related row,
     candidate row, view filter) admits it, since the type, not the position, was the refusal.
   - Content key: it extends the existing `tag:` with serde's adjacent-tagging pair, which the
     codecs already implement for the fixed key. It is one value of the union-encoding selector that
     `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant` and
     `story:union-tag-inline-with-fields` will add. `content:` is meaningful only under adjacent
     encoding, and that selector refuses it with the others. The design page records this so the
     three stories share one page.
5. **Second adopter.** (a) A payment request `method: {type: card | bank_transfer, …}` where a
   bank transfer is refused above a limit and a card is not. (b) A webhook envelope
   `{event, data}`, which is adjacently tagged with content member `data`.
6. **Cost.** One source format, the next one open (`ess/37` if no other decision allocates it
   first). One new key on unions (`content:`) and no new diagnostic codes. A new IR field on a union
   is left out when absent. A union guard needs no IR shape change, since a fact compared with a
   literal label is an existing `Compare`. `ess verify diff` needs a kind for a changed content key,
   which breaks callers and readers. The suite format pair is in the projections and suite stories.
7. **Alternatives.**
   - (a) Change nothing and keep the enum idiom: rejected under policy 1.
   - (b) A path through the tag, `kind.type == web_crawl`: rejected. It names the wire member, so a
     guard would change meaning when the tag is renamed, and it reads like a payload path.
   - (c) Compare the union as its variant label, as an enum: chosen. It is the requester's spelling.
   - For the content key, (d) a general encoding selector first: not needed. `content:` is
     independent of the selector and is recorded on the same page.

## Decisions

accept as proposed, for both spellings. The guard compares a union as its variant label, exactly as
an enum, which opens grid O's union column for comparison and membership. A union may declare
`content:` beside `tag:`, at `ess/37`. Follow-ups:

- `story:synthesis-and-the-interpreter-decide-a-guard-on-a-union-variant`
- `story:generated-targets-decide-a-guard-on-a-union-variant`
- `story:projections-use-a-unions-declared-content-key`
- `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key`

## Source

github:beyond10x/ess#637

