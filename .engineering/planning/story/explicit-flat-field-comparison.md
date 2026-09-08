---
format: aep.planning-md/1
id: story:explicit-flat-field-comparison
kind: story
status: archived
title: Represent a flat sibling field as an explicit comparison operand
relations:
- informed_by: story:review-expression-typechecking
revision: 3
---
## Observed consumer refusal
Published ESS 0.20.0 correctly refuses the authored numeric invariant `processed <= total` when both sibling struct fields are Integer: the current predicate parser classifies bare right-hand words as Text literals. Existing consumers previously admitted by older validation therefore fail current typechecking. The synthetic example expresses a numeric relation, not comparison to the text "total".

## Required clarification or contract
Provide or document an explicit authored syntax for a right-hand Fact operand that names a single-segment sibling field. Preserve the existing interpretation of unquoted bare enum/text values and dotted fact paths. If such a supported syntax already exists, return an executable minimal example and the exact supported release. Otherwise add a reviewed unambiguous form through the owning grammar, typechecking, canonical representation and projection/evaluation boundaries. Current parsing accepts only scalar comparison operands; dotted workarounds require a real declared nested field and cannot manufacture a self root.

Do not suggest deleting the invariant, changing the public data shape to add artificial nesting, weakening the new typechecker or relabelling an unknown comparison as verified. Include parse/serialize/reload and runtime evaluation evidence that the RHS is a fact lookup, with missing operands remaining unknown. Preserve old literal behavior. Synthetic tests should distinguish sibling numeric fields, quoted/bare text literals, malformed or undeclared references, type mismatch and a valid/invalid actual numeric pair.

## Ownership
This is a brain consumer compatibility request against the published release. ESS agents own investigation, implementation and release if required. The brain session will validate a published compatible result without implementing or releasing ESS. No private source, operator, customer or instance data belongs in this request or its fixtures.

## Withdrawn: consumer modeling correction
The request was filed prematurely. Published ESS 0.20.0 already supports a transparent newtype over a struct representation. Its invariant environment exposes that representation as `value`, permitting `value.processed <= value.total` with both operands resolved as numeric facts. The representation remains the existing flat object; `value` is an invariant pseudo-field, not a serialized wrapper property. The brain consumer now validates and compiles all nine intended cross-field constraints using this existing contract. No ESS implementation or release is requested. Preserve this original request and correction as evidence of the mistaken escalation.
