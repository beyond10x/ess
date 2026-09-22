// FixtureContract is source-owned type authority, resolved before scenario activity.
type FixtureContract struct {
	Fields       []accessorField                 `json:"fields"`
	Declarations map[string]selectionDeclaration `json:"declarations"`
}

// FixtureProvider supplies independently provisioned values. It owns provisioning cleanup.
type FixtureProvider interface {
	FixtureValues(ScenarioContext, FixtureContract) (map[string]Node, error)
}

func fixtureName(value any) error {
	label, ok := value.(string)
	if !ok || len(label) > 128 || !regexp.MustCompile(`^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$`).MatchString(label) {
		return fmt.Errorf("invalid fixture name")
	}
	return nil
}

func (c *FixtureContract) UnmarshalJSON(raw []byte) error {
	type plain FixtureContract
	var value plain
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return err
	}
	*c = FixtureContract(value)
	return c.validate()
}

func (c FixtureContract) validate() error {
	if len(c.Fields) == 0 || len(c.Fields) > 256 || len(c.Declarations) > 4096 {
		return fmt.Errorf("fixture field/declaration bound")
	}
	for _, field := range c.Fields {
		if err := fixtureName(field.Name); err != nil {
			return err
		}
	}
	if err := validateTypedFields([][]accessorField{c.Fields}, c.Declarations); err != nil {
		return err
	}
	raw, err := json.Marshal(c)
	if err != nil {
		return err
	}
	if len(raw) > 1048576 {
		return fmt.Errorf("fixture contract byte limit")
	}
	return nil
}

func admitFixtures(value any) error {
	root, err := closed(value, "fields declarations", "")
	if err != nil {
		return err
	}
	if err := admitTypedDeclarations(root["declarations"]); err != nil {
		return err
	}
	fields, err := array(root["fields"])
	if err != nil {
		return err
	}
	for _, raw := range fields {
		field, err := closed(raw, "name type", "")
		if err != nil {
			return err
		}
		if err := fixtureName(field["name"]); err != nil {
			return err
		}
		if _, err := text(field["type"]); err != nil {
			return err
		}
	}
	raw, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var contract FixtureContract
	return json.Unmarshal(raw, &contract)
}

func copyFixtureValue(value Node) (Node, error) {
	raw, err := json.Marshal(value)
	if err != nil {
		return nil, err
	}
	if len(raw) > 1048576 {
		return nil, fmt.Errorf("fixture value byte limit")
	}
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	var copied Node
	err = decoder.Decode(&copied)
	return copied, err
}

func (c FixtureContract) values(provided map[string]Node) (map[string]Node, error) {
	if err := c.validate(); err != nil {
		return nil, err
	}
	if len(provided) != len(c.Fields) {
		return nil, fmt.Errorf("fixture result must contain exactly the declared names")
	}
	copied, err := copyFixtureValue(provided)
	if err != nil {
		return nil, err
	}
	values, ok := copied.(map[string]any)
	if !ok {
		return nil, fmt.Errorf("fixture values must be an object")
	}
	observer := selectionObservation{Declarations: c.Declarations, responseMode: true}
	bytes := 0
	for _, field := range c.Fields {
		value, exists := values[field.Name]
		if !exists {
			return nil, fmt.Errorf("missing fixture %s", field.Name)
		}
		if err := observer.validateValue(field.Type, value, exists, &bytes, 0); err != nil {
			return nil, err
		}
	}
	if bytes > 1048576 {
		return nil, fmt.Errorf("fixture value byte limit")
	}
	return values, nil
}

func (r *run) resolveFixtures(context ScenarioContext, contract *FixtureContract) error {
	provider, ok := r.target.(FixtureProvider)
	if !ok {
		return fmt.Errorf("%w: no pre-execution fixture provider", ErrUnsupported)
	}
	// Give the provider its own contract copy, never mutable assertion authority.
	raw, err := json.Marshal(contract)
	if err != nil {
		return err
	}
	var request FixtureContract
	if err := json.Unmarshal(raw, &request); err != nil {
		return err
	}
	provided, err := provider.FixtureValues(context, request)
	if err != nil {
		return err
	}
	r.fixtures, err = contract.values(provided)
	return err
}

func admitFixtureSteps(steps []any) error {
	declared, referenced := map[string]bool{}, map[string]bool{}
	for index, raw := range steps {
		step := raw.(map[string]any)
		if step["step"] == "resolve_fixtures" {
			if index != 0 {
				return fmt.Errorf("fixture resolution must be the first and only prelude")
			}
			contract := step["fixtures"].(map[string]any)
			for _, raw := range contract["fields"].([]any) {
				declared[raw.(map[string]any)["name"].(string)] = true
			}
		}
		groups := []any{}
		switch step["step"] {
		case "execute_command", "expect_invocation":
			groups = append(groups, step["input"])
		case "snapshot_subject":
			groups = append(groups, step["subject"])
		case "query_view", "eventually_view", "expect_halt", "eventually_halt":
			groups = append(groups, step["params"])
		case "expect_event_values":
			if len(step["payload"].(map[string]any)) == 0 {
				return fmt.Errorf("empty event value assertion")
			}
			groups = append(groups, step["payload"])
		}
		if step["step"] == "expect_view" || step["step"] == "eventually_view" {
			groups = append(groups, step["expectation"].(map[string]any)["fields"])
		}
		for _, raw := range groups {
			if group, ok := raw.(map[string]any); ok {
				for _, raw := range group {
					value := raw.(map[string]any)
					if value["kind"] == "fixture" {
						referenced[value["fixture"].(string)] = true
					}
				}
			}
		}
	}
	if len(declared) != len(referenced) {
		return fmt.Errorf("fixture declarations must exactly match the referenced names")
	}
	for key := range referenced {
		if !declared[key] {
			return fmt.Errorf("unbound fixture %s", key)
		}
	}
	return nil
}

func (r *run) expectEventValues(index int, step Step) bool {
	// Payload is decoded as Node for legacy literal steps; this step explicitly admits Value.
	raw, err := json.Marshal(step.Payload)
	if err != nil {
		return r.fail(index, "fixture event payload: %v", err)
	}
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	var values map[string]Value
	if err := decoder.Decode(&values); err != nil {
		return r.fail(index, "fixture event values: %v", err)
	}
	payload, ok := r.resolveAll(index, values)
	if !ok {
		return false
	}
	// Match the Rust runner: select the first direct occurrence by name, never by its values.
	for _, event := range r.last.DirectEvents {
		if event.Event != step.Event {
			continue
		}
		if !matches(event.Payload, payload) {
			return r.fail(index, "`%s` carried different fixture values", step.Event)
		}
		if reason := holds(event.Payload, step.Shape); reason != "" {
			return r.fail(index, "`%s` was emitted, and %s", step.Event, reason)
		}
		return true
	}
	return r.fail(index, "`%s` was not emitted", step.Event)
}
