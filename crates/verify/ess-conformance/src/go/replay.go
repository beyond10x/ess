// A complete subject descriptor carries typed observation authority, never expected row data.
type subjectShape struct {
	IdentityField string                       `json:"identity_field"`
	Fields        []replayField                `json:"fields"`
	Declarations  map[string]replayDeclaration `json:"declarations"`
}

// Direct Step decoding preserves its legacy number and event-shape behavior.
// Only admitted suite12/13 steps select the exact numeric decoder below.
func (step *Step) UnmarshalJSON(raw []byte) error {
	decoded, err := decodeStep(raw, false)
	if err == nil {
		*step = decoded
	}
	return err
}

func decodeStep(raw []byte, exact bool) (Step, error) {
	type plain Step
	value := Step{}
	decoded := struct {
		*plain
		Shape json.RawMessage `json:"shape"`
	}{plain: (*plain)(&value)}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	if exact {
		decoder.UseNumber()
	}
	if err := decoder.Decode(&decoded); err != nil {
		return Step{}, err
	}
	if len(decoded.Shape) != 0 {
		if value.Step == "snapshot_complete_subject" {
			if err := json.Unmarshal(decoded.Shape, &value.CompleteShape); err != nil {
				return Step{}, err
			}
		} else if err := json.Unmarshal(decoded.Shape, &value.Shape); err != nil {
			return Step{}, err
		}
	}
	return value, nil
}

func decodeExactSteps(raw []json.RawMessage) ([]Step, error) {
	steps := make([]Step, 0, len(raw))
	for _, value := range raw {
		step, err := decodeStep(value, true)
		if err != nil {
			return nil, err
		}
		steps = append(steps, step)
	}
	return steps, nil
}

func decodeExactSuiteSteps(suite *Suite) error {
	if suite.Provenance.SuiteVersion != "ess-conformance/12" && suite.Provenance.SuiteVersion != "ess-conformance/13" && suite.Provenance.SuiteVersion != "ess-conformance/14" && suite.Provenance.SuiteVersion != "ess-conformance/15" && suite.Provenance.SuiteVersion != "ess-conformance/16" && suite.Provenance.SuiteVersion != "ess-conformance/17" {
		return nil
	}
	var raw struct {
		Scenarios map[string]struct {
			Steps []json.RawMessage `json:"steps"`
		} `json:"scenarios"`
	}
	if err := json.Unmarshal([]byte(suite.original), &raw); err != nil {
		return err
	}
	for id, scenario := range suite.Scenarios {
		steps, err := decodeExactSteps(raw.Scenarios[id].Steps)
		if err != nil {
			return err
		}
		scenario.Steps = steps
		suite.Scenarios[id] = scenario
	}
	return nil
}

func (shape subjectShape) schema() replayObservation {
	return replayObservation{Fields: shape.Fields, Declarations: shape.Declarations}
}

func (shape subjectShape) validate() error {
	if err := shape.schema().validateSchema(); err != nil {
		return err
	}
	for _, field := range shape.Fields {
		if field.Name == shape.IdentityField {
			if shape.optionalIdentity(field.Type, 0) {
				return fmt.Errorf("complete subject identity is optional")
			}
			return nil
		}
	}
	return fmt.Errorf("complete subject identity is not a required declared field")
}

func (shape subjectShape) optionalIdentity(source string, depth int) bool {
	if depth > 128 {
		return true
	}
	if _, optional := accessorOptional(source); optional {
		return true
	}
	if body, exists := shape.Declarations[source]; exists && body.Kind == "newtype" {
		return shape.optionalIdentity(body.Of, depth+1)
	}
	return false
}

func admitSubjectShape(value any) error {
	root, err := closed(value, "identity_field fields declarations", "")
	if err != nil {
		return err
	}
	if _, err := text(root["identity_field"]); err != nil {
		return err
	}
	if err := admitRetainedSchema(root); err != nil {
		return err
	}
	raw, err := json.Marshal(root)
	if err != nil {
		return err
	}
	var shape subjectShape
	if err := json.Unmarshal(raw, &shape); err != nil {
		return err
	}
	return shape.validate()
}

func (shape subjectShape) admitRow(row map[string]Node) error {
	observer := selectionObservation{Declarations: shape.schema().responseDeclarations(), responseMode: true}
	size := 0
	for _, field := range shape.Fields {
		value, present := row[field.Name]
		if err := observer.validateValue(field.Type, value, present, &size, 0); err != nil {
			return fmt.Errorf("%s: %v", field.Name, err)
		}
	}
	raw, err := json.Marshal(row)
	if err != nil {
		return err
	}
	if len(raw) > 1048576 || size > 1048576 {
		return fmt.Errorf("complete subject row exceeds retained byte bound")
	}
	return nil
}

func completeSubjectMatches(row, identity map[string]Node) bool {
	for field, expected := range identity {
		actual, present := row[field]
		if !present {
			return false
		}
		ownedActual, err := replaySnapshot(actual)
		if err != nil {
			return false
		}
		ownedExpected, err := replaySnapshot(expected)
		if err != nil || !responseEqual(ownedActual, ownedExpected) {
			return false
		}
	}
	return true
}

func (r *run) snapshotCompleteSubject(index int, step Step) bool {
	if r.queried != step.View {
		return r.fail(index, "complete subject requires a fresh query of %s", step.View)
	}
	capture := step.Step == "snapshot_complete_subject"
	var identity map[string]Node
	var shape *subjectShape
	if capture {
		var ok bool
		identity, ok = r.resolveAll(index, step.Subject)
		if !ok {
			return false
		}
		shape = step.CompleteShape
	} else {
		previous, ok := r.snapshots[step.View]
		if !ok || previous.shape == nil {
			return r.fail(index, "complete subject has no typed original snapshot")
		}
		identity, shape = previous.identity, previous.shape
	}
	if shape == nil || shape.validate() != nil {
		return r.fail(index, "complete subject has no valid descriptor")
	}
	var rows []map[string]Node
	for _, row := range r.lastView.Rows {
		if completeSubjectMatches(row, identity) {
			rows = append(rows, row)
		}
	}
	if len(rows) != 1 {
		return r.fail(index, "complete subject matched %d rows, want exactly one", len(rows))
	}
	// The owned wire-value copy keeps native i64 values exact and prevents target map reuse.
	row, err := replaySnapshot(rows[0])
	if err != nil {
		return r.fail(index, "complete subject snapshot: %v", err)
	}
	if capture {
		if err := shape.admitRow(row); err != nil {
			return r.fail(index, "complete subject row: %v", err)
		}
		ownedIdentity, err := replaySnapshot(identity)
		if err != nil {
			return r.fail(index, "complete subject identity: %v", err)
		}
		ownedShape, err := replaySnapshot(*shape)
		if err != nil {
			return r.fail(index, "complete subject descriptor: %v", err)
		}
		if r.snapshots == nil {
			r.snapshots = map[string]subjectSnapshot{}
		}
		r.snapshots[step.View] = subjectSnapshot{identity: ownedIdentity, shape: &ownedShape, complete: row}
		return true
	}
	if err := shape.admitRow(row); err != nil {
		return r.fail(index, "complete subject row: %v", err)
	}
	if !responseEqual(r.snapshots[step.View].complete, row) {
		return r.fail(index, "complete subject changed in %s", step.View)
	}
	return true
}

func admitCompleteSubjectSteps(steps []Step) error {
	type snapshot struct {
		invocation int
		instance   string
	}
	pending := map[string]snapshot{}
	instances := map[string]bool{}
	invocation, queriedAt := 0, -1
	queried := ""
	for _, step := range steps {
		switch step.Step {
		case "execute_command":
			for _, saved := range pending {
				if invocation > saved.invocation {
					return fmt.Errorf("extra command precedes complete subject comparison")
				}
			}
			invocation++
			queried = ""
		case "query_view":
			queried, queriedAt = step.View, invocation
		case "capture_instance", "establish_entity":
			for _, saved := range pending {
				if saved.instance == step.Instance {
					return fmt.Errorf("complete subject identity was overwritten")
				}
			}
			instances[step.Instance] = true
		case "snapshot_complete_subject":
			_, exists := pending[step.View]
			if step.CompleteShape == nil || queried != step.View || queriedAt != invocation || exists || len(step.Subject) != 1 {
				return fmt.Errorf("invalid or overwritten complete subject snapshot")
			}
			identity, ok := step.Subject[step.CompleteShape.IdentityField]
			if !ok || identity.Kind != "instance" || !instances[identity.Instance] {
				return fmt.Errorf("complete subject identity has no matching binding")
			}
			pending[step.View] = snapshot{invocation, identity.Instance}
		case "expect_complete_subject_unchanged":
			previous, exists := pending[step.View]
			if !exists || queried != step.View || queriedAt != invocation || invocation != previous.invocation+1 {
				return fmt.Errorf("complete subject comparison lacks its fresh original pair")
			}
			delete(pending, step.View)
		case "snapshot_subject", "expect_subject_unchanged":
			if _, exists := pending[step.View]; exists {
				return fmt.Errorf("legacy subject step cannot replace complete observation")
			}
		}
	}
	if len(pending) != 0 {
		return fmt.Errorf("complete subject snapshot has no comparison")
	}
	return nil
}

// Retained results observe an actual original invocation, independently of event mappings.
type replayObservation struct {
	Snapshot     string                       `json:"snapshot"`
	Origin       OutcomeRef                   `json:"origin"`
	Replay       OutcomeRef                   `json:"replay"`
	Instance     string                       `json:"instance"`
	Identity     replayIdentity               `json:"identity"`
	Fields       []replayField                `json:"fields"`
	Declarations map[string]replayDeclaration `json:"declarations"`
}

// Identity is derived from the original effect; a neighboring bound instance is not authority.
type replayIdentity struct {
	Kind  string `json:"kind"`
	Field string `json:"field"`
	Event string `json:"event,omitempty"`
}

func (identity replayIdentity) validate() error {
	if !regexp.MustCompile(`^[A-Za-z][A-Za-z0-9_]*$`).MatchString(identity.Field) {
		return fmt.Errorf("invalid replay identity field")
	}
	switch identity.Kind {
	case "input":
		if identity.Event != "" {
			return fmt.Errorf("input identity cannot carry event authority")
		}
	case "event":
		return name(identity.Event, false)
	default:
		return fmt.Errorf("unknown replay identity source")
	}
	return nil
}

// Keep complete field authority, including optional schema metadata, for retry comparison.
type replayField struct {
	Name    string  `json:"name"`
	Type    string  `json:"type"`
	Wire    *string `json:"wire,omitempty"`
	Display *string `json:"display,omitempty"`
	Summary *string `json:"summary,omitempty"`
}
type replayDeclaration struct {
	Kind     string          `json:"kind"`
	Of       string          `json:"of,omitempty"`
	Fields   []replayField   `json:"fields,omitempty"`
	Variants json.RawMessage `json:"variants,omitempty"`
	Tag      string          `json:"tag,omitempty"`
}

func (r replayObservation) responseDeclarations() map[string]selectionDeclaration {
	result := map[string]selectionDeclaration{}
	for name, body := range r.Declarations {
		fields := make([]accessorField, 0, len(body.Fields))
		for _, field := range body.Fields {
			fields = append(fields, accessorField{Name: field.Name, Type: field.Type})
		}
		result[name] = selectionDeclaration{Kind: body.Kind, Of: body.Of, Fields: fields, Variants: body.Variants, Tag: body.Tag}
	}
	return result
}

func (r *replayObservation) UnmarshalJSON(raw []byte) error {
	type plain replayObservation
	var value plain
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return err
	}
	*r = replayObservation(value)
	for name, body := range r.Declarations {
		if len(body.Variants) != 0 {
			value, err := strictJSON(string(body.Variants))
			if err != nil {
				return err
			}
			body.Variants, err = json.Marshal(value)
			if err != nil {
				return err
			}
			r.Declarations[name] = body
		}
	}
	return r.validate()
}

func (r replayObservation) validate() error {
	if err := r.Identity.validate(); err != nil {
		return err
	}
	if r.Origin.Command != r.Replay.Command || r.Origin == r.Replay {
		return fmt.Errorf("invalid replay owner or schema bounds")
	}
	for _, label := range []string{r.Snapshot, r.Instance, r.Origin.Outcome, r.Replay.Outcome} {
		if err := name(label, true); err != nil {
			return err
		}
	}
	if err := name(r.Origin.Command, false); err != nil {
		return err
	}
	return r.validateSchema()
}

func (r replayObservation) validateSchema() error {
	if len(r.Fields) == 0 || len(r.Fields) > 256 || len(r.Declarations) > 4096 {
		return fmt.Errorf("invalid retained schema bounds")
	}
	// Reuse the response type grammar, but refuse both floating families recursively.
	check := responseObservation{Declarations: r.responseDeclarations()}
	used, fields := map[string]bool{}, map[string]bool{}
	exact := map[string]bool{}
	for _, field := range r.Fields {
		if field.Name == "" || fields[field.Name] {
			return fmt.Errorf("duplicate/empty replay field")
		}
		fields[field.Name] = true
		if err := check.checkType(field.Type, used, map[string]bool{}, 0); err != nil {
			return err
		}
		if err := r.exactType(field.Type, 0, exact); err != nil {
			return err
		}
	}
	if len(used) != len(r.Declarations) {
		return fmt.Errorf("unrelated replay declaration")
	}
	for label := range r.Declarations {
		if err := name(label, false); err != nil {
			return err
		}
	}
	raw, err := json.Marshal(r)
	if err != nil {
		return err
	}
	if len(raw) > 1048576 {
		return fmt.Errorf("replay schema byte limit")
	}
	return nil
}
func (r replayObservation) exactType(source string, depth int, visited map[string]bool) error {
	if depth > 128 {
		return fmt.Errorf("replay type depth limit")
	}
	if source == "Decimal" || source == "Binary64" {
		return fmt.Errorf("Decimal and Binary64 replay results are unsupported")
	}
	if inner, ok := accessorOptional(source); ok {
		return r.exactType(inner, depth+1, visited)
	}
	if inner, ok, err := accessorCollection(source); ok || err != nil {
		if err != nil {
			return err
		}
		return r.exactType(inner, depth+1, visited)
	}
	body, ok := r.Declarations[source]
	if !ok {
		return nil
	}
	if visited[source] {
		return nil
	}
	visited[source] = true
	switch body.Kind {
	case "newtype":
		return r.exactType(body.Of, depth+1, visited)
	case "struct":
		for _, field := range body.Fields {
			if err := r.exactType(field.Type, depth+1, visited); err != nil {
				return err
			}
		}
	case "union":
		var variants map[string]string
		if err := json.Unmarshal(body.Variants, &variants); err != nil {
			return err
		}
		for _, child := range variants {
			if err := r.exactType(child, depth+1, visited); err != nil {
				return err
			}
		}
	}
	return nil
}
func (r replayObservation) admitResult(response map[string]Node) error {
	if err := r.validate(); err != nil {
		return err
	}
	if response == nil {
		return fmt.Errorf("command returned no response")
	}
	fields := map[string]bool{}
	for _, field := range r.Fields {
		fields[field.Name] = true
	}
	for field := range response {
		if !fields[field] {
			return fmt.Errorf("undeclared replay response field")
		}
	}
	observer := selectionObservation{Declarations: r.responseDeclarations(), responseMode: true}
	size := 0
	for _, field := range r.Fields {
		value, present := response[field.Name]
		if err := observer.validateValue(field.Type, value, present, &size, 0); err != nil {
			return fmt.Errorf("replay field %s: %v", field.Name, err)
		}
	}
	if size > 1048576 {
		return fmt.Errorf("replay response byte limit")
	}
	return nil
}

func (r replayObservation) compare(original, retry map[string]Node) error {
	if err := r.admitResult(original); err != nil {
		return err
	}
	if err := r.admitResult(retry); err != nil {
		return err
	}
	if !responseEqual(original, retry) {
		return fmt.Errorf("retry differs from retained original response")
	}
	return nil
}

func admitReplay(value any) error {
	root, err := closed(value, "snapshot origin replay instance identity fields declarations", "")
	if err != nil {
		return err
	}
	identity, ok := root["identity"].(map[string]any)
	if !ok {
		return fmt.Errorf("replay identity must be object")
	}
	required := "kind field"
	if identity["kind"] == "event" {
		required += " event"
	}
	if _, err = closed(identity, required, ""); err != nil {
		return err
	}
	for _, key := range []string{"origin", "replay"} {
		if err = admitOutcome(root[key]); err != nil {
			return err
		}
	}
	if err := admitRetainedSchema(root); err != nil {
		return err
	}
	encoded, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var replay replayObservation
	return json.Unmarshal(encoded, &replay)
}

func admitRetainedSchema(root map[string]any) error {
	fields, err := array(root["fields"])
	if err != nil {
		return err
	}
	for _, field := range fields {
		if err = admitAccessorField(field); err != nil {
			return err
		}
	}
	declarations, ok := root["declarations"].(map[string]any)
	if !ok {
		return fmt.Errorf("replay declarations must be object")
	}
	for label, raw := range declarations {
		if err = name(label, false); err != nil {
			return err
		}
		body, ok := raw.(map[string]any)
		if !ok {
			return fmt.Errorf("invalid replay declaration")
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
			return fmt.Errorf("invalid replay declaration kind")
		}
		if _, err = closed(raw, required, ""); err != nil {
			return err
		}
		if rawFields, exists := body["fields"]; exists {
			children, e := array(rawFields)
			if e != nil {
				return e
			}
			for _, field := range children {
				if e = admitAccessorField(field); e != nil {
					return e
				}
			}
		}
	}
	return nil
}

// This is scenario-local observation, never a target-provided expected-response cache.
type retainedCommandResult struct {
	Authority replayObservation
	Response  map[string]Node
	Identity  Node
	Input     map[string]Node
	Actor     string
}

func replayInputsEqual(left, right map[string]Value) bool {
	if len(left) != len(right) {
		return false
	}
	for key, value := range left {
		other, present := right[key]
		if !present || value.Kind != other.Kind || value.Instance != other.Instance || value.Event != other.Event || value.Field != other.Field || !reflect.DeepEqual(value.Accessor, other.Accessor) || !reflect.DeepEqual(value.Selection, other.Selection) || !responseEqual(value.Value, other.Value) {
			return false
		}
	}
	return true
}

func replaySnapshot[T any](value T) (T, error) {
	var snapshot T
	raw, err := json.Marshal(value)
	if err != nil {
		return snapshot, err
	}
	if len(raw) > 1048576 {
		return snapshot, fmt.Errorf("replay observation byte limit")
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	err = decoder.Decode(&snapshot)
	return snapshot, err
}
func (r *run) retainedResult(index int, step Step) bool {
	capture := step.Capture
	if capture == nil {
		return r.fail(index, "missing replay observation")
	}
	original := step.Step == "capture_command_result"
	expected := capture.Replay
	if original {
		expected = capture.Origin
	}
	if r.lastCommand != expected.Command || r.last.Outcome != expected.Outcome || r.last.Error != "" {
		return r.fail(index, "retained result requires its successful command/outcome")
	}
	identity, bound := r.instances[capture.Instance]
	if !bound {
		return r.fail(index, "retained result subject is unbound")
	}
	if original {
		var expected Node
		present := false
		switch capture.Identity.Kind {
		case "input":
			expected, present = r.lastInput[capture.Identity.Field]
		case "event":
			count := 0
			for _, event := range r.last.DirectEvents {
				if event.Event == capture.Identity.Event {
					count++
					expected, present = event.Payload[capture.Identity.Field]
				}
			}
			if count != 1 {
				return r.fail(index, "original identity requires exactly one declared event occurrence")
			}
		}
		if !present || !responseEqual(identity, expected) {
			return r.fail(index, "captured subject differs from original identity authority")
		}
		if err := capture.admitResult(r.last.Response); err != nil {
			return r.fail(index, "%v", err)
		}
		if r.retained == nil {
			r.retained = map[string]retainedCommandResult{}
		}
		if _, exists := r.retained[capture.Snapshot]; exists {
			return r.fail(index, "retained result is write-once")
		}
		snapshot, err := replaySnapshot(retainedCommandResult{Authority: *capture, Response: r.last.Response, Identity: identity, Input: r.lastInput, Actor: r.lastActor})
		if err != nil {
			return r.fail(index, "%v", err)
		}
		r.retained[capture.Snapshot] = snapshot
		return true
	}
	saved, exists := r.retained[capture.Snapshot]
	if !exists || !reflect.DeepEqual(saved.Authority, *capture) || !responseEqual(saved.Identity, identity) || !responseEqual(saved.Input, r.lastInput) || saved.Actor != r.lastActor {
		return r.fail(index, "replay substituted original contract, identity, input, or actor")
	}
	if len(r.last.DirectEvents) != 0 {
		return r.fail(index, "replay emitted a direct event")
	}
	if err := capture.compare(saved.Response, r.last.Response); err != nil {
		return r.fail(index, "%v", err)
	}
	return true
}

// Reject substituted or overwritten authority before creating a target or calling its hooks.
func admitReplaySteps(raw []any) error {
	hasReplay := false
	hasComplete := false
	for _, item := range raw {
		step := item.(map[string]any)
		if step["step"] == "capture_command_result" || step["step"] == "expect_replay_result" {
			hasReplay = true
		}
		if step["step"] == "snapshot_complete_subject" || step["step"] == "expect_complete_subject_unchanged" {
			hasComplete = true
		}
	}
	if !hasReplay && !hasComplete {
		return nil
	}
	encoded, err := json.Marshal(raw)
	if err != nil {
		return err
	}
	var encodedSteps []json.RawMessage
	if err = json.Unmarshal(encoded, &encodedSteps); err != nil {
		return err
	}
	steps, err := decodeExactSteps(encodedSteps)
	if err != nil {
		return err
	}
	if err := admitCompleteSubjectSteps(steps); err != nil {
		return err
	}
	if !hasReplay {
		return nil
	}
	instances := map[string]bool{}
	type eventBinding struct {
		event, field string
		invocation   int
	}
	bindings := map[string]eventBinding{}
	captures := map[string]replayObservation{}
	command := ""
	var outcome *OutcomeRef
	var request Step
	requests := map[string]Step{}
	var active *replayObservation
	views, pending := map[string]bool{}, map[string]bool{}
	invocation, originalInvocation := 0, 0
	queried, queriedAt := "", 0
	for _, step := range steps {
		switch step.Step {
		case "query_view":
			queried, queriedAt = step.View, invocation
		case "configure_external_outcome":
			if active != nil || len(pending) != 0 {
				return fmt.Errorf("retained replay cannot inject an external outcome")
			}
		case "execute_command":
			if len(pending) != 0 {
				return fmt.Errorf("command precedes pending retained subject comparison")
			}
			if step.Input == nil {
				step.Input = map[string]Value{}
			}
			if active != nil {
				original := requests[active.Snapshot]
				if len(views) == 0 || invocation != originalInvocation || step.Command != original.Command || step.Actor != original.Actor || !replayInputsEqual(step.Input, original.Input) {
					return fmt.Errorf("replay substitutes original command, input, or actor")
				}
			}
			command = step.Command
			outcome = nil
			invocation++
			queried = ""
			request = step
		case "expect_outcome":
			outcome = step.Outcome
		case "capture_instance", "establish_entity":
			if len(pending) != 0 {
				return fmt.Errorf("instance binding precedes pending retained subject comparison")
			}
			if instances[step.Instance] {
				for _, capture := range captures {
					if capture.Instance == step.Instance {
						return fmt.Errorf("captured subject identity was overwritten")
					}
				}
			}
			instances[step.Instance] = true
			delete(bindings, step.Instance)
			if step.Step == "capture_instance" {
				bindings[step.Instance] = eventBinding{step.Event, step.Field, invocation}
			}
		case "capture_command_result":
			capture := step.Capture
			_, exists := captures[capture.Snapshot]
			if command != capture.Origin.Command || outcome == nil || *outcome != capture.Origin || !instances[capture.Instance] || exists {
				return fmt.Errorf("invalid or overwritten original result capture")
			}
			switch capture.Identity.Kind {
			case "input":
				value, present := request.Input[capture.Identity.Field]
				if !present || value.Kind != "instance" || value.Instance != capture.Instance {
					return fmt.Errorf("replay identity is not the original command input instance")
				}
			case "event":
				binding, present := bindings[capture.Instance]
				if !present || binding.invocation != invocation || binding.event != capture.Identity.Event || binding.field != capture.Identity.Field {
					return fmt.Errorf("replay identity is not the original invocation event field")
				}
			}
			captures[capture.Snapshot] = *capture
			if active != nil || len(pending) != 0 {
				return fmt.Errorf("aliased or incomplete retained result snapshot")
			}
			active = capture
			views = map[string]bool{}
			originalInvocation = invocation
			requests[capture.Snapshot] = request
		case "expect_replay_result":
			capture := step.Capture
			saved, exists := captures[capture.Snapshot]
			if !exists || !reflect.DeepEqual(saved, *capture) || command != capture.Replay.Command || outcome == nil || *outcome != capture.Replay {
				return fmt.Errorf("replay has no matching original capture")
			}
			if active == nil || !reflect.DeepEqual(*active, *capture) || len(views) == 0 || invocation != originalInvocation+1 {
				return fmt.Errorf("replay lacks original subject snapshots or a retry invocation")
			}
			pending = views
			active = nil
		case "snapshot_complete_subject":
			if len(pending) != 0 {
				return fmt.Errorf("original subject snapshot was overwritten after replay")
			}
			if active != nil {
				if invocation != originalInvocation || queried != step.View || queriedAt != originalInvocation || len(step.Subject) == 0 || views[step.View] {
					return fmt.Errorf("original subject snapshot is late, empty, or overwritten")
				}
				for _, value := range step.Subject {
					if value.Kind != "instance" || value.Instance != active.Instance {
						return fmt.Errorf("original subject snapshot names a different identity")
					}
				}
				views[step.View] = true
			}
		case "expect_complete_subject_unchanged":
			if len(pending) != 0 {
				if !pending[step.View] || queried != step.View || queriedAt != invocation {
					return fmt.Errorf("replay compares a different subject snapshot")
				}
				delete(pending, step.View)
			}
		case "snapshot_subject", "expect_subject_unchanged":
			if active != nil || len(pending) != 0 {
				return fmt.Errorf("retained result requires complete subject observation")
			}
		}
	}
	if active != nil || len(pending) != 0 {
		return fmt.Errorf("retained result has no complete subject comparison")
	}
	return nil
}
