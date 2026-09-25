---
format: aep.planning-md/2
id: story:d2-constraint-has-a-home
kind: story
status: implemented
title: Gap register D-2 is written where it lives, not quoted from a missing file
summary: The linker constraint survives only as a sentence in a plan document; give it a durable home
owner: ess
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design
- confidence: cited
  path: docs/design/ess-model-driven-interpretation-design-v0.1.md
- confidence: inferred
  path: docs/plan/ess-roadmap.md
- confidence: cited
  path: docs/plan/ess-wave-6-structural-synthesis.md
- confidence: inferred
  path: docs/plan/ess-wave-7-closing-the-loop.md
revision: 7
---
# Story: gap register D-2 is written where it lives

## Outcome

D-2 — the linker never chooses; zero implementations for an obligation is an unsatisfied obligation,
two is an ambiguity error naming both, and selection among alternatives is `Realization` material —
is stated in a durable home rather than quoted from a file that is gone.

`docs/plan/ess-wave-6-structural-synthesis.md:73-76` cites `docs/plan/gap-register.md`, which does
not exist at `11fc669`. The constraint currently survives as that quotation and as a paraphrase in a
design page. A constraint whose home is missing is one the next reader re-litigates, which is what
this repository refuses everywhere else.

## Acceptance

D-2 is stated once, in a document that is not a plan file, and every place that quotes it links
there instead of restating it; no reference to `docs/plan/gap-register.md` remains that does not say
the file is gone.

## Scope

Confirmed by the implementor against the tree, 2026-09-11, across three correction rounds.

- `docs/design/linker-never-chooses.md` — **new, the durable home.** Its claims are deliberately
  normative rather than complete: it states what a document applying the rule must do, and claims
  nothing about what every file in the tree contains. Two adversary passes falsified four
  completeness claims before that was settled.
- `docs/plan/ess-wave-6-structural-synthesis.md:73-76` — cited, confirmed.
- `docs/design/ess-model-driven-interpretation-design-v0.1.md` — **the cited line numbers were all
  wrong.** The verbatim D-2 quote is at `:52-54`, not `:46-50`; the "file is gone" paragraph at
  `:77-81`, not `:73-76`; `:152` is blank, the relevant bullet being `:155-156`.
- `docs/plan/ess-roadmap.md:307` and `docs/plan/ess-wave-7-closing-the-loop.md:347` — both inferred,
  both exact.
- **`.gitignore:40` — the census missed it entirely.** A live reference to the deleted register that
  did not say it was gone, and the acceptance's third clause reaches it verbatim.
- **`crates/edge/ess-xtask/tests/d2_constraint_home.rs` — outside the documentation-only reading, and
  kept.** The story is about a constraint that rots without a check; dropping the check would keep
  the label and lose the guarantee. It asks `git ls-files --cached --others --exclude-standard`
  rather than walking the filesystem, so gitignored build trees cannot change its verdict.
- **`generated/rust/README.md` and `website/blog/2026-08-20-2316-structural-synthesis.md`** — one
  line each, outside `docs/`. The first was verified hand-written three ways: no `Artifact::new`
  emits it, the Rust emitter emits no README, and `PROJECTION_EXCLUSIONS` at
  `ess-xtask/src/main.rs:29` contains `"rust"`. The second names the home as inline code and not a
  link, because `docs/` is never published and a relative link would break `site-build`.
- Deliberately untouched: roughly 25 files under `crates/`, `examples/` and `generated/` that cite or
  restate D-2, including four full restatements. A linker stating the rule it enforces is the rule
  where it binds.
- `examples/gatepass-go-realization/linker.go:14` names a Go test defined nowhere in the repository,
  byte-identical at this story's base. Pre-existing; pinned by a case rather than fixed here.
