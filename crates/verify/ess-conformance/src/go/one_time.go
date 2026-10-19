// Closed suite/34–35 authority. Policy preparation does not enable Run's newer vocabulary.
type oneTimeConstraints struct {
	Alphabet   *string           `json:"alphabet"`
	Prefix     *string           `json:"prefix"`
	Invariants []json.RawMessage `json:"invariants"`
	parsed     []predicate
}
type oneTimeResponse struct {
	Fields       []accessorField                 `json:"fields"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
	Constraints  map[string]oneTimeConstraints   `json:"constraints"`
	authority    json.RawMessage
}

// Executable field projections omit naming metadata; retain it for authority accounting.
func (response oneTimeResponse) MarshalJSON() ([]byte, error) {
	if response.authority != nil {
		return response.authority, nil
	}
	type wire oneTimeResponse
	return json.Marshal(wire(response))
}

type oneTimeOrigin struct {
	Command  string          `json:"command"`
	Outcome  OutcomeRef      `json:"outcome"`
	Response oneTimeResponse `json:"response"`
	Fields   []string        `json:"fields"`
}
type oneTimeEventAuthority struct {
	Event        string `json:"event"`
	WithinMillis uint64 `json:"within_ms"`
}
type oneTimeEventWindow struct {
	Event        string `json:"event"`
	AfterStep    uint64 `json:"after_step"`
	WithinMillis uint64 `json:"within_ms"`
}
type oneTimeTrace struct {
	Origins         []oneTimeOrigin         `json:"origins"`
	RequiredOrigins []OutcomeRef            `json:"required_origins"`
	Events          []oneTimeEventAuthority `json:"events"`
	EventWindows    []oneTimeEventWindow    `json:"event_windows"`
}

func oneTimeRequiredString(ty string, declarations map[string]selectionDeclaration) bool {
	seen := map[string]bool{}
	for {
		ty = strings.TrimSpace(ty)
		if ty == "String" {
			return true
		}
		if seen[ty] {
			return false
		}
		seen[ty] = true
		body, ok := declarations[ty]
		if !ok || body.Kind != "newtype" {
			return false
		}
		ty = body.Of
	}
}

// Reuse the existing predicate parser, including its aliases, with only the source-declared
// String environment: value is text and value.count is a Unicode scalar count.
func oneTimePredicateNode(value any) any {
	switch value := value.(type) {
	case string:
		trimmed := strings.TrimSpace(value)
		switch trimmed {
		case "true", "always":
			return true
		case "false", "never":
			return false
		}
		if rest, ok := strings.CutPrefix(trimmed, "not "); ok {
			return map[string]any{"not": oneTimePredicateNode(rest)}
		}
		for _, function := range []string{"defined", "exists", "missing"} {
			if rest, ok := strings.CutPrefix(trimmed, function); ok {
				rest = strings.TrimSpace(rest)
				if strings.HasPrefix(rest, "(") && strings.HasSuffix(rest, ")") {
					return map[string]any{strings.TrimSpace(rest[1 : len(rest)-1]): map[string]any{"defined": function != "missing"}}
				}
			}
		}
		if left, op, right, ok := splitPredicateComparison(trimmed); ok {
			right = strings.TrimSpace(right)
			if right == "null" || right == "Null" || right == "NULL" || right == "~" {
				return nil
			}
			if !strings.HasPrefix(right, "\"") && !strings.HasPrefix(right, "'") {
				for _, token := range strings.Fields(right) {
					if token == "&&" || token == "||" {
						return nil
					}
				}
			}
			return map[string]any{strings.TrimSpace(left): map[string]any{op: oneTimeNumericOperand(right)}}
		}
		return value
	case []any:
		out := make([]any, 0, len(value))
		for _, child := range value {
			out = append(out, oneTimePredicateNode(child))
		}
		return out
	case map[string]any:
		keys := make([]string, 0, len(value))
		for key := range value {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		entries := make([]any, 0, len(keys))
		for _, key := range keys {
			child := value[key]
			out := map[string]any{}
			switch key {
			case "none", "none_of_these":
				out["not"] = map[string]any{"any": oneTimePredicateNode(meaningSequence(child))}
			case "all", "and", "all_of", "any", "or":
				out[key] = oneTimePredicateNode(meaningSequence(child))
			case "not":
				out[key] = oneTimePredicateNode(child)
			default:
				switch body := child.(type) {
				case []any:
					out[key] = map[string]any{"any_of": body}
				case map[string]any:
					operators := copyObject(body)
					for op, raw := range body {
						if _, ok := compareSpellings[op]; ok {
							if text, ok := raw.(string); ok {
								operators[op] = oneTimeNumericOperand(text)
							}
						}
					}
					out[key] = operators
				case string:
					literal := oneTimeNumericOperand(strings.TrimSpace(body))
					if text, ok := literal.(string); ok {
						parsed := parseLiteral(text)
						if text, ok := parsed.(string); ok {
							literal = "\"" + text + "\""
						} else {
							literal = parsed
						}
					}
					out[key] = map[string]any{"eq": literal}
				default:
					out[key] = child
				}
			}
			entries = append(entries, out)
		}
		if len(entries) == 1 {
			return entries[0]
		}
		return entries
	default:
		return value
	}
}

func oneTimeNumericOperand(raw string) any {
	trimmed := strings.TrimSpace(raw)
	if binary, ok := parseDecimalLiteral(trimmed); ok {
		// Canonicalize authored numeric text before it reaches the payload-number helper's
		// token budget. A long spelling of1 or an underflowing exponent is still admitted.
		if index := strings.IndexAny(trimmed, "eE"); index >= 0 {
			exponent, err := strconv.Atoi(trimmed[index+1:])
			if err != nil || exponent < -512 || exponent > 512 {
				return binary
			}
		}
		if exact, ok := new(big.Rat).SetString(trimmed); ok && exact.IsInt() && decimalDigitsFit(trimmed) {
			return json.Number(exact.Num().String())
		}
		return binary
	}
	return raw
}

// Match Node's JSON read door before parsing predicates: JSON integers within64 bits are
// exact; fractional/exponent and larger tokens arrive through binary64. Compact expression
// literals are parsed afterward and retain the domain's larger exact-integer range.
func oneTimeSourceNumbers(raw any) any {
	switch value := raw.(type) {
	case json.Number:
		if string(value) == "-0" {
			return math.Copysign(0, -1)
		}
		if !strings.ContainsAny(string(value), ".eE") {
			if _, err := strconv.ParseInt(string(value), 10, 64); err == nil {
				return value
			}
			if _, err := strconv.ParseUint(string(value), 10, 64); err == nil {
				return value
			}
		}
		if number, err := value.Float64(); err == nil {
			return number
		}
	case []any:
		out := []any{}
		for _, child := range value {
			out = append(out, oneTimeSourceNumbers(child))
		}
		return out
	case map[string]any:
		out := map[string]any{}
		for key, child := range value {
			out[key] = oneTimeSourceNumbers(child)
		}
		return out
	}
	return raw
}

// Number's existing serializer keeps binary64 JSON spelling unless that would lose an
// exact integer. Its finite formatting uses fixed exponents[-5,15], otherwise scientific.
func oneTimeLiteralWire(value Node) Node {
	number, ok := numberValue(value)
	if !ok {
		return value
	}
	binary, _ := number.Float64()
	if raw, ok := value.(float64); ok {
		binary = raw
	}
	carried, _ := binaryValue(binary)
	if number.IsInt() && carried.Cmp(number) != 0 {
		return json.Number(number.Num().String())
	}
	scientific := strconv.FormatFloat(binary, 'e', -1, 64)
	mantissa, exponentText, _ := strings.Cut(scientific, "e")
	exponent, _ := strconv.Atoi(exponentText)
	if exponent >= -5 && exponent <= 15 {
		fixed := strconv.FormatFloat(binary, 'f', -1, 64)
		if !strings.Contains(fixed, ".") {
			fixed += ".0"
		}
		return json.Number(fixed)
	}
	sign := "+"
	if exponent < 0 {
		sign = "-"
		exponent = -exponent
	}
	return json.Number(mantissa + "e" + sign + strconv.Itoa(exponent))
}

func oneTimeSimplify(p predicate) predicate {
	switch p.kind {
	case "all", "any":
		for i := range p.children {
			p.children[i] = oneTimeSimplify(p.children[i])
		}
		if len(p.children) == 0 {
			if p.kind == "all" {
				return predicate{kind: "always"}
			}
			return predicate{kind: "never"}
		}
		if len(p.children) == 1 {
			return p.children[0]
		}
	case "not":
		child := oneTimeSimplify(*p.body)
		switch child.kind {
		case "always":
			return predicate{kind: "never"}
		case "never":
			return predicate{kind: "always"}
		case "not":
			return *child.body
		}
		p.body = &child
	}
	return p
}

func oneTimeOperandText(value operand) string {
	if value.isFact {
		return value.path
	}
	if text, ok := value.literal.(string); ok {
		quote := text == "" || strings.Contains(text, ".") || text == "null" || text == "Null" || text == "NULL" || text == "~"
		for _, token := range strings.Fields(text) {
			if token == "&&" || token == "||" {
				quote = true
			}
		}
		if quote {
			return strconv.Quote(text)
		}
		return text
	}
	if number, ok := numberValue(value.literal); ok {
		if number.IsInt() {
			return number.Num().String()
		}
		return strings.TrimRight(strings.TrimRight(number.FloatString(400), "0"), ".")
	}
	return render(value.literal)
}

// Predicate serialization follows the native typed DTO, rather than counting the source alias.
func oneTimePredicateWire(p predicate) any {
	switch p.kind {
	case "always":
		return true
	case "never":
		return false
	case "all", "any":
		children := []any{}
		for _, child := range p.children {
			children = append(children, oneTimePredicateWire(child))
		}
		return map[string]any{p.kind: children}
	case "not":
		return map[string]any{"not": oneTimePredicateWire(*p.body)}
	case "truthy":
		return p.path
	case "defined":
		return "defined(" + p.path + ")"
	case "any_of", "none_of":
		values := []Node{}
		for _, value := range p.values {
			values = append(values, oneTimeLiteralWire(value))
		}
		return map[string]any{p.path: map[string]any{p.kind: values}}
	case "starts_with", "ends_with", "contains":
		return map[string]any{p.path: map[string]any{p.kind: p.right.literal}}
	case "equals_ignore_case", "in_ignore_case":
		var value any = p.values
		if p.kind == "equals_ignore_case" && len(p.values) == 1 {
			value = p.values[0]
		}
		return map[string]any{p.path: map[string]any{p.kind: value}}
	case "compare":
		compact := oneTimeOperandText(p.left) + " " + p.op + " " + oneTimeOperandText(p.right)
		if text, ok := p.right.literal.(string); p.left.isFact && ok {
			parsed, err := fromNode(compact)
			if err != nil || !reflect.DeepEqual(oneTimeSimplify(parsed), p) {
				scalar := text
				if !reflect.DeepEqual(parseOperand(text), operand{literal: text}) {
					scalar = "\"" + text + "\""
				}
				return map[string]any{p.left.path: map[string]any{p.op: scalar}}
			}
		}
		return compact
	}
	return nil
}

func oneTimeCanonicalField(raw any) any {
	field := raw.(map[string]any)
	ty, _, _ := accessorType(field["type"].(string), 0)
	result := map[string]any{"name": field["name"], "type": ty}
	naming := field
	if nested, ok := field["naming"].(map[string]any); ok {
		naming = nested
	}
	for _, key := range []string{"wire", "display", "summary", "code"} {
		if naming[key] != nil {
			result[key] = naming[key]
		}
	}
	if field["presence"] != nil {
		result["presence"] = field["presence"]
	}
	return result
}

func oneTimeCanonicalFields(raw any) []any {
	fields := []any{}
	for _, field := range raw.([]any) {
		fields = append(fields, oneTimeCanonicalField(field))
	}
	return fields
}

func oneTimeCanonicalDeclarations(raw any) map[string]any {
	result := map[string]any{}
	for name, raw := range raw.(map[string]any) {
		body := raw.(map[string]any)
		declaration := map[string]any{"kind": body["kind"]}
		switch body["kind"] {
		case "newtype":
			declaration["of"], _, _ = accessorType(body["of"].(string), 0)
		case "struct":
			declaration["fields"] = oneTimeCanonicalFields(body["fields"])
		case "enum":
			declaration["variants"] = body["variants"]
		case "union":
			declaration["tag"] = body["tag"]
			variants := map[string]any{}
			for name, raw := range body["variants"].(map[string]any) {
				// A unit variant (ess/22) is null: it names no type.
				if raw == nil {
					variants[name] = nil
					continue
				}
				variants[name], _, _ = accessorType(raw.(string), 0)
			}
			declaration["variants"] = variants
		}
		result[name] = declaration
	}
	return result
}

// Normalize only validated type and field spellings before the executable shape checker.
// Unknown declaration keys remain visible to the closed checker.
func oneTimeShapeInput(body map[string]any) (map[string]any, error) {
	fields := func(raw any) ([]any, error) {
		items, err := array(raw)
		if err != nil {
			return nil, err
		}
		out := []any{}
		for _, raw := range items {
			field, err := directField(raw)
			if err != nil {
				return nil, err
			}
			out = append(out, oneTimeCanonicalField(field))
		}
		return out, nil
	}
	canonicalFields, err := fields(body["fields"])
	if err != nil {
		return nil, err
	}
	declarations, ok := body["declarations"].(map[string]any)
	if !ok {
		return nil, fmt.Errorf("invalid declaration map")
	}
	canonicalDeclarations := map[string]any{}
	for name, raw := range declarations {
		declaration, ok := raw.(map[string]any)
		if !ok {
			return nil, fmt.Errorf("invalid declaration")
		}
		declaration = copyObject(declaration)
		switch declaration["kind"] {
		case "newtype":
			if text, ok := declaration["of"].(string); ok {
				canonical, _, err := accessorType(text, 0)
				if err != nil {
					return nil, err
				}
				declaration["of"] = canonical
			}
		case "struct":
			declaration["fields"], err = fields(declaration["fields"])
			if err != nil {
				return nil, err
			}
		case "union":
			if variants, ok := declaration["variants"].(map[string]any); ok {
				canonical := copyObject(variants)
				for name, raw := range variants {
					if text, ok := raw.(string); ok {
						ty, _, err := accessorType(text, 0)
						if err != nil {
							return nil, err
						}
						canonical[name] = ty
					}
				}
				declaration["variants"] = canonical
			}
		}
		canonicalDeclarations[name] = declaration
	}
	return map[string]any{"fields": canonicalFields, "declarations": canonicalDeclarations}, nil
}
func oneTimeOperandKind(value operand) (string, error) {
	if value.isFact {
		switch value.path {
		case "value":
			return "text", nil
		case "value.count":
			return "number", nil
		default:
			return "", fmt.Errorf("one-time invariant reads an undeclared path")
		}
	}
	switch value.literal.(type) {
	case string:
		return "text", nil
	case bool:
		return "bool", nil
	case json.Number, float64:
		return "number", nil
	default:
		return "", fmt.Errorf("invalid invariant scalar")
	}
}
func oneTimePredicate(condition predicate) error {
	pathKind := func(path string) (string, error) { return oneTimeOperandKind(operand{path: path, isFact: true}) }
	switch condition.kind {
	case "always", "never":
		return nil
	case "all", "any":
		for _, child := range condition.children {
			if err := oneTimePredicate(child); err != nil {
				return err
			}
		}
		return nil
	case "not":
		if condition.body == nil {
			return fmt.Errorf("missing invariant child")
		}
		return oneTimePredicate(*condition.body)
	case "truthy", "defined":
		_, err := pathKind(condition.path)
		return err
	case "compare":
		left, err := oneTimeOperandKind(condition.left)
		if err != nil {
			return err
		}
		right, err := oneTimeOperandKind(condition.right)
		if err != nil {
			return err
		}
		if left != right || (left == "bool" && condition.op != "==" && condition.op != "!=") {
			return fmt.Errorf("one-time invariant scalar mismatch")
		}
		if condition.left.isFact && condition.right.literal == "value" || condition.right.isFact && condition.left.literal == "value" {
			return fmt.Errorf("one-time invariant mistakes a field for a literal")
		}
		return nil
	case "any_of", "none_of":
		kind, err := pathKind(condition.path)
		if err != nil {
			return err
		}
		for _, value := range condition.values {
			other, err := oneTimeOperandKind(operand{literal: value})
			if err != nil || other != kind {
				return fmt.Errorf("one-time membership scalar mismatch")
			}
		}
		return nil
	case "starts_with", "ends_with", "contains":
		kind, err := pathKind(condition.path)
		text, ok := condition.right.literal.(string)
		if err != nil || kind != "text" || !ok || text == "" || text == "value" {
			return fmt.Errorf("invalid String match invariant")
		}
		return nil
	case "equals_ignore_case", "in_ignore_case":
		kind, err := pathKind(condition.path)
		if err != nil || kind != "text" || len(condition.values) == 0 {
			return fmt.Errorf("invalid String fold invariant")
		}
		for _, value := range condition.values {
			text, ok := value.(string)
			if !ok || text == "value" {
				return fmt.Errorf("invalid String fold literal")
			}
		}
		return nil
	default:
		return fmt.Errorf("one-time String has no collection or other selectors")
	}
}

func admitOneTimeResponse(raw any, command string) (oneTimeResponse, error) {
	fail := func(err error) (oneTimeResponse, error) { return oneTimeResponse{}, err }
	body, err := closed(raw, "fields declarations constraints", "")
	if err != nil {
		return fail(err)
	}
	shapeInput, err := oneTimeShapeInput(body)
	if err != nil {
		return fail(err)
	}
	shapeInput["command"], shapeInput["expected"] = command, map[string]any{}
	shape, err := admitDirectResponse(shapeInput)
	if err != nil {
		return fail(err)
	}
	constraints, ok := body["constraints"].(map[string]any)
	if !ok {
		return fail(fmt.Errorf("one-time constraints must be an object"))
	}
	result := oneTimeResponse{Fields: shape.Fields, Declarations: shape.Declarations}
	if result.Constraints, err = admitStringConstraints(constraints, result.Declarations); err != nil {
		return fail(err)
	}
	result.authority, err = directWire(map[string]any{"fields": oneTimeCanonicalFields(body["fields"]), "declarations": oneTimeCanonicalDeclarations(body["declarations"]), "constraints": result.Constraints})
	if err != nil {
		return fail(err)
	}
	return result, nil
}

// admitStringConstraints admits closed String-newtype rules against the declarations they travel
// with: the one-time profile, and the response observations of suite/46 and /47 (beyond10x/ess#499).
func admitStringConstraints(constraints map[string]any, declarations map[string]selectionDeclaration) (map[string]oneTimeConstraints, error) {
	admitted := map[string]oneTimeConstraints{}
	for name_, rawRules := range constraints {
		declaration, ok := declarations[name_]
		if !ok || declaration.Kind != "newtype" || !oneTimeRequiredString(name_, declarations) {
			return nil, fmt.Errorf("one-time constraint requires a declared String newtype")
		}
		rules, err := closed(rawRules, "invariants", "alphabet prefix")
		if err != nil {
			return nil, err
		}
		typed := oneTimeConstraints{}
		for _, key := range []string{"alphabet", "prefix"} {
			if raw := rules[key]; raw != nil {
				text, err := text(raw)
				if err != nil || text == "" {
					return nil, fmt.Errorf("invalid String %s", key)
				}
				if key == "alphabet" {
					seen := map[rune]bool{}
					for _, character := range text {
						if seen[character] {
							return nil, fmt.Errorf("duplicate alphabet character")
						}
						seen[character] = true
					}
					typed.Alphabet = &text
				} else {
					typed.Prefix = &text
				}
			}
		}
		invariants, err := array(rules["invariants"])
		if err != nil {
			return nil, err
		}
		typed.Invariants = []json.RawMessage{}
		for _, rawPredicate := range invariants {
			if err = admitPredicateEnvelope(rawPredicate, 0); err != nil {
				return nil, err
			}
			parsed, err := fromNode(oneTimePredicateNode(oneTimeSourceNumbers(rawPredicate)))
			if err != nil {
				return nil, err
			}
			parsed = oneTimeSimplify(parsed)
			if err = oneTimePredicate(parsed); err != nil {
				return nil, err
			}
			encoded, err := directWire(oneTimePredicateWire(parsed))
			if err != nil {
				return nil, err
			}
			typed.Invariants = append(typed.Invariants, encoded)
			typed.parsed = append(typed.parsed, parsed)
		}
		admitted[name_] = typed
	}
	for name_, rules := range admitted {
		prefix := ""
		alphabets := []string{}
		ty := name_
		for ty != "String" {
			current := admitted[ty]
			if current.Prefix != nil {
				if !strings.HasPrefix(prefix, *current.Prefix) && !strings.HasPrefix(*current.Prefix, prefix) {
					return nil, fmt.Errorf("conflicting String prefixes")
				}
				if len(*current.Prefix) > len(prefix) {
					prefix = *current.Prefix
				}
			}
			if current.Alphabet != nil {
				alphabets = append(alphabets, *current.Alphabet)
			}
			ty = strings.TrimSpace(declarations[ty].Of)
		}
		if rules.Alphabet != nil {
			for _, alphabet := range alphabets {
				shared := false
				for _, character := range *rules.Alphabet {
					if strings.ContainsRune(alphabet, character) {
						shared = true
					}
				}
				if !shared {
					return nil, fmt.Errorf("conflicting String alphabets")
				}
			}
		}
		for _, alphabet := range alphabets {
			for _, character := range prefix {
				if !strings.ContainsRune(alphabet, character) {
					return nil, fmt.Errorf("prefix outside String alphabet")
				}
			}
		}
	}
	return admitted, nil
}

func admitOneTimeTrace(scenario map[string]any, major int) (*oneTimeTrace, error) {
	raw, present := scenario["one_time_response"]
	if !present || raw == nil {
		return nil, nil
	}
	if major < 34 {
		return nil, fmt.Errorf("one-time policy requires suite34/35")
	}
	policy, err := closed(raw, "origins required_origins events event_windows", "")
	if err != nil {
		return nil, err
	}
	origins, err := array(policy["origins"])
	if err != nil {
		return nil, err
	}
	required, err := array(policy["required_origins"])
	if err != nil {
		return nil, err
	}
	events, err := array(policy["events"])
	if err != nil {
		return nil, err
	}
	windows, err := array(policy["event_windows"])
	if err != nil {
		return nil, err
	}
	if len(origins) == 0 || len(origins) > 256 || len(windows) > 65536 || len(required) == 0 {
		return nil, fmt.Errorf("one-time trace authority bound")
	}
	steps, err := array(scenario["steps"])
	if err != nil {
		return nil, err
	}
	sources, err := array(scenario["source"])
	if err != nil {
		return nil, err
	}
	hasSource := func(kind string, source any) bool {
		for _, raw := range sources {
			ref, ok := raw.(map[string]any)
			if ok && ref["kind"] == kind && reflect.DeepEqual(ref["name"], source) {
				return true
			}
		}
		return false
	}
	result := &oneTimeTrace{Origins: []oneTimeOrigin{}, RequiredOrigins: []OutcomeRef{}, Events: []oneTimeEventAuthority{}, EventWindows: []oneTimeEventWindow{}}
	seen := map[OutcomeRef]bool{}
	schemas := map[string]oneTimeResponse{}
	decodeOutcome := func(raw any) (OutcomeRef, error) {
		if err := admitOutcome(raw); err != nil {
			return OutcomeRef{}, err
		}
		body := raw.(map[string]any)
		return OutcomeRef{Command: body["command"].(string), Outcome: body["outcome"].(string)}, nil
	}
	for _, rawOrigin := range origins {
		origin, err := closed(rawOrigin, "command outcome response fields", "")
		if err != nil {
			return nil, err
		}
		if err = name(origin["command"], false); err != nil {
			return nil, err
		}
		command := origin["command"].(string)
		outcome, err := decodeOutcome(origin["outcome"])
		if err != nil {
			return nil, err
		}
		if outcome.Command != command || seen[outcome] {
			return nil, fmt.Errorf("invalid or duplicate origin")
		}
		seen[outcome] = true
		invoked := false
		for _, rawStep := range steps {
			step, ok := rawStep.(map[string]any)
			if ok && (step["step"] == "execute_command" || step["step"] == "execute_command_without_input") && step["command"] == command {
				invoked = true
			}
		}
		if !invoked || !hasSource("command", command) || !hasSource("outcome", origin["outcome"]) {
			return nil, fmt.Errorf("origin lacks invocation or source authority")
		}
		response, err := admitOneTimeResponse(origin["response"], command)
		if err != nil {
			return nil, err
		}
		if prior, ok := schemas[command]; ok && !bytes.Equal(prior.authority, response.authority) {
			return nil, fmt.Errorf("contradictory response schemas")
		}
		schemas[command] = response
		fields, err := array(origin["fields"])
		if err != nil || len(fields) == 0 || len(fields) > 256 {
			return nil, fmt.Errorf("origin field bound")
		}
		names := map[string]bool{}
		typed := oneTimeOrigin{Command: command, Outcome: outcome, Response: response, Fields: []string{}}
		for _, rawField := range fields {
			field, err := text(rawField)
			if err != nil || names[field] {
				return nil, fmt.Errorf("invalid or duplicate origin field")
			}
			names[field] = true
			valid := false
			for _, declared := range response.Fields {
				if declared.Name == field && oneTimeRequiredString(declared.Type, response.Declarations) {
					valid = true
				}
			}
			if !valid {
				return nil, fmt.Errorf("one-time field must be declared required String")
			}
			typed.Fields = append(typed.Fields, field)
		}
		result.Origins = append(result.Origins, typed)
	}
	requiredSeen := map[OutcomeRef]bool{}
	for _, raw := range required {
		outcome, err := decodeOutcome(raw)
		if err != nil {
			return nil, err
		}
		if !seen[outcome] || requiredSeen[outcome] {
			return nil, fmt.Errorf("invalid required origin")
		}
		requiredSeen[outcome] = true
		result.RequiredOrigins = append(result.RequiredOrigins, outcome)
	}
	catalog := map[string]uint64{}
	for _, raw := range events {
		event, err := closed(raw, "event within_ms", "")
		if err != nil {
			return nil, err
		}
		if err = name(event["event"], false); err != nil {
			return nil, err
		}
		within, err := unsigned(event["within_ms"])
		if err != nil {
			return nil, err
		}
		name_ := event["event"].(string)
		if _, exists := catalog[name_]; exists || !hasSource("event", name_) {
			return nil, fmt.Errorf("event catalog contradicts source")
		}
		catalog[name_] = within
		result.Events = append(result.Events, oneTimeEventAuthority{Event: name_, WithinMillis: within})
	}
	observed := map[oneTimeEventWindow]bool{}
	for _, raw := range windows {
		window, err := closed(raw, "event after_step within_ms", "")
		if err != nil {
			return nil, err
		}
		if err = name(window["event"], false); err != nil {
			return nil, err
		}
		after, err := unsigned(window["after_step"])
		if err != nil {
			return nil, err
		}
		within, err := unsigned(window["within_ms"])
		if err != nil {
			return nil, err
		}
		name_ := window["event"].(string)
		if _, ok := catalog[name_]; !ok || after >= uint64(len(steps)) {
			return nil, fmt.Errorf("window lacks catalog or operation")
		}
		step := steps[after].(map[string]any)
		if step["step"] != "execute_command" && step["step"] != "execute_command_without_input" && step["step"] != "query_view" && step["step"] != "eventually_view" {
			return nil, fmt.Errorf("window is not anchored to operation")
		}
		typed := oneTimeEventWindow{Event: name_, AfterStep: after, WithinMillis: within}
		if observed[typed] {
			return nil, fmt.Errorf("duplicate event window")
		}
		observed[typed] = true
		result.EventWindows = append(result.EventWindows, typed)
	}
	for index, rawStep := range steps {
		step := rawStep.(map[string]any)
		if step["step"] == "execute_command" || step["step"] == "execute_command_without_input" || step["step"] == "query_view" || step["step"] == "eventually_view" {
			for event, within := range catalog {
				if !observed[oneTimeEventWindow{Event: event, AfterStep: uint64(index), WithinMillis: 0}] || !observed[oneTimeEventWindow{Event: event, AfterStep: uint64(index), WithinMillis: within}] {
					return nil, fmt.Errorf("omitted required event window")
				}
			}
		}
	}
	encoded, err := directWire(result)
	if err != nil || accessorCompactBytes(encoded) > 1048576 {
		return nil, fmt.Errorf("one-time authority byte bound")
	}
	return result, nil
}

// ---- rules a suite/52 or /53 states once (beyond10x/ess#570) -----------------------------------

// compactEvents reads a list of catalog event names that must appear in catalog order.
func compactEvents(value any, catalog []string) ([]string, error) {
	list, err := array(value)
	if err != nil {
		return nil, err
	}
	names := []string{}
	at := -1
	for _, raw := range list {
		event, err := text(raw)
		if err != nil {
			return nil, err
		}
		found := -1
		for index := at + 1; index < len(catalog); index++ {
			if catalog[index] == event {
				found = index
				break
			}
		}
		if found < 0 {
			return nil, fmt.Errorf("an event list leaves out catalog events in catalog order")
		}
		at = found
		names = append(names, event)
	}
	if len(names) >= len(catalog) {
		return nil, fmt.Errorf("an event list keeps at least one catalog event")
	}
	return names, nil
}

// compactKept is every catalog event outside except, in catalog order.
func compactKept(catalog, except []string) []string {
	left := map[string]bool{}
	for _, event := range except {
		left[event] = true
	}
	kept := []string{}
	for _, event := range catalog {
		if !left[event] {
			kept = append(kept, event)
		}
	}
	return kept
}

// expandCompactSuite expands, in place and before any admission reads a scenario, what a suite/52
// or /53 states once at its root: each scenario's unpublished-event lists and absent-event runs
// into its steps, and each compact one-time trace into the complete trace with the catalog events
// among its sources. It returns the document to decode, re-encoded where anything was expanded.
func expandCompactSuite(root map[string]any, major int, raw string) (string, error) {
	rawEvents, hasEvents := root["events"]
	rawCatalog, hasCatalog := root["one_time_response"]
	if (hasEvents || hasCatalog) && major < 52 {
		return "", fmt.Errorf("a suite-level event or one-time catalog requires suite/52 or /53")
	}
	scenarios, ok := root["scenarios"].(map[string]any)
	if !ok {
		return raw, nil
	}
	catalog := []string{}
	if hasEvents {
		list, err := array(rawEvents)
		if err != nil {
			return "", err
		}
		for _, value := range list {
			if err := name(value, false); err != nil {
				return "", err
			}
			event := value.(string)
			if len(catalog) > 0 && catalog[len(catalog)-1] >= event {
				return "", fmt.Errorf("the event catalog lists each event once, in name order")
			}
			catalog = append(catalog, event)
		}
	}
	type marked struct {
		command  string
		response any
		outcomes []map[string]any
	}
	commands := []marked{}
	if hasCatalog {
		body, err := closed(rawCatalog, "commands", "")
		if err != nil {
			return "", err
		}
		list, err := array(body["commands"])
		if err != nil {
			return "", err
		}
		for _, value := range list {
			entry, err := closed(value, "command response outcomes", "")
			if err != nil {
				return "", err
			}
			if err := name(entry["command"], false); err != nil {
				return "", err
			}
			command := entry["command"].(string)
			if len(commands) > 0 && commands[len(commands)-1].command >= command {
				return "", fmt.Errorf("the one-time catalog lists each command once, in name order")
			}
			outcomes, err := array(entry["outcomes"])
			if err != nil || len(outcomes) == 0 {
				return "", fmt.Errorf("the one-time catalog lists each command with an outcome")
			}
			typed := marked{command: command, response: entry["response"]}
			for _, value := range outcomes {
				outcome, err := closed(value, "outcome fields", "")
				if err != nil {
					return "", err
				}
				typed.outcomes = append(typed.outcomes, outcome)
			}
			commands = append(commands, typed)
		}
	}
	operations := map[any]bool{"execute_command": true, "execute_command_without_input": true, "query_view": true, "eventually_view": true}
	referred, changed, used := false, false, map[string]bool{}
	for _, id := range meaningKeys(scenarios) {
		scenario, ok := scenarios[id].(map[string]any)
		if !ok {
			continue
		}
		_, patched := scenario["unpublished_events"]
		_, inserted := scenario["absent_events"]
		trace, traced := scenario["one_time_response"].(map[string]any)
		_, compactTrace := trace["within_ms"]
		if !patched && !inserted && !(traced && compactTrace) {
			continue
		}
		if major < 52 {
			return "", fmt.Errorf("absent-event runs, unpublished-event lists and compact traces require suite/52 or /53")
		}
		if !hasEvents {
			return "", fmt.Errorf("%s: a compact spelling requires the suite event catalog", id)
		}
		steps, err := array(scenario["steps"])
		if err != nil {
			return "", err
		}
		referred, changed = true, true
		if patched {
			list, err := array(scenario["unpublished_events"])
			if err != nil || len(list) == 0 {
				return "", fmt.Errorf("an empty unpublished_events is written as none")
			}
			previous := -1
			for _, value := range list {
				patch, err := closed(value, "step except", "")
				if err != nil {
					return "", err
				}
				index, err := unsigned(patch["step"])
				if err != nil || int(index) <= previous || index >= uint64(len(steps)) {
					return "", fmt.Errorf("unpublished-event lists name written steps in ascending order")
				}
				previous = int(index)
				step, ok := steps[index].(map[string]any)
				if !ok || step["step"] != "expect_not_granted" {
					return "", fmt.Errorf("an unpublished-event list names an expect_not_granted step")
				}
				if present, ok := step["unpublished"]; ok {
					if list, ok := present.([]any); !ok || len(list) != 0 {
						return "", fmt.Errorf("an unpublished-event list completes a step written without one")
					}
				}
				except, err := compactEvents(patch["except"], catalog)
				if err != nil {
					return "", err
				}
				kept := []any{}
				for _, event := range compactKept(catalog, except) {
					kept = append(kept, event)
				}
				step["unpublished"] = kept
			}
			delete(scenario, "unpublished_events")
		}
		if inserted {
			list, err := array(scenario["absent_events"])
			if err != nil || len(list) == 0 {
				return "", fmt.Errorf("an empty absent_events is written as none")
			}
			expanded := []any{}
			next := 0
			for _, value := range list {
				run, err := closed(value, "before except", "")
				if err != nil {
					return "", err
				}
				before, err := unsigned(run["before"])
				if err != nil || before < uint64(next) || before > uint64(len(steps)) {
					return "", fmt.Errorf("absent-event runs stand at ascending positions within the steps")
				}
				expanded = append(expanded, steps[next:before]...)
				next = int(before)
				except, err := compactEvents(run["except"], catalog)
				if err != nil {
					return "", err
				}
				for _, event := range compactKept(catalog, except) {
					expanded = append(expanded, map[string]any{"step": "expect_no_event", "event": event})
				}
			}
			steps = append(expanded, steps[next:]...)
			scenario["steps"] = steps
			delete(scenario, "absent_events")
		}
		if traced && compactTrace {
			if !hasCatalog {
				return "", fmt.Errorf("%s: a compact one-time trace requires the suite one-time catalog", id)
			}
			body, err := closed(trace, "required_origins within_ms", "")
			if err != nil {
				return "", err
			}
			within, err := unsigned(body["within_ms"])
			if err != nil {
				return "", err
			}
			invoked := map[any]bool{}
			positions := []int{}
			for index, value := range steps {
				step, _ := value.(map[string]any)
				if step["step"] == "execute_command" || step["step"] == "execute_command_without_input" {
					invoked[step["command"]] = true
				}
				if operations[step["step"]] {
					positions = append(positions, index)
				}
			}
			for command := range invoked {
				if command, ok := command.(string); ok {
					used[command] = true
				}
			}
			lengths := []uint64{0}
			if within != 0 {
				lengths = append(lengths, within)
			}
			if len(catalog)*len(positions)*len(lengths) > 65536 {
				return "", fmt.Errorf("one-time trace authority bound")
			}
			origins := []any{}
			for _, entry := range commands {
				if !invoked[entry.command] {
					continue
				}
				for _, outcome := range entry.outcomes {
					origins = append(origins, map[string]any{
						"command":  entry.command,
						"outcome":  map[string]any{"command": entry.command, "outcome": outcome["outcome"]},
						"response": entry.response,
						"fields":   outcome["fields"],
					})
				}
			}
			number := func(value uint64) json.Number { return json.Number(strconv.FormatUint(value, 10)) }
			events, windows := []any{}, []any{}
			for _, event := range catalog {
				events = append(events, map[string]any{"event": event, "within_ms": number(within)})
				for _, position := range positions {
					for _, length := range lengths {
						windows = append(windows, map[string]any{"event": event, "after_step": number(uint64(position)), "within_ms": number(length)})
					}
				}
			}
			scenario["one_time_response"] = map[string]any{
				"origins": origins, "required_origins": body["required_origins"], "events": events, "event_windows": windows,
			}
			sources, err := array(scenario["source"])
			if err != nil {
				return "", err
			}
			for _, event := range catalog {
				present := false
				for _, value := range sources {
					if ref, ok := value.(map[string]any); ok && ref["kind"] == "event" && ref["name"] == event {
						present = true
					}
				}
				if !present {
					sources = append(sources, map[string]any{"kind": "event", "name": event})
				}
			}
			scenario["source"] = sources
		}
	}
	if hasEvents && !referred {
		return "", fmt.Errorf("the suite event catalog is written only where a scenario refers to it")
	}
	for _, entry := range commands {
		if !used[entry.command] {
			return "", fmt.Errorf("the one-time catalog carries authority no compact trace uses")
		}
	}
	delete(root, "events")
	delete(root, "one_time_response")
	if !changed {
		return raw, nil
	}
	var encoded bytes.Buffer
	encoder := json.NewEncoder(&encoded)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(root); err != nil {
		return "", err
	}
	return encoded.String(), nil
}
