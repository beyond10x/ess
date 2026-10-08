---
format: aep.planning-md/3
id: story:refusal-above-held-state-branch-validates
kind: story
status: draft
title: An external branch can be declared to answer before held-state selection
tags:
- adopter-report
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
---
## Outcome

A command can declare that an `external:` branch answers before held-state and related-row
selection, so a model can require a caller to be authenticated before the stored row is read, and
synthesis injects that external cause first.

Design first: a page under `docs/design/` before code.

## Evidence

An adopter on ess 0.56.0 declares a revocation command in protocol order: input
`{token, client_id}`, entity `Token {client_id: String, state Active|Revoked}`:

```yaml
outcomes:
  - name: invalid-client
    external: client authentication failed
    error: demo.revoke.InvalidClient
  - name: invalid-token
    unknown_instance: true
    refuses: false
  - name: issued-to-another-client
    when_subject: {predicate: client_id != input.client_id}
    error: demo.revoke.TokenNotIssuedToClient
  - name: already-revoked
    when_subject_state: Revoked
  - name: revoked
    moves: demo.revoke.Token.revoke
```

Validation refuses it, by the ordering rule shipped for
https://github.com/beyond10x/ess/issues/486:

```
error[ESS-COMMAND-004]: `issued-to-another-client` is selected by the held state, which answers before the external branch `invalid-client` declared above it; where both guards hold, the declaration order and the precedence order disagree
  help: declare `issued-to-another-client` before `invalid-client`: the held state selects first in either order
```

The same refusal is reported for `already-revoked`. RFC 7009 § 2.1 and RFC 6749 §§ 3.2.1 and
4.1.3 authenticate the client before they look at the token or code. With held state always
first, a server that answers "not issued to you" to an unauthenticated caller conforms to the
model, and that answer says whether a token exists.

Asked for: a declaration that an external branch precedes held-state and related-row selection,
for example `precedes: held_state` on the branch or an ordered `precedence:` list. Related:
`story:external-branch-narrowed-by-subject-guard`.

## Acceptance

- The design page chooses the declaration and states how it composes with the #486 rule, the
  related-row guards and `unknown_instance:`.
- With it, the model above validates; without it, the #486 refusal is unchanged.
- Synthesis writes the `invalid-client` scenario on a row where `issued-to-another-client` and
  `already-revoked` would also hold, and the interpreted, Rust, Go and TypeScript targets all
  answer `invalid-client` there.
- It adds an authored key, so it ships with a source format.
