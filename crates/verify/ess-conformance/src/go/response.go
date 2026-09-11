// Actual command responses are compared under immutable typed suite authority.
type responseObservation struct {
	Command      string                          `json:"command"`
	Outcome      OutcomeRef                      `json:"outcome"`
	Event        string                          `json:"event"`
	Fields       []accessorField                 `json:"fields"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
	Mappings     map[string]string               `json:"mappings"`
	Targets      []accessorField                 `json:"targets"`
}

func (r *responseObservation) UnmarshalJSON(raw []byte) error {
	type plain responseObservation
	var value plain
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return err
	}
	*r = responseObservation(value)
	return r.validate()
}
func (r responseObservation) validate() error {
	if r.Command != r.Outcome.Command || r.Outcome.Outcome == "" || len(r.Fields) == 0 || len(r.Fields) > 256 || len(r.Targets) > 256 || len(r.Declarations) > 4096 || len(r.Mappings) == 0 || len(r.Mappings) != len(r.Targets) {
		return fmt.Errorf("invalid response contract bounds or owner")
	}
	for _, label := range []string{r.Command, r.Event} {
		if err := name(label, false); err != nil {
			return err
		}
	}
	used := map[string]bool{}
	roots := map[string]string{}
	for _, fields := range [][]accessorField{r.Fields, r.Targets} {
		seen := map[string]bool{}
		for _, field := range fields {
			if field.Name == "" || seen[field.Name] {
				return fmt.Errorf("duplicate/empty response field")
			}
			seen[field.Name] = true
			if err := r.checkType(field.Type, used, map[string]bool{}, 0); err != nil {
				return err
			}
		}
	}
	for _, field := range r.Fields {
		roots[field.Name] = field.Type
	}
	if len(used) != len(r.Declarations) {
		return fmt.Errorf("unrelated response declarations")
	}
	for _, target := range r.Targets {
		source, ok := r.Mappings[target.Name]
		from, declared := roots[source]
		if !ok || !declared || !responseAssignable(from, target.Type) {
			return fmt.Errorf("invalid response mapping")
		}
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
func responseAssignable(from, to string) bool {
	if from == to {
		return true
	}
	if inner, ok := accessorOptional(to); ok {
		return responseAssignable(from, inner)
	}
	return false
}
func (r responseObservation) checkType(source string, used, stack map[string]bool, depth int) error {
	if depth > 128 {
		return fmt.Errorf("response type depth limit")
	}
	if inner, ok := accessorOptional(source); ok {
		return r.checkType(inner, used, stack, depth+1)
	}
	if strings.HasPrefix(source, "Map<") && !strings.HasPrefix(source, "Map<String, ") {
		return fmt.Errorf("response map key must be String")
	}
	if inner, ok, err := accessorCollection(source); ok || err != nil {
		if err != nil {
			return err
		}
		return r.checkType(inner, used, stack, depth+1)
	}
	if accessorPrimitive(source) && source != "Binary64" {
		return nil
	}
	body, ok := r.Declarations[source]
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
		if err := r.checkType(child, used, stack, depth+1); err != nil {
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

func admitResponse(value any) error {
	root, err := closed(value, "command outcome event fields declarations mappings targets", "")
	if err != nil {
		return err
	}
	if err = admitOutcome(root["outcome"]); err != nil {
		return err
	}
	declarations, ok := root["declarations"].(map[string]any)
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
	for _, key := range []string{"fields", "targets"} {
		fields, e := array(root[key])
		if e != nil {
			return e
		}
		for _, field := range fields {
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
func responseEqual(left, right Node) bool {
	switch value := left.(type) {
	case json.Number:
		a, ok := responseNumber(value)
		b, other := responseNumber(right)
		return ok && other && a.Cmp(b) == 0
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
