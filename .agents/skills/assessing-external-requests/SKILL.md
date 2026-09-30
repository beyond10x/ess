---
name: assessing-external-requests
description: >-
  Assess a change an adopter asks ESS to make — a GitHub issue, a message from another session, a
  review of a downstream specification — before any story is written around it. Use when a request
  asks ESS to accept a new construct, key or value in an authored document, to emit something new in
  a versioned format or generated code, to stop refusing something, or to change what a diagnostic
  means. Not for defects in ESS's own tests or tooling that no authored document can reach.
---

# Assessing an external request before adopting it

An adopter's request is evidence of a need. It is not a design. The syntax the adopter proposes was
written to unblock one specification, by someone who sees one corner of ESS; adopting it as written
is how ESS ends up with two spellings for one idea, a construct that works in bindings but not in
guards, and a format bump per request. Every key ESS accepts is permanent: it must be read, diffed,
generated, synthesized and documented on every target for as long as the format lives.

So a request passes this review before `aep plan artifact new story` is run for it. The review's
answer is recorded in the story (or in the artifact that records a decline) — never only in a chat.

## The review: seven questions, answered with citations

Answer each in the story's `## Fit review` section. Every answer cites a file and line, a document,
a command's output, or says "I don't know". An answer with no citation is not an answer.

1. **What is the need, apart from the proposed syntax?** Restate it as a domain fact the adopter
   cannot express or a behaviour ESS gets wrong, in brand-free words, with a minimal reproduction
   (an ESS example or a new minimal specification — never the adopter's files copied in). Write the
   requester's proposed syntax separately, labelled as theirs.
2. **Which class is it?**
   - *defect* — ESS contradicts its own documented semantics (cite the page or the code comment);
   - *gap* — a real domain fact cannot be expressed or checked at all;
   - *convenience* — it can be expressed today another way, just longer;
   - *local policy* — one adopter's convention, not a fact about the domain.
   Only a defect or a gap justifies new authored surface. A convenience is answered with the idiom;
   a local policy belongs in the adopter's specification or tooling.
3. **Can it already be expressed?** Search the language before designing: `docs/design/`, the
   reference pages under `website/docs/reference/`, `ess specify validate` on a minimal attempt, and
   the existing constructs a sibling concept uses. If an idiom exists, the answer is the idiom — show
   it running.
4. **Does the design fit what is already there?** Check each:
   - it reuses existing vocabulary, key shapes and naming (the same concept is spelled the same way
     in guards, bindings, views, outcomes and formats);
   - it composes with the constructs it will meet: guards (`when:`, `when_subject`, `when_related`),
     bindings (cause, mapping, delivery, failure), views, relations, outcomes;
   - it is general across siblings: a capability added to one construct is added to the siblings
     where the same question arises, or the refusal names why not;
   - every target handles it or refuses it by name: Rust, Go, TypeScript, web, the interpreter,
     `ess verify diff`, the entity runtime, generated docs, synthesis and authoring.
5. **Would a second, unrelated adopter need it?** Write that second example, brand-free. If you
   cannot, it is probably local policy.
6. **What does it cost?** Format version bumps, new keywords, new diagnostics, a migration for
   existing documents, new diff classifications, generated-API changes (breaking for whom).
   Prefer the design that adds no surface, then the one that extends an existing construct, then a
   new construct.
7. **What else was considered?** At least two designs, one of them "change nothing". Say why the
   chosen one wins, and why the requester's proposal was taken, changed or refused.

## The decision

One of four, written into the story's `## Decisions` section:

| Decision | When | What gets recorded |
|---|---|---|
| **accept as proposed** | the requester's design already passes question 4 unchanged | the fit review; the story as usual |
| **accept, redesigned** | the need is real but the proposed shape does not fit | the fit review, the chosen design, and what changed from the request and why |
| **decline, with the idiom** | a convenience or local policy, or already expressible | a `specification` artifact stating the idiom, with the command output showing it works; reply to the requester with it |
| **defer** | the need is real but the design needs a decision nobody has made | a `decision-blocker` with `blocks:` to the story, naming the question |

Tell the requester which of the four it was and why, in one message, before the work starts.

## Red flags that stop adoption

Any of these sends the request back to question 4:

- the proposed key names a concept ESS already spells differently elsewhere;
- the change works in one construct and leaves a sibling construct unable to say the same thing;
- it adds a format version for something a nearby, unreleased bump could carry;
- a target would silently ignore the new construct;
- the request's reproduction is the adopter's document rather than a minimal one;
- the justification is urgency ("blocks our next wave") rather than a domain fact.

Urgency is a reason to schedule a well-designed change soon. It is not a reason to adopt a design.

## In a wave

A story that came from an external request cannot be dispatched without a `## Fit review` and a
`## Decisions` section. The wave's stage-1 proposal lists, per story, which of the four decisions it
carries; a plan critic that finds either section missing returns `needs-revision`.
