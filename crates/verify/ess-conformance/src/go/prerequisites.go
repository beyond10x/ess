// Suite/28–33: complete direct responses, channel context and structured references.

// The suite envelope does not consume a direct expected field's local depth128 budget.
// Only this finite envelope path resets depth, exactly as count_json::Scope does.
func strictSuiteJSON(raw string) (any, error) {
	var probe struct {
		Provenance struct {
			SuiteVersion string `json:"suite_version"`
		} `json:"provenance"`
	}
	if json.Unmarshal([]byte(raw), &probe) != nil || suiteMajor(probe.Provenance.SuiteVersion) < 28 {
		return strictJSON(raw)
	}
	if !utf8.ValidString(raw) {
		return nil, fmt.Errorf("input is not UTF-8")
	}
	if err := scalarStrings(raw); err != nil {
		return nil, err
	}
	return suiteJSONNode([]byte(raw), 0, "suite")
}
func suiteJSONNode(raw []byte, depth int, scope string) (any, error) {
	if depth > 128 {
		return nil, fmt.Errorf("JSON nesting exceeds128")
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	token, err := decoder.Token()
	if err != nil {
		return nil, err
	}
	switch token {
	case json.Delim('{'):
		fields := map[string]json.RawMessage{}
		for decoder.More() {
			key, err := decoder.Token()
			if err != nil {
				return nil, err
			}
			name := key.(string)
			if _, exists := fields[name]; exists {
				return nil, fmt.Errorf("duplicate key")
			}
			var child json.RawMessage
			if err = decoder.Decode(&child); err != nil {
				return nil, err
			}
			fields[name] = child
		}
		if _, err = decoder.Token(); err != nil {
			return nil, err
		}
		var tag string
		_ = json.Unmarshal(fields["step"], &tag)
		object := map[string]any{}
		for key, child := range fields {
			next := "plain"
			childDepth := depth + 1
			switch {
			case scope == "suite" && key == "scenarios":
				next = "scenarios"
			case scope == "scenarios":
				next = "scenario"
			case scope == "scenario" && key == "steps":
				next = "steps"
			case scope == "step" && key == "response" && tag == "expect_direct_response":
				next = "response"
			case scope == "response" && key == "expected":
				next = "expected"
			case scope == "expected":
				childDepth = 0
			}
			value, err := suiteJSONNode(child, childDepth, next)
			if err != nil {
				return nil, err
			}
			object[key] = value
		}
		if _, err = decoder.Token(); err != io.EOF {
			return nil, fmt.Errorf("trailing JSON input")
		}
		return object, nil
	case json.Delim('['):
		array := []any{}
		for decoder.More() {
			var child json.RawMessage
			if err = decoder.Decode(&child); err != nil {
				return nil, err
			}
			next := "plain"
			if scope == "steps" {
				next = "step"
			}
			value, err := suiteJSONNode(child, depth+1, next)
			if err != nil {
				return nil, err
			}
			array = append(array, value)
		}
		if _, err = decoder.Token(); err != nil {
			return nil, err
		}
		if _, err = decoder.Token(); err != io.EOF {
			return nil, fmt.Errorf("trailing JSON input")
		}
		return array, nil
	default:
		if _, err = decoder.Token(); err != io.EOF {
			return nil, fmt.Errorf("trailing JSON input")
		}
		return token, nil
	}
}

type directResponseObservation struct {
	Command      string                          `json:"command"`
	Outcome      *OutcomeRef                     `json:"outcome"`
	Fields       []accessorField                 `json:"fields"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
	Expected     map[string]Node                 `json:"expected"`
}

func directPresence(field accessorField, value Node, present bool) error {
	if field.Presence == "null_when_absent" && !present || field.Presence == "omitted_when_absent" && present && value == nil {
		return fmt.Errorf("invalid_presence")
	}
	return nil
}

func directField(value any) (map[string]any, error) {
	field, ok := value.(map[string]any)
	if !ok {
		return nil, fmt.Errorf("invalid response field")
	}
	field = copyObject(field)
	if raw := field["naming"]; raw != nil {
		naming, err := closed(raw, "", "wire display summary code")
		if err != nil {
			return nil, err
		}
		for _, key := range []string{"wire", "display", "summary", "code"} {
			if field[key] != nil {
				return nil, fmt.Errorf("nested naming conflicts with flat naming")
			}
		}
		delete(field, "naming")
		for key, value := range naming {
			field[key] = value
		}
	}
	delete(field, "naming")
	if code := field["code"]; code != nil {
		if _, err := text(code); err != nil {
			return nil, err
		}
	}
	plain, err := directPlainField(field)
	if err != nil {
		return nil, err
	}
	if err = admitAccessorField(plain); err != nil {
		return nil, err
	}
	return field, nil
}

func directPlainField(value any) (any, error) {
	field := copyObject(value.(map[string]any))
	if field["presence"] == nil {
		delete(field, "presence")
	}
	plain, err := responseFieldPresence(field)
	if err != nil {
		return nil, err
	}
	field = copyObject(plain.(map[string]any))
	delete(field, "code")
	return field, nil
}

func admitDirectResponse(value any) (*directResponseObservation, error) {
	root, err := closed(value, "command fields declarations expected", "outcome")
	if err != nil {
		return nil, err
	}
	if err = name(root["command"], false); err != nil {
		return nil, err
	}
	if root["outcome"] != nil {
		if err = admitOutcome(root["outcome"]); err != nil {
			return nil, err
		}
	}
	fields, err := array(root["fields"])
	if err != nil {
		return nil, err
	}
	root = copyObject(root)
	normalFields := []any{}
	for _, field := range fields {
		field, e := directField(field)
		if e != nil {
			return nil, e
		}
		normalFields = append(normalFields, field)
		plain, e := directPlainField(field)
		if e != nil {
			return nil, e
		}
		if e = admitAccessorField(plain); e != nil {
			return nil, e
		}
	}
	root["fields"] = normalFields
	declarations, ok := root["declarations"].(map[string]any)
	if !ok {
		return nil, fmt.Errorf("direct response declarations must be object")
	}
	// Nested field presence is part of this profile. Validate the closed declarations with
	// presence removed, then retain the original authority for observation.
	plain := map[string]any{}
	normalDeclarations := map[string]any{}
	for key, value := range declarations {
		body, ok := value.(map[string]any)
		if !ok {
			return nil, fmt.Errorf("invalid declaration")
		}
		copy := copyObject(body)
		normal := copyObject(body)
		if body["kind"] == "struct" {
			members, e := array(body["fields"])
			if e != nil {
				return nil, e
			}
			clean := []any{}
			normalMembers := []any{}
			for _, member := range members {
				member, e := directField(member)
				if e != nil {
					return nil, e
				}
				normalMembers = append(normalMembers, member)
				field, e := directPlainField(member)
				if e != nil {
					return nil, e
				}
				clean = append(clean, field)
			}
			copy["fields"] = clean
			normal["fields"] = normalMembers
		}
		plain[key] = copy
		normalDeclarations[key] = normal
	}
	root["declarations"] = normalDeclarations
	if err = admitTypedDeclarations(plain); err != nil {
		return nil, err
	}
	if _, ok := root["expected"].(map[string]any); !ok {
		return nil, fmt.Errorf("direct expectations must be object")
	}
	raw, err := directWire(root)
	if err != nil {
		return nil, err
	}
	if accessorCompactBytes(raw) > 1048576 {
		return nil, fmt.Errorf("direct response contract byte limit")
	}
	var result directResponseObservation
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if err = decoder.Decode(&result); err != nil {
		return nil, err
	}
	if len(result.Fields) == 0 || len(result.Fields) > 256 || len(result.Declarations) > 4096 {
		return nil, fmt.Errorf("direct response contract bounds")
	}
	if result.Outcome != nil && result.Outcome.Command != result.Command {
		return nil, fmt.Errorf("direct outcome owner differs")
	}
	if err = validateTypedFields([][]accessorField{result.Fields}, result.Declarations); err != nil {
		return nil, err
	}
	observer := selectionObservation{Declarations: result.Declarations, responseMode: true, directMode: true}
	count := 0
	for name, value := range result.Expected {
		found := false
		for _, field := range result.Fields {
			if field.Name == name {
				found = true
				if err = observer.validateValue(field.Type, value, true, &count, 0); err != nil {
					return nil, err
				}
				if err = directPresence(field, value, true); err != nil {
					return nil, err
				}
			}
		}
		if !found {
			return nil, fmt.Errorf("undeclared direct expected field")
		}
	}
	return &result, nil
}

func (r directResponseObservation) compare(actual map[string]Node) error {
	if actual == nil {
		return fmt.Errorf("command returned no response")
	}
	names := map[string]bool{}
	for _, field := range r.Fields {
		names[field.Name] = true
	}
	for key := range actual {
		if !names[key] {
			return fmt.Errorf("response has undeclared field")
		}
	}
	observer := selectionObservation{Declarations: r.Declarations, responseMode: true, directMode: true}
	count := 0
	for _, field := range r.Fields {
		value, present := actual[field.Name]
		if err := observer.validateValue(field.Type, value, present, &count, 0); err != nil {
			return err
		}
		if err := directPresence(field, value, present); err != nil {
			return err
		}
	}
	raw, err := directWire(actual)
	if err != nil || accessorCompactBytes(raw) > 1048576 {
		return fmt.Errorf("direct response resource byte limit")
	}
	for name, value := range r.Expected {
		actual, present := actual[name]
		if !present || !responseEqual(value, actual) {
			return fmt.Errorf("response differs from declared literal")
		}
	}
	return nil
}

func (r *run) expectDirectResponse(index int, step Step) bool {
	observation := step.DirectResponse
	if observation == nil || r.lastCommand != observation.Command || observation.Outcome != nil && r.last.Outcome != observation.Outcome.Outcome || r.last.Error != "" {
		return r.assertionFailure(index, "ESS-CF-PAYLOAD: direct response invocation differs")
	}
	if err := observation.compare(r.last.Response); err != nil {
		return r.assertionFailure(index, "ESS-CF-PAYLOAD: %v", err)
	}
	return true
}

func admitDirectSteps(steps []any) error {
	command := ""
	for _, raw := range steps {
		step := raw.(map[string]any)
		switch step["step"] {
		case "execute_command", "execute_command_without_input":
			command = step["command"].(string)
		case "expect_direct_response":
			response, err := admitDirectResponse(step["response"])
			if err != nil {
				return err
			}
			if response.Command != command {
				return fmt.Errorf("InvalidResponse: direct response names no immediately preceding invocation")
			}
		}
	}
	return nil
}

func directJSON(value Node, count *int, depth int) error {
	*count++
	if depth > 128 || *count > 1048576 {
		return fmt.Errorf("resource")
	}
	switch value := value.(type) {
	case string:
		*count += len(value)
	case []any:
		if len(value) > 65536 {
			return fmt.Errorf("resource")
		}
		for _, child := range value {
			if err := directJSON(child, count, depth+1); err != nil {
				return err
			}
		}
	case map[string]any:
		if len(value) > 65536 {
			return fmt.Errorf("resource")
		}
		for key, child := range value {
			*count += len(key)
			if err := directJSON(child, count, depth+1); err != nil {
				return err
			}
		}
	}
	if *count > 1048576 {
		return fmt.Errorf("resource")
	}
	return nil
}

// EventDeliveryTarget is optional on older implementations, like its native counterpart.
type EventDeliveryTarget interface {
	DeliverEvent(EventDeliveryRequest) error
}
type EventDeliveryRequest struct {
	Event, Authority, Correlation string
	Payload, Context              map[string]Node
}

func (r *run) deliverEvent(index int, step Step) bool {
	target, ok := r.target.(EventDeliveryTarget)
	if !ok {
		r.skip("ESS-CF-TARGET: target cannot deliver external events")
		return false
	}
	err := target.DeliverEvent(EventDeliveryRequest{Event: step.Event, Authority: step.Authority, Correlation: r.correlation, Payload: step.Payload, Context: step.Context})
	if err != nil {
		return r.targetFailure(index, err, "delivering external event")
	}
	return true
}
func (r *run) expectEveryInvocation(index int, step Step) bool {
	selected, ok := r.resolveAll(index, step.Selecting)
	if !ok {
		return false
	}
	wanted, absent, ok := r.invocationWanted(index, step)
	if !ok {
		return false
	}
	attempts := r.harness.Deadline().Attempts
	for attempt := 0; attempt < attempts; attempt++ {
		seen, err := r.target.ObserveInvocations(InvocationObservationRequest{Binding: step.Binding, Command: step.Command, Correlation: r.correlation, Deadline: Deadline{Attempts: attempts - attempt}})
		if err != nil {
			if errors.Is(err, ErrUnsupported) {
				return r.unsupportedObservation(index, err)
			}
			return r.targetFailure(index, err, "observing every invocation")
		}
		chosen := 0
		for _, invocation := range seen {
			if invocation.Command != step.Command || !matches(invocation.Input, selected) {
				continue
			}
			chosen++
			if !matches(invocation.Input, wanted) {
				return r.assertionFailure(index, "ESS-CF-INVOCATION: delivery context differs")
			}
			for _, field := range absent {
				if _, exists := invocation.Input[field]; exists {
					return r.assertionFailure(index, "ESS-CF-INVOCATION: expected input absence")
				}
			}
		}
		if attempt+1 == attempts {
			if chosen > 0 {
				return true
			}
			return r.assertionFailure(index, "ESS-CF-INVOCATION: no invocation for occurrence")
		}
	}
	return r.assertionFailure(index, "ESS-CF-INVOCATION: empty observation window")
}

func admitStructured(value any, major, depth int) error {
	if major < 32 {
		return fmt.Errorf("structured values require suite/32 or /33")
	}
	if depth > 64 {
		return fmt.Errorf("InvalidStructuredValue: depth limit")
	}
	body, ok := value.(map[string]any)
	if !ok {
		return fmt.Errorf("structured value must be object")
	}
	var children []any
	switch body["kind"] {
	case "list":
		if _, err := closed(body, "kind items", ""); err != nil {
			return err
		}
		items, err := array(body["items"])
		if err != nil {
			return err
		}
		children = items
	case "members":
		if _, err := closed(body, "kind members", ""); err != nil {
			return err
		}
		members, ok := body["members"].(map[string]any)
		if !ok {
			return fmt.Errorf("members must be object")
		}
		for _, child := range members {
			children = append(children, child)
		}
	default:
		return fmt.Errorf("invalid structured value")
	}
	for _, child := range children {
		body, ok := child.(map[string]any)
		if !ok {
			return fmt.Errorf("invalid structured child")
		}
		switch body["kind"] {
		case "list", "members":
			if err := admitStructured(child, major, depth+1); err != nil {
				return err
			}
		case "literal", "instance":
			if err := admitValues(map[string]any{"child": child}, major, false); err != nil {
				return err
			}
		default:
			return fmt.Errorf("InvalidStructuredValue: forbidden child")
		}
	}
	return nil
}

func (r *run) targetFailure(index int, err error, operation string) bool {
	if errors.Is(err, ErrUnsupported) {
		r.skip("ESS-CF-TARGET: %s: %v", operation, err)
		return false
	}
	r.recordStatus(statusError)
	r.t.Errorf("step %d: ESS-CF-TARGET: %s: %v", index, operation, err)
	return false
}

// Continuation belongs to this assertion's result, never the accumulated scenario status.
func (r *run) assertionFailure(index int, format string, args ...any) bool {
	r.fail(index, format, args...)
	return r.continuedAssertions
}
func (r *run) resolutionFailure(index int, field string, err error) bool {
	if !r.preciseStatuses {
		return r.fail(index, "`%s`: %v", field, err)
	}
	r.recordStatus(statusError)
	r.t.Errorf("step %d: ESS-CF-SUITE: resolving `%s`: %v", index, field, err)
	return false
}

// Preserve the boundary between callback errors and malformed evidence in helpers which
// perform both. Errors returned by a target are unavailable execution, not contradictions.
type callbackError struct{ cause error }

func (e callbackError) Error() string { return e.cause.Error() }
func (e callbackError) Unwrap() error { return e.cause }
func fromCallback(err error) error {
	if err == nil {
		return nil
	}
	return callbackError{err}
}

func (r *run) recordStatus(status string) {
	if r.disclosure != nil && (status == statusError || status == statusUnsupported) {
		r.disclosureUnavailable = true
	}
	if !r.preciseStatuses {
		r.status = status
		return
	}
	rank := map[string]int{statusPassed: 0, statusSkipped: 0, statusUnsupported: 1, statusError: 2, statusFailed: 3}
	if rank[status] > rank[r.status] {
		r.status = status
	}
}

// Native observation assertions continue after unavailable capability; a later assertion may
// still establish a contradiction. Executing commands and delivering events stop on errors.
func (r *run) unsupportedObservation(index int, err error) bool {
	r.recordStatus(statusUnsupported)
	r.t.Errorf("step %d: ESS-CF-INVOCATION: unsupported observation: %v", index, err)
	return true
}

func directWire(value any) ([]byte, error) {
	var out bytes.Buffer
	encoder := json.NewEncoder(&out)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(value); err != nil {
		return nil, err
	}
	return bytes.TrimSuffix(out.Bytes(), []byte{'\n'}), nil
}

func snapshotDirectResult(result CommandResult) (CommandResult, error) {
	// Only the declared response consumes its direct-return budget, not the result envelope.
	response, err := directWire(result.Response)
	if err != nil || accessorCompactBytes(response) > 1048576 {
		return CommandResult{}, fmt.Errorf("direct response resource byte limit")
	}
	raw, err := directWire(result)
	if err != nil {
		return CommandResult{}, err
	}
	var copied CommandResult
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	err = decoder.Decode(&copied)
	return copied, err
}
