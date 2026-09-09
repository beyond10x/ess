// Admission is completed before the player creates any replay state.
// Admission for replay/1. Only declared Node payload owners have finite Number meaning.
// Original suite strings and unsigned metadata never pass through JSON.parse's Number conversion.
const fail = reason => { throw new Error(`Invalid coverage replay: ${reason}`) }
const require = (condition, reason) => { if (!condition) fail(reason) }
class NumberToken { constructor(raw) { this.raw = raw } }
const own = (value, key) => Object.hasOwn(value, key)
const object = value => {
  require(value !== null && typeof value === 'object' && !Array.isArray(value) && !(value instanceof NumberToken), 'expected object')
  return value
}
const array = value => { require(Array.isArray(value), 'expected array'); return value }
const text = value => { require(typeof value === 'string', 'expected string'); return value }
// Rust str::trim/split_whitespace use Unicode White_Space. JavaScript trim instead includes
// U+FEFF and excludes U+0085; using it would change predicate literal and ranking meaning.
const white = '[\\u0009-\\u000d\\u0020\\u0085\\u00a0\\u1680\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000]'
const trimEdges = new RegExp(`^${white}+|${white}+$`, 'gu')
const whiteRuns = new RegExp(`${white}+`, 'u')
const trim = value => value.replace(trimEdges, '')
const boolean = value => { require(typeof value === 'boolean', 'expected boolean'); return value }
const nullable = (value, check) => value === null ? null : check(value)
const closed = (value, required, optional = '') => {
  object(value)
  const keys = required.split(' ').filter(Boolean), allowed = new Set([...keys, ...optional.split(' ')])
  require(keys.every(key => own(value, key)) && Object.keys(value).every(key => allowed.has(key)), 'missing or unknown field')
  return value
}
const oneOf = (value, choices) => { require(choices.split(' ').includes(text(value)), 'unsupported vocabulary'); return value }
const unsigned = (value, maximum = 18446744073709551615n) => {
  require(value instanceof NumberToken && /^(0|[1-9][0-9]*)$/.test(value.raw), 'expected exact unsigned integer')
  const result = BigInt(value.raw)
  require(result <= maximum, 'unsigned integer out of range')
  return result
}
const unicode = value => {
  for (let i = 0; i < value.length; i += 1) {
    const unit = value.charCodeAt(i)
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = value.charCodeAt(++i)
      require(next >= 0xdc00 && next <= 0xdfff, 'unpaired Unicode surrogate')
    } else require(unit < 0xdc00 || unit > 0xdfff, 'unpaired Unicode surrogate')
  }
  return value
}
function parse(original) {
  text(original)
  let offset = 0
  const space = () => { while (offset < original.length && /[\x20\t\r\n]/.test(original[offset])) offset += 1 }
  const string = () => {
    const start = offset++
    while (offset < original.length) {
      const char = original[offset++]
      if (char === '"') return unicode(JSON.parse(original.slice(start, offset)))
      if (char === '\\') offset += 1
    }
    return fail('unterminated string')
  }
  const value = depth => {
    require(depth <= 128, 'JSON nesting exceeds 128')
    space()
    const char = original[offset]
    if (char === '"') return string()
    if (char === '{' || char === '[') {
      offset += 1; space()
      const mapping = char === '{', end = mapping ? '}' : ']', result = mapping ? Object.create(null) : []
      if (original[offset] === end) { offset += 1; return result }
      while (offset < original.length) {
        space()
        if (mapping) {
          require(original[offset] === '"', 'expected object key')
          const key = string(); space()
          require(!own(result, key), 'duplicate object key')
          require(original[offset++] === ':', 'expected colon')
          result[key] = value(depth + 1)
        } else result.push(value(depth + 1))
        space()
        const next = original[offset++]
        if (next === end) return result
        require(next === ',', 'expected separator')
      }
      return fail('unterminated container')
    }
    for (const [word, literal] of [['true', true], ['false', false], ['null', null]]) {
      if (original.startsWith(word, offset)) { offset += word.length; return literal }
    }
    const number = original.slice(offset).match(/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/)
    require(number !== null, 'invalid JSON token')
    offset += number[0].length
    return new NumberToken(number[0])
  }
  const result = value(0); space()
  require(offset === original.length, 'trailing JSON data')
  return result
}
// Unicode scalar ordering matches Rust's UTF-8 String Ord for well-formed strings.
const compare = (a, b) => {
  const left = [...a], right = [...b]
  for (let i = 0; i < Math.min(left.length, right.length); i += 1) {
    const difference = left[i].codePointAt(0) - right[i].codePointAt(0)
    if (difference !== 0) return difference
  }
  return left.length - right.length
}
const keys = value => Object.keys(value).sort(compare)
function equal(a, b) {
  if (a instanceof NumberToken || b instanceof NumberToken) return a instanceof NumberToken && b instanceof NumberToken && a.raw === b.raw
  if (a === b) return true
  if (a === null || b === null || typeof a !== 'object' || typeof b !== 'object') return false
  if (Array.isArray(a) !== Array.isArray(b)) return false
  const left = Array.isArray(a) ? a.map((_, i) => String(i)) : keys(a)
  const right = Array.isArray(b) ? b.map((_, i) => String(i)) : keys(b)
  return left.length === right.length && left.every((key, i) => key === right[i] && equal(a[key], b[key]))
}
const digest = async original => `sha256:${[...new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(original)))].map(byte => byte.toString(16).padStart(2, '0')).join('')}`
const sha = value => { require(/^sha256:[0-9a-f]{64}$/.test(text(value)), 'invalid source digest'); return value }
const modelDigest = value => { require(/^[0-9a-f]{64}$/.test(text(value)), 'invalid model digest'); return value }
const qualified = value => { require(/^[A-Za-z][A-Za-z0-9_-]*(\.[A-Za-z][A-Za-z0-9_-]*)*$/.test(text(value)), 'invalid qualified name'); return value }
const local = value => { qualified(value); require(!value.includes('.'), 'expected local name'); return value }
const kebab = value => { require(/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(text(value)), 'invalid kebab name'); return value }
const fact = value => { require(/^[A-Za-z][A-Za-z0-9_-]*(\.[A-Za-z0-9_-]+)*$/.test(text(value)), 'invalid fact path'); return value }
const sourceIdentity = value => {
  text(value)
  require(!/[\\:\x00-\x1f\x7f-\x9f]/.test(value) && value.split('/').every(part => part !== '' && part !== '.' && part !== '..'), 'invalid root-relative source identity')
  return value
}
const outcome = value => { closed(value, 'command outcome'); qualified(value.command); kebab(value.outcome); return value }
const referenceKinds = 'domain type entity command outcome event error view actor transition binding component'.split(' ')
function reference(value) {
  closed(value, 'kind name'); oneOf(value.kind, referenceKinds.join(' '))
  if (value.kind === 'outcome') outcome(value.name)
  else if (value.kind === 'transition') { closed(value.name, 'entity transition'); qualified(value.name.entity); local(value.name.transition) }
  else if (value.kind === 'binding' || value.kind === 'component') kebab(value.name)
  else qualified(value.name)
  return value
}
const referenceKey = value => value === null ? '' : `${String(referenceKinds.indexOf(value.kind)).padStart(2, '0')}:${typeof value.name === 'string' ? value.name : value.kind === 'outcome' ? `${value.name.command}/${value.name.outcome}` : `${value.name.entity}/${value.name.transition}`}`
function scenarioID(value) {
  const parts = text(value).split('/')
  const q = i => qualified(parts[i]), k = i => kebab(parts[i])
  switch (parts[1]) {
    case 'outcome': case 'authored': require(parts.length === 3, 'invalid scenario identity'); q(0); k(2); break
    case 'binding': require(parts.length === 3, 'invalid scenario identity'); k(0); oneOf(parts[2], 'delivery flow mapping on-failure'); break
    case 'state': require(parts.length === 5 && /^[A-Z][A-Za-z0-9]*$/.test(parts[2]), 'invalid state scenario identity'); q(0); oneOf(parts[3], 'accepts refuses'); q(4); break
    case 'transition': require(parts.length === 6 && parts[3] === 'by', 'invalid transition scenario identity'); q(0); local(parts[2]); q(4); k(5); break
    case 'invariant': require(parts.length === 5, 'invalid invariant identity'); q(0); oneOf(parts[2], 'after at'); q(3); if (parts[2] === 'after') k(4); else require(parts[4].length > 0, 'empty invariant suffix'); break
    default: fail('unsupported scenario identity')
  }
  return value
}
const ordered = (values, check, key = value => value) => {
  array(values).forEach(value => check(value))
  for (let i = 1; i < values.length; i += 1) require(compare(key(values[i - 1]), key(values[i])) < 0, 'inventory order or duplicate')
  return values
}
const suiteReference = value => {
  closed(value, 'version digest_profile digest')
  require(value.version === 'ess-conformance/5' && value.digest_profile === 'sha256-json-bytes/1', 'unsupported suite reference')
  sha(value.digest); return value
}
const referenceFor = suite => ({ version: 'ess-conformance/5', digest_profile: 'sha256-json-bytes/1', digest: suite.digest })
function node(value) {
  if (value instanceof NumberToken) { const result = Number(value.raw); require(Number.isFinite(result), 'non-finite Node number'); return result }
  if (Array.isArray(value)) return value.map(node)
  if (value !== null && typeof value === 'object') return Object.fromEntries(keys(value).map(key => [key, node(value[key])]))
  return value
}
function values(value) {
  const result = Object.create(null)
  for (const [key, field] of Object.entries(object(value))) {
    switch (object(field).kind) {
      case 'literal': closed(field, 'kind value'); result[key] = { kind: 'literal', value: node(field.value) }; break
      case 'instance': closed(field, 'kind instance'); kebab(field.instance); result[key] = field; break
      case 'observed': closed(field, 'kind event field'); qualified(field.event); text(field.field); result[key] = field; break
      default: fail('unsupported scenario value')
    }
  }
  return result
}
const combine = (kind, children) => children.length === 0 ? kind === 'all' : children.length === 1 ? children[0] : [kind, children]
const negate = inner => typeof inner === 'boolean' ? !inner : Array.isArray(inner) && inner[0] === 'not' ? inner[1] : ['not', inner]
const scalar = value => {
  require(typeof value === 'string' || typeof value === 'boolean' || value instanceof NumberToken, 'predicate operand must be scalar')
  if (typeof value !== 'string') return node(value)
  const trimmed = trim(value)
  if (trimmed.length >= 2 && ['"', "'"].includes(trimmed[0]) && trimmed.at(-1) === trimmed[0]) return trimmed.slice(1, -1)
  if (trimmed === 'true' || trimmed === 'false') return trimmed === 'true'
  // f64's decimal grammar, without JavaScript's hexadecimal/empty-string coercions.
  if (/^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/.test(trimmed) && Number.isFinite(Number(trimmed))) return Number(trimmed)
  return trimmed
}
const operand = value => {
  if (typeof value === 'string') {
    const trimmed = trim(value), numeric = /^[+-]?(?:(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?|inf(?:inity)?|nan)$/i.test(trimmed)
    if (!['"', "'"].includes(trimmed[0]) && trimmed.includes('.') && !numeric && /^[A-Za-z][A-Za-z0-9_-]*(\.[A-Za-z0-9_-]+)*$/.test(trimmed)) return ['fact', trimmed]
  }
  return ['literal', scalar(value)]
}
const operatorNames = { eq: '==', equals: '==', '==': '==', ne: '!=', not_equals: '!=', '!=': '!=', lt: '<', '<': '<', le: '<=', lte: '<=', '<=': '<=', gt: '>', '>': '>', ge: '>=', gte: '>=', '>=': '>=' }
const sequence = value => value === null ? [] : Array.isArray(value) ? value : [value]
function predicate(value, depth = 0) {
  require(depth <= 32, 'predicate nesting exceeds 32')
  const nested = item => predicate(item, depth + 1)
  if (typeof value === 'boolean') return value
  if (typeof value === 'string') {
    const expression = trim(value)
    if (expression === 'true' || expression === 'always') return true
    if (expression === 'false' || expression === 'never') return false
    if (expression.startsWith('not ')) return negate(nested(expression.slice(4)))
    const call = expression.match(/^(defined|exists|missing)(.*)$/s)
    if (call) {
      const rest = trim(call[2])
      if (rest.startsWith('(') && rest.endsWith(')')) {
        const defined = ['defined', fact(trim(rest.slice(1, -1)))]
        return call[1] === 'missing' ? negate(defined) : defined
      }
    }
    let quote = null
    for (let i = 0; i < expression.length; i += 1) {
      const char = expression[i]
      if (quote !== null) { if (char === quote) quote = null; continue }
      if (char === '"' || char === "'") { quote = char; continue }
      let op = expression.slice(i, i + 2)
      if (!['==', '!=', '<=', '>='].includes(op)) op = ['<', '>'].includes(char) ? char : null
      if (op !== null) {
        const left = fact(trim(expression.slice(0, i))), right = trim(expression.slice(i + op.length))
        require(right !== '', 'missing comparison operand')
        return ['compare', left, op, operand(right)]
      }
    }
    return ['truthy', fact(expression)]
  }
  if (Array.isArray(value)) return combine('all', value.map(nested))
  object(value)
  return combine('all', keys(value).map(key => {
    const item = value[key]
    if (['all', 'and', 'all_of', 'any', 'or', 'none', 'none_of_these'].includes(key)) {
      const result = combine(['all', 'and', 'all_of'].includes(key) ? 'all' : 'any', sequence(item).map(nested))
      return ['none', 'none_of_these'].includes(key) ? negate(result) : result
    }
    if (key === 'not') return negate(nested(item))
    if (key === 'forall' || key === 'exists') {
      closed(item, 'in as that'); fact(item.in); local(item.as)
      return [key, item.in, item.as, nested(item.that)]
    }
    fact(key)
    if (Array.isArray(item)) return ['any_of', key, item.map(scalar)]
    if (typeof item === 'boolean' || typeof item === 'string' || item instanceof NumberToken) return ['compare', key, '==', ['literal', scalar(item)]]
    object(item)
    return combine('all', keys(item).map(operator => {
      const argument = item[operator]
      if (own(operatorNames, operator)) return ['compare', key, operatorNames[operator], operand(argument)]
      if (['any_of', 'in', 'one_of', 'none_of', 'not_in'].includes(operator)) return [['none_of', 'not_in'].includes(operator) ? 'none_of' : 'any_of', key, sequence(argument).map(scalar)]
      if (operator === 'defined' || operator === 'exists') return boolean(argument) ? ['defined', key] : negate(['defined', key])
      if (operator === 'truthy') { node(argument); return ['truthy', key] }
      return fail('unsupported predicate operator')
    }))
  }))
}
function ranking(value) {
  return array(value).map(part => {
    const words = trim(text(part)).split(whiteRuns)
    require(words.length <= 2, 'invalid ranking')
    require(words[0] !== '', 'empty ranking field'); const direction = words[1] ?? 'asc'
    oneOf(direction, 'asc ascending desc descending')
    return [words[0], direction.startsWith('asc') ? 'asc' : 'desc']
  })
}
function expectation(value) {
  object(value)
  const result = { ...value }
  switch (value.expect) {
    case 'contains': case 'excludes': closed(value, 'expect fields'); result.fields = values(value.fields); break
    case 'satisfies': closed(value, 'expect predicate'); result.predicate = predicate(value.predicate); break
    case 'counts':
      closed(value, 'expect', 'at_least at_most')
      for (const key of ['at_least', 'at_most']) { result[key] = value[key] ?? null; nullable(result[key], unsigned) }
      break
    case 'ranked': closed(value, 'expect order_by'); result.order_by = ranking(value.order_by); break
    case 'at':
      closed(value, 'expect order_by position', 'fields'); result.order_by = ranking(value.order_by)
      oneOf(object(value.position).row, 'first last nth')
      closed(value.position, value.position.row === 'nth' ? 'row index' : 'row')
      if (value.position.row === 'nth') unsigned(value.position.index)
      result.fields = values(value.fields ?? {})
      break
    default: fail('unsupported view expectation')
  }
  return result
}
// A UUID in the one hyphenated form the specification publishes: 8-4-4-4-12 hexadecimal digits, in
// either case. urn:uuid: and brace-wrapped spellings are refused, because one value has one
// spelling. The same grammar as ess-gen's UUID_PATTERN, ess_primitives::facts::is_canonical_uuid
// and the Go runtime's canonicalUUID, checked from all four against one corpus
// (crates/specify/ess-primitives/tests/vectors/primitive-semantics.json).
const canonicalUUID = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/
// Base64 with padding, the standard alphabet only, as ess-gen's BASE64_PATTERN publishes it.
const paddedBase64 = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/
// Whether a value is one the declared primitive admits.
//
// A grammar, not merely a kind name. Every kind was admitted by name alone, so this adapter showed
// a replay a `uuid` field holding `x` that every schema the same specification publishes refuses —
// review finding F08, docs/design/review-primitive-semantics.md.
export const primitiveAdmits = (kind, value) => {
  switch (kind) {
    case 'string': case 'timestamp': case 'duration': return typeof value === 'string'
    case 'boolean': return typeof value === 'boolean'
    case 'integer': return typeof value === 'number' && Number.isInteger(value) && value >= -9223372036854775808 && value < 9223372036854775808
    case 'decimal': return typeof value === 'number' && Number.isFinite(value)
    case 'uuid': return typeof value === 'string' && canonicalUUID.test(value)
    case 'bytes': return typeof value === 'string' && paddedBase64.test(value)
    default: return false
  }
}
function shape(value) {
  return Object.fromEntries(Object.entries(object(value)).map(([key, leaf]) => {
    oneOf(object(leaf).holds, 'primitive enum list map union')
    closed(leaf, leaf.holds === 'primitive' ? 'holds kind' : leaf.holds === 'enum' ? 'holds variants' : 'holds', 'optional')
    if (leaf.holds === 'primitive') oneOf(leaf.kind, 'string boolean integer decimal timestamp duration uuid bytes')
    if (leaf.holds === 'enum') array(leaf.variants).forEach(text)
    const optional = own(leaf, 'optional') ? boolean(leaf.optional) : false
    return [key, { ...leaf, optional }]
  }))
}
// A step declares both what a field holds and what its declared type permits there, and until this
// ran nothing compared them. Only the fields the payload names: a shape is a claim about the
// declaration and a payload is a partial claim about values, so requiring every declared leaf to be
// present would refuse suites this repository already writes. `admission.rs`'s
// `payload_agrees_with_its_shape` applies the identical rule, so the two admitters cannot disagree
// about one document.
const payloadAgreesWithShape = (payload, declared) => {
  for (const [field, leaf] of Object.entries(declared)) {
    if (!own(payload, field) || payload[field] === null) continue
    const value = payload[field]
    const admitted = leaf.holds === 'primitive' ? primitiveAdmits(leaf.kind, value)
      : leaf.holds === 'enum' ? leaf.variants.includes(value)
      : leaf.holds === 'list' ? Array.isArray(value)
      : value !== null && typeof value === 'object' && !Array.isArray(value)
    require(admitted, 'payload value the step\'s own shape does not admit')
  }
}
const stepFields = {
  configure_external_outcome: ['force', ''], execute_command: ['command', 'actor input'], expect_outcome: ['outcome', ''],
  expect_error: ['error', 'fields'], expect_event: ['event', 'payload shape'], eventually_event: ['event', 'payload shape'],
  expect_no_event: ['event', ''], redeliver_event: ['event', ''], capture_instance: ['instance entity event field', ''],
  expect_invocation: ['binding command', 'input'], query_view: ['view', 'params'], expect_view: ['view expectation', ''],
  eventually_view: ['view expectation', 'params'], mark_instant: ['instant', ''], expect_not_before: ['instant elapsed', ''],
  expect_within: ['instant elapsed', ''], expect_quiet: ['event instant elapsed', ''], expect_halt: ['view after', 'params'], eventually_halt: ['view after', 'params'],
}
function step(value) {
  const fields = stepFields[object(value).step]
  require(fields !== undefined && own(stepFields, value.step), 'unsupported suite step')
  closed(value, `step ${fields[0]}`, fields[1])
  const result = { ...value }
  for (const key of fields[1].split(' ')) {
    if (key === '') continue
    if (!own(result, key)) result[key] = key === 'actor' ? null : {}
  }
  for (const [key, field] of Object.entries(result)) {
    switch (key) {
      case 'step': break
      case 'actor': nullable(field, qualified); break
      case 'input': case 'params': result[key] = values(field); break
      case 'fields': case 'payload': object(field); result[key] = node(field); break
      case 'shape': result[key] = shape(field); break
      case 'expectation': result[key] = expectation(field); break
      case 'force': case 'outcome': outcome(field); break
      case 'elapsed': unsigned(field, 4294967295n); break
      case 'after': unsigned(field); break
      case 'instance': case 'instant': case 'binding': kebab(field); break
      case 'field': text(field); break
      default: qualified(field)
    }
  }
  if (own(result, 'payload') && own(result, 'shape')) payloadAgreesWithShape(result.payload, result.shape)
  return result
}
const includes = (selection, origin) => selection.origins === 'generated_and_authored' || selection.origins === origin
const needs = (value, required) => { ordered(value, reference, referenceKey); require(required === (value.length > 0), 'invalid component proof') }
function selection(value, provenance) {
  closed(value, 'scope origins filter'); oneOf(value.origins, 'generated authored generated_and_authored')
  oneOf(object(value.scope).kind, 'system component')
  closed(value.scope, value.scope.kind === 'system' ? 'kind' : 'kind component')
  if (value.scope.kind === 'component') kebab(value.scope.component)
  require((value.scope.component ?? null) === (provenance.component ?? null), 'scope differs from provenance')
  const filter = object(value.filter)
  oneOf(filter.kind, 'all explicit'); closed(filter, filter.kind === 'all' ? 'kind' : 'kind ids parent')
  if (filter.kind === 'explicit') { ordered(filter.ids, scenarioID); suiteReference(filter.parent) }
}
const refusalKey = value => [value.origin, value.source ?? '', referenceKey(value.subject), value.scenario ?? '', value.code, value.effect, value.retained?.origin ?? '', value.retained?.source ?? '', value.scope, value.needs.map(referenceKey).join('\0'), value.message]
function inventory(suite) {
  const c = closed(suite.coverage, 'selection knowledge generated authored outside refused authored_sources counts')
  selection(c.selection, suite.provenance); oneOf(c.knowledge, 'unknown complete_inventory')
  closed(c.counts, 'generated authored outside refused')
  for (const key of ['generated', 'authored', 'outside', 'refused']) require(unsigned(c.counts[key]) === BigInt(array(c[key]).length), 'coverage count mismatch')
  const survivors = new Map(), owners = new Map()
  for (const origin of ['generated', 'authored']) for (const id of ordered(c[origin], scenarioID)) {
    require(includes(c.selection, origin) && !survivors.has(id), 'selected origins overlap or are excluded')
    survivors.set(id, origin)
  }
  require(equal([...survivors.keys()].sort(compare), keys(suite.scenarios)), 'selected origin union differs from scenarios')
  if (c.selection.filter.kind === 'explicit') require(equal(c.selection.filter.ids, keys(suite.scenarios)), 'explicit IDs differ from scenarios')
  ordered(c.outside, outside => {
    closed(outside, 'scenario origin reason needs'); scenarioID(outside.scenario); oneOf(outside.origin, 'generated authored')
    oneOf(outside.reason, 'other_component origin_selection selection_filter')
    needs(outside.needs, outside.reason === 'other_component')
    const included = includes(c.selection, outside.origin)
    require(outside.reason === 'origin_selection' ? !included : included, 'invalid outside origin')
    if (outside.reason === 'other_component') require(c.selection.scope.kind === 'component', 'outside component in system scope')
    if (outside.reason === 'selection_filter') require(c.selection.filter.kind === 'explicit', 'outside selection without filter')
    require(!survivors.has(outside.scenario), 'outside survivor overlaps')
    survivors.set(outside.scenario, outside.origin)
  }, value => value.scenario)
  for (const [identity, source] of Object.entries(object(c.authored_sources))) {
    sourceIdentity(identity); closed(source, 'digest scenario disposition'); sha(source.digest); nullable(source.scenario, scenarioID); oneOf(source.disposition, 'accepted refused')
    if (source.disposition === 'accepted') {
      require(survivors.get(source.scenario) === 'authored' && !owners.has(source.scenario), 'invalid or duplicate authored owner')
      owners.set(source.scenario, identity)
    }
  }
  for (const [id, origin] of survivors) require(origin !== 'authored' || owners.has(id), 'missing authored source')
  const refusedSources = new Set()
  let previous = null
  for (const r of c.refused) {
    closed(r, 'origin scenario subject source code message effect retained scope needs')
    oneOf(r.origin, 'authored generated'); nullable(r.scenario, scenarioID); nullable(r.subject, reference); nullable(r.source, sourceIdentity)
    require(trim(text(r.message)) !== '', 'empty refusal cause'); text(r.code)
    oneOf(r.effect, 'candidate_not_emitted check_not_emitted'); oneOf(r.scope, 'in_scope outside_component outside_origin')
    if (r.origin === 'generated') {
      const number = Array.from({ length: 14 }, (_, i) => i + 1).find(i => r.code === `ESS-SYNTH-${String(i).padStart(3, '0')}`)
      require(number !== undefined && r.subject !== null && r.source === null, 'invalid generated refusal')
      require(r.effect === ([5, 11, 12, 14].includes(number) ? 'check_not_emitted' : 'candidate_not_emitted'), 'refusal effect differs from code')
    } else {
      require(Array.from({ length: 35 }, (_, i) => `ESS-AUTHOR-${String(i + 1).padStart(3, '0')}`).includes(r.code) && r.effect === 'candidate_not_emitted', 'invalid authored refusal')
      const source = c.authored_sources[r.source]
      require(r.source !== null && source !== undefined && source.disposition === 'refused' && source.scenario === r.scenario, 'refusal source disagrees')
      refusedSources.add(r.source)
    }
    const retained = survivors.has(r.scenario) ? { origin: survivors.get(r.scenario), source: owners.get(r.scenario) ?? null } : null
    if (r.retained !== null) closed(r.retained, 'origin source')
    require(equal(retained, r.retained), 'refusal survivor identity differs')
    needs(r.needs, r.scope === 'outside_component')
    require(r.scope === 'outside_origin' ? !includes(c.selection, r.origin) : includes(c.selection, r.origin), 'refusal origin scope differs')
    if (r.scope === 'outside_component') require(c.selection.scope.kind === 'component', 'invalid refusal component scope')
    const current = refusalKey(r)
    if (previous !== null) {
      const difference = current.map((key, index) => compare(previous[index], key)).find(order => order !== 0) ?? 0
      require(difference <= 0, 'refusal ordering differs')
    }
    previous = current
  }
  for (const [identity, source] of Object.entries(c.authored_sources)) require(source.disposition !== 'refused' || refusedSources.has(identity), 'missing source refusal')
}
export async function admitSuite(original) {
  const document = closed(parse(original), 'provenance scenarios coverage')
  const p = closed(document.provenance, 'suite_version system specification_version spec_digest contract_digest', 'component')
  require(p.suite_version === 'ess-conformance/5', 'replay requires suite/5')
  text(p.system); text(p.specification_version)
  modelDigest(p.spec_digest); modelDigest(p.contract_digest); if (own(p, 'component')) nullable(p.component, text)
  const meaning = Object.create(null)
  for (const [id, scenario] of Object.entries(object(document.scenarios))) {
    scenarioID(id); closed(scenario, 'purpose steps source')
    const purpose = text(scenario.purpose)
    require(trim(purpose) !== '' && [...purpose].length <= 200 && !/[\x00-\x1f\x7f-\x9f]/.test(purpose), 'invalid scenario purpose')
    array(scenario.source).forEach(reference)
    meaning[id] = { purpose, steps: array(scenario.steps).map(step), source: [...new Set(scenario.source.map(referenceKey))].sort(compare) }
  }
  inventory(document)
  return { original, document, meaning, digest: await digest(original) }
}
function parentPair(child, parent) {
  const current = child.document.coverage, previous = parent.document.coverage, filter = current.selection.filter
  require(filter.kind === 'explicit' && equal(filter.parent, referenceFor(parent)), 'parent original digest mismatch')
  require(equal({ ...child.document.provenance, component: child.document.provenance.component ?? null }, { ...parent.document.provenance, component: parent.document.provenance.component ?? null }), 'parent provenance mismatch')
  const expected = { ...previous, selection: { ...previous.selection, filter }, outside: [...previous.outside], counts: { ...previous.counts } }
  for (const origin of ['generated', 'authored']) {
    expected[origin] = []
    for (const id of previous[origin]) {
      if (own(child.meaning, id)) expected[origin].push(id)
      else expected.outside.push({ scenario: id, origin, reason: 'selection_filter', needs: [] })
    }
    expected.counts[origin] = new NumberToken(String(expected[origin].length))
  }
  expected.outside.sort((a, b) => compare(a.scenario, b.scenario)); expected.counts.outside = new NumberToken(String(expected.outside.length))
  require(equal(expected, current), 'child changed inherited coverage inventory')
  for (const id of keys(child.meaning)) require(own(parent.meaning, id) && equal(child.meaning[id], parent.meaning[id]), 'child changed full scenario definition or dependencies')
}
async function input(value) {
  closed(value, 'format suite_json parent_suites'); require(value.format === 'ess-conformance-input/1', 'unsupported suite input format')
  const selected = await admitSuite(text(value.suite_json)), seen = new Set([selected.digest])
  let child = selected
  // Lineage is a flat sequence of original strings, with no semantic generation limit.
  for (const original of array(value.parent_suites)) {
    const parent = await admitSuite(text(original))
    require(!seen.has(parent.digest), 'repeated parent original')
    seen.add(parent.digest); parentPair(child, parent); child = parent
  }
  require(child.document.coverage.selection.filter.kind === 'all', 'missing final parent')
  return selected
}
function model(value) {
  closed(value, 'system version spec_digest contract_digest entities commands views actors bindings')
  for (const key of ['system', 'version']) text(value[key])
  modelDigest(value.spec_digest); modelDigest(value.contract_digest)
  for (const entity of array(value.entities)) {
    closed(entity, 'name display identity initial states terminal fields')
    for (const key of ['name', 'display', 'identity', 'initial']) text(entity[key])
    for (const key of ['states', 'terminal', 'fields']) array(entity[key]).forEach(text)
  }
  for (const command of array(value.commands)) {
    closed(command, 'name display outcomes'); text(command.name); text(command.display)
    for (const outcome of array(command.outcomes)) {
      closed(outcome, 'name refuses subject emits sets'); text(outcome.name); boolean(outcome.refuses); array(outcome.emits).forEach(text)
      if (outcome.subject !== null) {
        const subject = closed(outcome.subject, 'entity kind transition from to')
        text(subject.entity); oneOf(subject.kind, 'creates updates moves'); nullable(subject.transition, text); nullable(subject.to, text); array(subject.from).forEach(text)
      }
      for (const assignment of array(outcome.sets)) { closed(assignment, 'target from'); text(assignment.target); nullable(assignment.from, text) }
    }
  }
  for (const view of array(value.views)) {
    closed(view, 'name display entity consistency filter fields params')
    for (const key of ['name', 'display', 'entity']) text(view[key])
    oneOf(view.consistency, 'read_your_writes eventual'); nullable(view.filter, text)
    array(view.fields).forEach(text); array(view.params).forEach(text)
  }
  for (const actor of array(value.actors)) { closed(actor, 'name display may'); text(actor.name); text(actor.display); array(actor.may).forEach(text) }
  for (const binding of array(value.bindings)) {
    closed(binding, 'name event command delivery failure')
    for (const key of ['name', 'event', 'command']) text(binding[key])
    oneOf(binding.delivery, 'at_least_once'); oneOf(binding.failure, 'retry drop escalate')
  }
  return value
}
export async function admitReplay(original) {
  const replay = closed(parse(original), 'format model suite input')
  require(replay.format === 'ess-conformance-replay/1', 'unsupported replay format')
  model(replay.model); suiteReference(replay.suite)
  const selected = await input(replay.input), p = selected.document.provenance, projection = replay.model
  require(equal(replay.suite, referenceFor(selected)), 'selected original reference mismatch')
  require(projection.system === p.system && projection.version === p.specification_version && projection.spec_digest === p.spec_digest && projection.contract_digest === p.contract_digest, 'model differs from selected suite identity')
  const coverage = selected.document.coverage, selection = coverage.selection
  const scope = selection.scope.kind === 'component' ? `component ${selection.scope.component}` : 'system'
  const inScope = coverage.refused.filter(refusal => refusal.scope === 'in_scope').length
  const state = coverage.knowledge === 'unknown' ? 'unknown inventory' : inScope > 0 ? 'incomplete selection' : 'complete declared inventory'
  const refusals = coverage.refused.map(refusal => `${refusal.code} [${refusal.scope}] ${refusal.source ?? referenceKey(refusal.subject)}: ${refusal.message}`)
  const description = [`Coverage: ${scope}; ${selection.origins}; ${selection.filter.kind}; ${coverage.knowledge}; ${state}; ${keys(selected.meaning).length} selected; ${coverage.outside.length} outside; ${coverage.refused.length} refusal occurrences (${inScope} in scope).`, ...refusals, 'Replay illustrates the selected declarations; it is not an execution or a coverage audit.'].join('\n')
  // The player gets only declared execution fields; exact metadata stays in the admitted record.
  const scenarios = Object.fromEntries(keys(selected.document.scenarios).map(id => [id, { ...selected.document.scenarios[id], steps: selected.document.scenarios[id].steps.map(value => {
    const result = { ...value }
    for (const key of ['input', 'params']) if (own(result, key)) result[key] = values(result[key])
    for (const key of ['payload', 'fields']) if (own(result, key)) result[key] = node(result[key])
    return result
  }) }]))
  return { model: projection, suite: { provenance: p, scenarios }, description }
}
