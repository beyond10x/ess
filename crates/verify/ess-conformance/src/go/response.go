// Actual command responses are compared under immutable typed suite authority.
type responseObservation struct {
	Command      string                          `json:"command"`
	Outcome      OutcomeRef                      `json:"outcome"`
	Event        string                          `json:"event"`
	Fields       []accessorField                 `json:"fields"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
	Mappings     map[string]string               `json:"mappings"`
	Targets      []accessorField                 `json:"targets"`
	Nested       *nestedResponseTargets          `json:"nested,omitempty"`
}

func (r *responseObservation) UnmarshalJSON(raw []byte) error {
	var members map[string]json.RawMessage
	if err := json.Unmarshal(raw, &members); err != nil {
		return err
	}
	if value, exists := members["nested"]; exists && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return fmt.Errorf("nested response cannot be null")
	}
	type plain responseObservation
	var value plain
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return err
	}
	*r = responseObservation(value)
	if r.Nested != nil {
		if err := normalizeNestedResponse(r); err != nil {
			return err
		}
	}
	return r.validate()
}
func (r responseObservation) validate() error {
	nestedCount := 0
	if r.Nested != nil {
		nestedCount = len(r.Nested.Mappings)
	}
	if r.Command != r.Outcome.Command || r.Outcome.Outcome == "" || len(r.Fields) == 0 || len(r.Fields) > 256 || len(r.Targets) > 256 || len(r.Declarations) > 4096 || len(r.Mappings)+nestedCount == 0 || len(r.Mappings)+nestedCount > 256 || len(r.Mappings) != len(r.Targets) {
		return fmt.Errorf("invalid response contract bounds or owner")
	}
	for _, label := range []string{r.Command, r.Event} {
		if err := name(label, false); err != nil {
			return err
		}
	}
	if err := validateTypedFields([][]accessorField{r.Fields, r.Targets}, r.Declarations); err != nil {
		return err
	}
	roots := map[string]string{}
	for _, field := range r.Fields {
		roots[field.Name] = field.Type
	}
	for _, target := range r.Targets {
		source, ok := r.Mappings[target.Name]
		from, declared := roots[source]
		if !ok || !declared || !responseAssignable(from, target.Type) {
			return fmt.Errorf("invalid response mapping")
		}
	}
	if r.Nested != nil {
		if err := r.Nested.validate(r); err != nil {
			return err
		}
		if len(nestedResponseCanonical(r)) > 1048576 {
			return fmt.Errorf("response contract byte limit")
		}
		return nil
	}
	raw, err := json.Marshal(r)
	if err != nil {
		return err
	}
	if len(raw) > 1048576 {
		return fmt.Errorf("response contract byte limit")
	}
	return nil
}
func validateTypedFields(groups [][]accessorField, declarations map[string]selectionDeclaration) error {
	used := map[string]bool{}
	for _, fields := range groups {
		seen := map[string]bool{}
		for _, field := range fields {
			if field.Name == "" || seen[field.Name] {
				return fmt.Errorf("duplicate/empty response field")
			}
			seen[field.Name] = true
			if err := checkResponseType(declarations, field.Type, used, map[string]bool{}, 0); err != nil {
				return err
			}
		}
	}
	if len(used) != len(declarations) {
		return fmt.Errorf("unrelated response declarations")
	}
	return nil
}
func responseAssignable(from, to string) bool {
	if from == to {
		return true
	}
	if inner, ok := accessorOptional(to); ok {
		return responseAssignable(from, inner)
	}
	return false
}
func checkResponseType(declarations map[string]selectionDeclaration, source string, used, stack map[string]bool, depth int) error {
	if depth > 128 {
		return fmt.Errorf("response type depth limit")
	}
	if inner, ok := accessorOptional(source); ok {
		return checkResponseType(declarations, inner, used, stack, depth+1)
	}
	if strings.HasPrefix(source, "Map<") && !strings.HasPrefix(source, "Map<String, ") {
		return fmt.Errorf("response map key must be String")
	}
	if inner, ok, err := accessorCollection(source); ok || err != nil {
		if err != nil {
			return err
		}
		return checkResponseType(declarations, inner, used, stack, depth+1)
	}
	if source == "Json" || accessorPrimitive(source) && source != "Binary64" {
		return nil
	}
	body, ok := declarations[source]
	if !ok || stack[source] {
		return fmt.Errorf("missing/recursive response type")
	}
	if used[source] {
		return nil
	}
	used[source] = true
	stack[source] = true
	defer delete(stack, source)
	children := []string{}
	switch body.Kind {
	case "newtype":
		children = append(children, body.Of)
	case "struct":
		seen := map[string]bool{}
		for _, field := range body.Fields {
			if field.Name == "" || seen[field.Name] {
				return fmt.Errorf("duplicate/empty response member")
			}
			seen[field.Name] = true
			children = append(children, field.Type)
		}
	case "enum":
		var variants []string
		if json.Unmarshal(body.Variants, &variants) != nil || len(variants) == 0 {
			return fmt.Errorf("invalid response enum")
		}
		seen := map[string]bool{}
		for _, label := range variants {
			if label == "" || seen[label] {
				return fmt.Errorf("duplicate response enum label")
			}
			seen[label] = true
		}
	case "union":
		var variants map[string]string
		if body.Tag == "" || json.Unmarshal(body.Variants, &variants) != nil || len(variants) == 0 {
			return fmt.Errorf("invalid response union")
		}
		for _, child := range variants {
			children = append(children, child)
		}
	default:
		return fmt.Errorf("unknown response declaration")
	}
	for _, child := range children {
		if err := checkResponseType(declarations, child, used, stack, depth+1); err != nil {
			return err
		}
	}
	return nil
}
func (r responseObservation) compare(response, payload map[string]Node) error {
	if err := r.validate(); err != nil {
		return err
	}
	if response == nil {
		return fmt.Errorf("command returned no response")
	}
	names := map[string]bool{}
	for _, field := range r.Fields {
		names[field.Name] = true
	}
	for name := range response {
		if !names[name] {
			return fmt.Errorf("response has an undeclared field")
		}
	}
	observer := selectionObservation{Declarations: r.Declarations, responseMode: true}
	bytes := 0
	for _, field := range r.Fields {
		value, present := response[field.Name]
		if err := observer.validateValue(field.Type, value, present, &bytes, 0); err != nil {
			return fmt.Errorf("response field %s: %v", field.Name, err)
		}
	}
	if bytes > 1048576 {
		return fmt.Errorf("response byte limit")
	}
	for _, field := range r.Fields {
		value, present := response[field.Name]
		if field.Presence == "null_when_absent" && !present {
			return fmt.Errorf("response field %s was left out, and it is declared null_when_absent", field.Name)
		}
		if field.Presence == "omitted_when_absent" && present && value == nil {
			return fmt.Errorf("response field %s was sent as null, and it is declared omitted_when_absent", field.Name)
		}
	}
	for target, source := range r.Mappings {
		actual, exists := response[source]
		emitted, present := payload[target]
		optional := false
		for _, field := range r.Fields {
			if field.Name == source {
				_, optional = accessorOptional(field.Type)
			}
		}
		if optional && (!exists || actual == nil) && (!present || emitted == nil) {
			continue
		}
		if !exists || !present || !responseEqual(actual, emitted) {
			return fmt.Errorf("event field %s differs from actual response field %s", target, source)
		}
	}
	if r.Nested != nil {
		return r.Nested.compare(r, response, payload)
	}
	return nil
}
func (r *run) expectResponsePayload(index int, step Step) bool {
	contract := step.Response
	if contract == nil {
		return r.fail(index, "missing response observation")
	}
	if r.lastCommand != contract.Command || r.last.Outcome != contract.Outcome.Outcome {
		return r.fail(index, "response observation names a different command/outcome")
	}
	for _, event := range r.last.DirectEvents {
		if event.Event == contract.Event {
			if err := contract.compare(r.last.Response, event.Payload); err != nil {
				return r.fail(index, "%v", err)
			}
			return true
		}
	}
	return r.fail(index, "same invocation did not emit the response-mapped event")
}

func admitResponse(value any, major int) error {
	root, err := closed(value, "command outcome event fields declarations mappings targets", "nested")
	if err != nil {
		return err
	}
	if nested, exists := root["nested"]; exists {
		if major < 34 {
			return fmt.Errorf("nested response authority requires suite/34 or /35")
		}
		if err := admitNestedResponse(nested); err != nil {
			return err
		}
		for _, key := range []string{"fields", "targets"} {
			items, err := array(root[key])
			if err != nil {
				return err
			}
			for _, item := range items {
				optional := ""
				if key == "fields" {
					optional = "presence"
				}
				if _, err := closed(item, "name type", optional); err != nil {
					return err
				}
			}
		}
		declarations, ok := root["declarations"].(map[string]any)
		if !ok {
			return fmt.Errorf("invalid response declarations")
		}
		for _, value := range declarations {
			body, ok := value.(map[string]any)
			if !ok {
				return fmt.Errorf("invalid response declaration")
			}
			if fields, exists := body["fields"]; exists {
				items, err := array(fields)
				if err != nil {
					return err
				}
				for _, item := range items {
					if _, err := closed(item, "name type", ""); err != nil {
						return err
					}
				}
			}
		}
	}
	if err = admitOutcome(root["outcome"]); err != nil {
		return err
	}
	if err := admitTypedDeclarations(root["declarations"]); err != nil {
		return err
	}
	for _, key := range []string{"fields", "targets"} {
		fields, e := array(root[key])
		if e != nil {
			return e
		}
		for _, field := range fields {
			if key == "fields" {
				field, e = responseFieldPresence(field)
				if e != nil {
					return e
				}
			}
			if e = admitAccessorField(field); e != nil {
				return e
			}
		}
	}
	encoded, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var response responseObservation
	return json.Unmarshal(encoded, &response)
}

func admitTypedDeclarations(value any) error {
	var err error
	declarations, ok := value.(map[string]any)
	if !ok {
		return fmt.Errorf("response declarations must be object")
	}
	for _, raw := range declarations {
		body, ok := raw.(map[string]any)
		if !ok {
			return fmt.Errorf("invalid response declaration")
		}
		required := "kind"
		switch body["kind"] {
		case "newtype":
			required += " of"
		case "struct":
			required += " fields"
		case "enum":
			required += " variants"
		case "union":
			required += " tag variants"
		default:
			return fmt.Errorf("invalid response declaration kind")
		}
		if _, err = closed(raw, required, ""); err != nil {
			return err
		}
		if fields, exists := body["fields"]; exists {
			items, e := array(fields)
			if e != nil {
				return e
			}
			for _, field := range items {
				if e = admitAccessorField(field); e != nil {
					return e
				}
			}
		}
	}
	return nil
}

// Snapshot response-bearing results before any subsequent target callback can mutate their maps.
func snapshotResponseResult(result CommandResult) (CommandResult, error) {
	raw, err := json.Marshal(result)
	if err != nil {
		return CommandResult{}, fmt.Errorf("response result is not a typed wire value")
	}
	if len(raw) > 1048576 {
		return CommandResult{}, fmt.Errorf("response result byte limit")
	}
	var snapshot CommandResult
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err = decoder.Decode(&snapshot); err != nil {
		return CommandResult{}, err
	}
	return snapshot, nil
}

// Exact bounded numeric observation; never round a returned Integer through binary64.
func responseNumber(value Node) (*big.Rat, bool) {
	number, ok := value.(json.Number)
	if !ok {
		return nil, false
	}
	raw := number.String()
	if len(raw) > 512 {
		return nil, false
	}
	if index := strings.IndexAny(raw, "eE"); index >= 0 {
		exponent, err := strconv.Atoi(raw[index+1:])
		if err != nil || exponent < -512 || exponent > 512 {
			return nil, false
		}
	}
	finite, err := strconv.ParseFloat(raw, 64)
	if err != nil || math.IsInf(finite, 0) || math.IsNaN(finite) {
		return nil, false
	}
	result, ok := new(big.Rat).SetString(raw)
	return result, ok
}
func responsePrimitiveAdmits(source string, value Node) bool {
	switch source {
	case "Integer":
		number, ok := responseNumber(value)
		return ok && number.IsInt() && number.Num().IsInt64()
	case "Decimal":
		_, ok := responseNumber(value)
		return ok
	case "String", "Timestamp", "Duration":
		_, ok := value.(string)
		return ok
	case "Boolean":
		_, ok := value.(bool)
		return ok
	case "Uuid":
		text, ok := value.(string)
		return ok && canonicalUUID(text)
	case "Bytes":
		text, ok := value.(string)
		return ok && paddedBase64(text)
	default:
		return false
	}
}

// A number falls through to equal, which compares it by value whatever Go type carries either side
// (beyond10x/ess#101), and never through binary64 for an integer token.
func responseEqual(left, right Node) bool {
	switch value := left.(type) {
	case []any:
		other, ok := right.([]any)
		if !ok || len(value) != len(other) {
			return false
		}
		for i, child := range value {
			if !responseEqual(child, other[i]) {
				return false
			}
		}
		return true
	case map[string]any:
		other, ok := right.(map[string]any)
		if !ok || len(value) != len(other) {
			return false
		}
		for key, child := range value {
			actual, present := other[key]
			if !present || !responseEqual(child, actual) {
				return false
			}
		}
		return true
	default:
		return equal(left, right)
	}
}

// responseFieldPresence admits a response field's presence policy (suite/24, beyond10x/ess#139) and
// returns the field without it, for the admission every other typed field takes.
func responseFieldPresence(value any) (any, error) {
	field, ok := value.(map[string]any)
	if !ok {
		return value, nil
	}
	presence, present := field["presence"]
	if !present {
		return value, nil
	}
	if presence != "null_when_absent" && presence != "omitted_when_absent" {
		return nil, fmt.Errorf("invalid presence %v", presence)
	}
	rest := copyObject(field)
	delete(rest, "presence")
	return rest, nil
}

// responsePresenceMajor refuses a response field presence policy under a major older than /24.
func responsePresenceMajor(step map[string]any, major int) error {
	response, _ := step["response"].(map[string]any)
	fields, _ := response["fields"].([]any)
	for _, field := range fields {
		if entry, ok := field.(map[string]any); ok {
			if _, present := entry["presence"]; present && major < 24 {
				return fmt.Errorf("field presence policies require suite/24 or /25")
			}
		}
	}
	return nil
}

// Nested destinations carry complete representation facts, separate from response value codecs.
type nestedResponseField struct {
	Name string `json:"name"`
	Type string `json:"type"`
}
type nestedResponseMapping struct {
	Target []string `json:"target"`
	Source string   `json:"source"`
}
type nestedResponseTargets struct {
	Roots        []nestedResponseField           `json:"roots"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
	Mappings     []nestedResponseMapping         `json:"mappings"`
}

var nestedResponseMember = regexp.MustCompile(`^_*[A-Za-z][A-Za-z0-9_]*$`)

func (n *nestedResponseTargets) UnmarshalJSON(raw []byte) error {
	var original any
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if err := decoder.Decode(&original); err != nil {
		return err
	}
	if err := admitNestedResponse(original); err != nil {
		return err
	}
	type plain nestedResponseTargets
	var decoded plain
	if err := json.Unmarshal(raw, &decoded); err != nil {
		return err
	}
	*n = nestedResponseTargets(decoded)
	return nil
}
func admitNestedResponse(value any) error {
	root, err := closed(value, "roots declarations mappings", "")
	if err != nil {
		return err
	}
	fields, err := array(root["roots"])
	if err != nil {
		return err
	}
	checkField := func(raw any) error {
		f, e := closed(raw, "name type", "")
		if e != nil {
			return e
		}
		s, e := text(f["name"])
		if e != nil || !nestedResponseMember.MatchString(s) {
			return fmt.Errorf("invalid nested response member")
		}
		t, e := text(f["type"])
		if e != nil {
			return e
		}
		_, e = nestedResponseType(t, 0)
		return e
	}
	for _, field := range fields {
		if err = checkField(field); err != nil {
			return err
		}
	}
	declarations, ok := root["declarations"].(map[string]any)
	if !ok {
		return fmt.Errorf("invalid nested response declarations")
	}
	for key, raw := range declarations {
		if err = name(key, false); err != nil {
			return err
		}
		body, ok := raw.(map[string]any)
		if !ok {
			return fmt.Errorf("invalid structural declaration")
		}
		required := "kind"
		switch body["kind"] {
		case "newtype":
			required += " of"
		case "struct":
			required += " fields"
		case "enum":
			required += " variants"
		case "union":
			required += " tag variants"
		default:
			return fmt.Errorf("invalid structural declaration kind")
		}
		if _, err = closed(raw, required, ""); err != nil {
			return err
		}
		if value, exists := body["fields"]; exists {
			items, e := array(value)
			if e != nil {
				return e
			}
			for _, field := range items {
				if e = checkField(field); e != nil {
					return e
				}
			}
		}
	}
	mappings, err := array(root["mappings"])
	if err != nil {
		return err
	}
	for _, raw := range mappings {
		mapping, e := closed(raw, "target source", "")
		if e != nil {
			return e
		}
		source, e := text(mapping["source"])
		if e != nil || !nestedResponseMember.MatchString(source) {
			return fmt.Errorf("invalid nested response source")
		}
		path, e := array(mapping["target"])
		if e != nil {
			return e
		}
		if len(path) < 2 || len(path) > 33 {
			return fmt.Errorf("nested response path requires 2 through 33 segments")
		}
		for _, value := range path {
			member, e := text(value)
			if e != nil || !nestedResponseMember.MatchString(member) {
				return fmt.Errorf("invalid nested response member")
			}
		}
	}
	return nil
}

// Match the source TypeRef parser, including its finite wrapper depth and canonical spacing.
func nestedResponseType(raw string, depth int) (string, error) {
	if depth > 32 {
		return "", fmt.Errorf("nested response type wrapper depth")
	}
	raw = strings.TrimSpace(raw)
	for _, wrapper := range []string{"Optional", "List", "Map"} {
		prefix := wrapper + "<"
		if !strings.HasPrefix(raw, prefix) || !strings.HasSuffix(raw, ">") {
			continue
		}
		inner := raw[len(prefix) : len(raw)-1]
		if wrapper == "Map" {
			parts := strings.SplitN(inner, ",", 2)
			if len(parts) != 2 {
				return "", fmt.Errorf("invalid structural map")
			}
			key := strings.TrimSpace(parts[0])
			if !accessorPrimitive(key) || key == "Binary64" {
				return "", fmt.Errorf("invalid structural map key")
			}
			value, e := nestedResponseType(parts[1], depth+1)
			if e != nil {
				return "", e
			}
			return "Map<" + key + ", " + value + ">", nil
		}
		value, e := nestedResponseType(inner, depth+1)
		if e != nil {
			return "", e
		}
		return wrapper + "<" + value + ">", nil
	}
	if accessorPrimitive(raw) || raw == "Json" {
		return raw, nil
	}
	if err := name(raw, false); err != nil {
		return "", err
	}
	return raw, nil
}
func normalizeNestedDeclaration(body selectionDeclaration) (selectionDeclaration, error) {
	var err error
	switch body.Kind {
	case "newtype":
		body.Of, err = nestedResponseType(body.Of, 0)
	case "struct":
		for i := range body.Fields {
			if body.Fields[i].Presence != "" {
				return body, fmt.Errorf("structural member metadata")
			}
			body.Fields[i].Type, err = nestedResponseType(body.Fields[i].Type, 0)
			if err != nil {
				return body, err
			}
		}
	case "union":
		var variants map[string]string
		if err = json.Unmarshal(body.Variants, &variants); err != nil {
			return body, err
		}
		for label, ty := range variants {
			variants[label], err = nestedResponseType(ty, 0)
			if err != nil {
				return body, err
			}
		}
		body.Variants, err = json.Marshal(variants)
	}
	return body, err
}
func normalizeNestedResponse(r *responseObservation) error {
	for _, fields := range [][]accessorField{r.Fields, r.Targets} {
		for i := range fields {
			value, e := nestedResponseType(fields[i].Type, 0)
			if e != nil {
				return e
			}
			fields[i].Type = value
		}
	}
	for key, body := range r.Declarations {
		value, e := normalizeNestedDeclaration(body)
		if e != nil {
			return e
		}
		r.Declarations[key] = value
	}
	for i := range r.Nested.Roots {
		value, e := nestedResponseType(r.Nested.Roots[i].Type, 0)
		if e != nil {
			return e
		}
		r.Nested.Roots[i].Type = value
	}
	for key, body := range r.Nested.Declarations {
		value, e := normalizeNestedDeclaration(body)
		if e != nil {
			return e
		}
		r.Nested.Declarations[key] = value
	}
	return nil
}
func nestedResponseReferences(body selectionDeclaration) ([]string, error) {
	switch body.Kind {
	case "newtype":
		return []string{body.Of}, nil
	case "struct":
		if len(body.Fields) == 0 {
			return nil, fmt.Errorf("empty structural struct")
		}
		seen := map[string]bool{}
		refs := []string{}
		for _, field := range body.Fields {
			if !nestedResponseMember.MatchString(field.Name) || seen[field.Name] || field.Presence != "" {
				return nil, fmt.Errorf("invalid structural member")
			}
			seen[field.Name] = true
			refs = append(refs, field.Type)
		}
		return refs, nil
	case "enum":
		var labels []string
		if json.Unmarshal(body.Variants, &labels) != nil || len(labels) == 0 {
			return nil, fmt.Errorf("invalid structural enum")
		}
		return nil, nil
	case "union":
		var variants map[string]string
		if body.Tag == "" || json.Unmarshal(body.Variants, &variants) != nil || len(variants) == 0 {
			return nil, fmt.Errorf("invalid structural union")
		}
		refs := []string{}
		for _, ty := range variants {
			refs = append(refs, ty)
		}
		return refs, nil
	}
	return nil, fmt.Errorf("invalid structural kind")
}
func nestedResponseNamed(ty string) string {
	for {
		if inner, ok := accessorOptional(ty); ok {
			ty = inner
			continue
		}
		if inner, ok, _ := accessorCollection(ty); ok {
			ty = inner
			continue
		}
		if accessorPrimitive(ty) || ty == "Json" {
			return ""
		}
		return ty
	}
}
func (n nestedResponseTargets) validate(r responseObservation) error {
	if len(n.Roots) == 0 || len(n.Roots) > 256 || len(n.Mappings) == 0 || len(n.Mappings)+len(r.Mappings) > 256 {
		return fmt.Errorf("nested response relationship bound")
	}
	names := map[string]bool{}
	for key := range r.Declarations {
		names[key] = true
	}
	for key, body := range n.Declarations {
		names[key] = true
		if old, ok := r.Declarations[key]; ok && nestedDeclarationCanonical(old) != nestedDeclarationCanonical(body) {
			return fmt.Errorf("conflicting nested response declaration")
		}
	}
	if len(names) > 4096 {
		return fmt.Errorf("nested response declaration union limit")
	}
	roots := map[string]string{}
	pending := []string{}
	for _, root := range n.Roots {
		if !nestedResponseMember.MatchString(root.Name) || roots[root.Name] != "" {
			return fmt.Errorf("duplicate nested response root")
		}
		if _, ok := r.Mappings[root.Name]; ok {
			return fmt.Errorf("overlapping nested response root")
		}
		roots[root.Name] = root.Type
		pending = append(pending, root.Type)
	}
	visited := map[string]bool{}
	for len(pending) > 0 {
		ty := pending[len(pending)-1]
		pending = pending[:len(pending)-1]
		parsed, err := nestedResponseType(ty, 0)
		if err != nil {
			return err
		}
		key := nestedResponseNamed(parsed)
		if key == "" || visited[key] {
			continue
		}
		body, ok := n.Declarations[key]
		if !ok {
			return fmt.Errorf("missing structural declaration")
		}
		visited[key] = true
		refs, err := nestedResponseReferences(body)
		if err != nil {
			return err
		}
		pending = append(pending, refs...)
	}
	if len(visited) != len(n.Declarations) {
		return fmt.Errorf("unrelated structural declarations")
	}
	used := map[string]bool{}
	paths := [][]string{}
	memo := map[string]string{}
	for _, mapping := range n.Mappings {
		path := mapping.Target
		if len(path) < 2 || len(path) > 33 {
			return fmt.Errorf("nested response path bound")
		}
		for _, member := range path {
			if !nestedResponseMember.MatchString(member) {
				return fmt.Errorf("invalid nested response member")
			}
		}
		ty, ok := roots[path[0]]
		if !ok {
			return fmt.Errorf("undeclared nested response root")
		}
		used[path[0]] = true
		for _, member := range path[1:] {
			key, err := n.structName(ty, memo)
			if err != nil {
				return err
			}
			found := false
			for _, field := range n.Declarations[key].Fields {
				if field.Name == member {
					ty = field.Type
					found = true
					break
				}
			}
			if !found {
				return fmt.Errorf("undeclared nested response member")
			}
		}
		from := ""
		for _, field := range r.Fields {
			if field.Name == mapping.Source {
				from = field.Type
				break
			}
		}
		if from == "" || !responseAssignable(from, ty) {
			return fmt.Errorf("nested response terminal mismatch")
		}
		for _, other := range paths {
			short := len(path)
			if len(other) < short {
				short = len(other)
			}
			same := true
			for i := 0; i < short; i++ {
				if path[i] != other[i] {
					same = false
					break
				}
			}
			if same {
				return fmt.Errorf("duplicate or overlapping nested response paths")
			}
		}
		paths = append(paths, path)
	}
	if len(used) != len(roots) {
		return fmt.Errorf("unused nested response root")
	}
	return nil
}
func (n nestedResponseTargets) structName(ty string, memo map[string]string) (string, error) {
	walked := map[string]bool{}
	for {
		if key, ok := memo[ty]; ok {
			for prior := range walked {
				memo[prior] = key
			}
			return key, nil
		}
		if walked[ty] {
			return "", fmt.Errorf("cyclic nested response ancestor")
		}
		walked[ty] = true
		if inner, ok := accessorOptional(ty); ok {
			ty = inner
			continue
		}
		body, ok := n.Declarations[ty]
		if !ok {
			return "", fmt.Errorf("nested response ancestor is not a struct")
		}
		switch body.Kind {
		case "newtype":
			ty = body.Of
		case "struct":
			for prior := range walked {
				memo[prior] = ty
			}
			return ty, nil
		default:
			return "", fmt.Errorf("nested response ancestor is not a struct")
		}
	}
}
func (n nestedResponseTargets) compare(r responseObservation, response, payload map[string]Node) error {
	for _, mapping := range n.Mappings {
		object := payload
		for _, member := range mapping.Target[:len(mapping.Target)-1] {
			next, ok := object[member].(map[string]any)
			if !ok {
				return fmt.Errorf("nested response ancestor %s is absent or not an object", member)
			}
			object = next
		}
		emitted, present := object[mapping.Target[len(mapping.Target)-1]]
		actual, exists := response[mapping.Source]
		optional := false
		for _, field := range r.Fields {
			if field.Name == mapping.Source {
				_, optional = accessorOptional(field.Type)
			}
		}
		if optional && (!exists || actual == nil) && (!present || emitted == nil) {
			continue
		}
		if !exists || !present || !responseEqual(actual, emitted) {
			return fmt.Errorf("event path %s differs from actual response field %s", strings.Join(mapping.Target, "."), mapping.Source)
		}
	}
	return nil
}

// Rust-compatible compact UTF-8 JSON: active declaration members only, no HTML or U+2028 escaping.
func nestedJSONString(value string) string {
	var out strings.Builder
	out.WriteByte('"')
	for _, r := range value {
		switch r {
		case '"':
			out.WriteString(`\"`)
		case '\\':
			out.WriteString(`\\`)
		case '\b':
			out.WriteString(`\b`)
		case '\t':
			out.WriteString(`\t`)
		case '\n':
			out.WriteString(`\n`)
		case '\f':
			out.WriteString(`\f`)
		case '\r':
			out.WriteString(`\r`)
		default:
			if r < 32 {
				out.WriteString(fmt.Sprintf(`\u%04x`, r))
			} else {
				out.WriteRune(r)
			}
		}
	}
	out.WriteByte('"')
	return out.String()
}
func nestedStringList(values []string) string {
	parts := make([]string, 0, len(values))
	for _, value := range values {
		parts = append(parts, nestedJSONString(value))
	}
	return "[" + strings.Join(parts, ",") + "]"
}
func nestedFieldsCanonical(fields []accessorField, presence bool) string {
	parts := make([]string, 0, len(fields))
	for _, field := range fields {
		value := `{"name":` + nestedJSONString(field.Name) + `,"type":` + nestedJSONString(field.Type)
		if presence && field.Presence != "" {
			value += `,"presence":` + nestedJSONString(field.Presence)
		}
		parts = append(parts, value+"}")
	}
	return "[" + strings.Join(parts, ",") + "]"
}
func nestedStringMap(values map[string]string) string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, key := range keys {
		parts = append(parts, nestedJSONString(key)+":"+nestedJSONString(values[key]))
	}
	return "{" + strings.Join(parts, ",") + "}"
}
func nestedDeclarationCanonical(body selectionDeclaration) string {
	value := `{"kind":` + nestedJSONString(body.Kind)
	switch body.Kind {
	case "newtype":
		value += `,"of":` + nestedJSONString(body.Of)
	case "struct":
		value += `,"fields":` + nestedFieldsCanonical(body.Fields, false)
	case "enum":
		var labels []string
		_ = json.Unmarshal(body.Variants, &labels)
		value += `,"variants":` + nestedStringList(labels)
	case "union":
		var variants map[string]string
		_ = json.Unmarshal(body.Variants, &variants)
		value += `,"tag":` + nestedJSONString(body.Tag) + `,"variants":` + nestedStringMap(variants)
	}
	return value + "}"
}
func nestedDeclarationsCanonical(values map[string]selectionDeclaration) string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, key := range keys {
		parts = append(parts, nestedJSONString(key)+":"+nestedDeclarationCanonical(values[key]))
	}
	return "{" + strings.Join(parts, ",") + "}"
}
func nestedResponseCanonical(r responseObservation) string {
	roots := make([]accessorField, 0, len(r.Nested.Roots))
	for _, field := range r.Nested.Roots {
		roots = append(roots, accessorField{Name: field.Name, Type: field.Type})
	}
	mappings := make([]string, 0, len(r.Nested.Mappings))
	for _, mapping := range r.Nested.Mappings {
		mappings = append(mappings, `{"target":`+nestedStringList(mapping.Target)+`,"source":`+nestedJSONString(mapping.Source)+"}")
	}
	return `{"command":` + nestedJSONString(r.Command) + `,"outcome":{"command":` + nestedJSONString(r.Outcome.Command) + `,"outcome":` + nestedJSONString(r.Outcome.Outcome) + `},"event":` + nestedJSONString(r.Event) + `,"fields":` + nestedFieldsCanonical(r.Fields, true) + `,"declarations":` + nestedDeclarationsCanonical(r.Declarations) + `,"mappings":` + nestedStringMap(r.Mappings) + `,"targets":` + nestedFieldsCanonical(r.Targets, false) + `,"nested":{"roots":` + nestedFieldsCanonical(roots, false) + `,"declarations":` + nestedDeclarationsCanonical(r.Nested.Declarations) + `,"mappings":[` + strings.Join(mappings, ",") + "]}}"
}
