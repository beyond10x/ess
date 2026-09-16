// LiveObservationTarget supplies complete facts on one subscribed monotonic lifetime.
// EndScenario owns release, including partially opened subscriptions.
type LiveObservationTarget interface {
	OpenObservations(events []string) (string, error)
	ObserveOccurrences(request LiveObservationRequest) (ObservationBatch, error)
}

type LiveObservationRequest struct {
	Lifetime  string
	After     uint64
	ThroughMS uint64
}

type LiveClaim struct {
	Event   string           `json:"event"`
	Matches map[string]Value `json:"matches"`
	Shape   map[string]Held  `json:"shape"`
}

type LiveCheck struct {
	Instance string `json:"instance"`
	Source string `json:"source"`
	Event string `json:"event"`
	Path string `json:"path"`
	RequirePresent bool `json:"require_present"`
	Min int64 `json:"min"`
	Max int64 `json:"max"`
	Plus int64 `json:"plus"`
	Kind       string           `json:"kind"`
	Events     []string         `json:"events"`
	Anchor     string           `json:"anchor"`
	After      *string          `json:"after"`
	Before     string           `json:"before"`
	Claim      *LiveClaim       `json:"claim"`
	DurationMS uint64           `json:"duration_ms"`
	Required   map[string]Value `json:"required"`
	Shape      map[string]Held  `json:"shape"`
}

type liveTraceState struct {
	ledger  *ObservationLedger
	anchors map[string]uint64
	events  map[string]bool
	quietAnchor string
	quietFloor *uint64
}

func admitLiveTrace(value any) error {
	f, ok := value.(map[string]any)
	if !ok {
		return fmt.Errorf("live trace must be an object")
	}
	kind, err := text(f["kind"])
	if err != nil {
		return err
	}
	required := "kind"
	switch kind {
	case "capture":
		required += " anchor instance event path shape require_present"
	case "offset":
		required += " instance source plus"
	case "integer_bounds":
		required += " anchor event path shape min max"
	case "quiet":
		required += " anchor duration_ms claim"
	case "open":
		required += " events"
	case "await":
		required += " anchor after claim"
	case "absent":
		required += " anchor duration_ms claim"
	case "stable":
		required += " anchor duration_ms claim required shape"
	case "ordered":
		required += " before after"
	default:
		return fmt.Errorf("unknown live trace kind")
	}
	if _, err := closed(value, required, ""); err != nil {
		return err
	}
	if raw, exists := f["shape"]; exists {
		if err := admitShape(raw); err != nil { return err }
	}
	if raw, exists := f["event"]; exists {
		if err := name(raw, false); err != nil { return err }
	}
	if raw, exists := f["claim"]; exists {
		claim, err := closed(raw, "event matches shape", "")
		if err != nil {
			return err
		}
		if err := name(claim["event"], false); err != nil {
			return err
		}
		if err := admitShape(claim["shape"]); err != nil {
			return err
		}
		if _, ok := claim["matches"].(map[string]any); !ok {
			return fmt.Errorf("live matches must be an object")
		}
		if err := admitValues(claim["matches"], 10, false); err != nil {
			return err
		}
	}
	if raw, exists := f["required"]; exists {
		required, ok := raw.(map[string]any)
		if !ok {
			return fmt.Errorf("live required must be an object")
		}
		if err := admitValues(required, 10, false); err != nil {
			return err
		}
		if err := admitShape(f["shape"]); err != nil {
			return err
		}
	}
	for _, field := range []string{"anchor", "before", "after", "instance", "source"} {
		if v, exists := f[field]; exists && !(kind == "await" && field == "after" && v == nil) {
			if err := name(v, true); err != nil {
				return err
			}
		}
	}
	encoded, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var check LiveCheck
	decoder := json.NewDecoder(strings.NewReader(string(encoded)))
	decoder.UseNumber()
	if err := decoder.Decode(&check); err != nil {
		return err
	}
	if kind == "capture" || kind == "integer_bounds" {
		leaf, exists := check.Shape[check.Path]
		if !exists || len(check.Shape) != 1 || check.Path == "" {
			return fmt.Errorf("event capture needs exactly one declared payload path")
		}
		for _, part := range strings.Split(check.Path, ".") {
			if part == "" { return fmt.Errorf("empty observed payload path segment") }
		}
		if kind == "capture" {
			if _, ok := f["require_present"].(bool); !ok || (leaf.Optional && !check.RequirePresent) {
				return fmt.Errorf("optional event capture requires explicit presence")
			}
		} else if !liveIntegerLeaf(leaf) || check.Min > check.Max {
			return fmt.Errorf("integer bounds require a declared Integer and ordered range")
		}
	}
	if kind == "open" {
		if len(check.Events) == 0 {
			return fmt.Errorf("empty observation subscription")
		}
		seen := map[string]bool{}
		for _, event := range check.Events {
			if !qualifiedName.MatchString(event) || seen[event] {
				return fmt.Errorf("invalid or duplicate subscription event")
			}
			seen[event] = true
		}
	}
	if (kind == "absent" || kind == "stable" || kind == "quiet") && check.DurationMS == 0 {
		return fmt.Errorf("live window must be positive")
	}
	return nil
}

func admitLiveSequence(steps []any, major int) error {
	events, captures := map[string]bool{}, map[string]bool{}
	anchors := map[string]string{}
	integers := map[string]bool{}
	for index, raw := range steps {
		data, _ := json.Marshal(raw)
		// Parent admission must not narrow unrelated legacy u64 metadata to
		// execution's machine-sized int fields. Decode only this vocabulary.
		var step struct {
			Step     string     `json:"step"`
			Instance string     `json:"instance"`
			Trace    *LiveCheck `json:"trace"`
			Field string `json:"field"`
			Shape map[string]Held `json:"shape"`
		}
		decoder := json.NewDecoder(strings.NewReader(string(data)))
		decoder.UseNumber()
		if err := decoder.Decode(&step); err != nil {
			return err
		}
		switch step.Step {
		case "capture_response", "capture_instance", "establish_entity":
			captures[step.Instance] = true
			if step.Step == "capture_response" && liveIntegerLeaf(step.Shape[step.Field]) {
				integers[step.Instance] = true
			}
		case "check_live":
			if major < 10 {
				return fmt.Errorf("live trace requires suite/10 or /11")
			}
			c := step.Trace
			if major < 12 && (c.Kind == "capture" || c.Kind == "offset" ||
				c.Kind == "integer_bounds" || c.Kind == "quiet") {
				return fmt.Errorf("captured observations require suite/12 or /13")
			}
			if c.Kind == "open" {
				if index != 0 {
					return fmt.Errorf("observation subscription must be first")
				}
				for _, event := range c.Events {
					events[event] = true
				}
				continue
			}
			if len(events) == 0 {
				return fmt.Errorf("live assertion before subscription")
			}
			if c.Event != "" && !events[c.Event] {
				return fmt.Errorf("observed event was not subscribed")
			}
			if c.Claim != nil {
				if !events[c.Claim.Event] {
					return fmt.Errorf("assertion event was not subscribed")
				}
				if err := admitLiveClaimValues(c.Claim.Matches, c.Claim.Shape, captures); err != nil {
					return err
				}
			}
			switch c.Kind {
			case "capture":
				if anchors[c.Anchor] != c.Event || captures[c.Instance] {
					return fmt.Errorf("capture needs its actual event and a unique value name")
				}
				captures[c.Instance] = true
				integers[c.Instance] = liveIntegerLeaf(c.Shape[c.Path])
			case "offset":
				if !integers[c.Source] || captures[c.Instance] {
					return fmt.Errorf("offset needs a preceding typed integer and unique value name")
				}
				captures[c.Instance], integers[c.Instance] = true, true
			case "integer_bounds":
				if anchors[c.Anchor] != c.Event {
					return fmt.Errorf("integer bound event differs from its occurrence")
				}
			case "quiet":
				if anchors[c.Anchor] != "" { return fmt.Errorf("duplicate occurrence anchor") }
				anchors[c.Anchor] = c.Claim.Event
			case "await":
				if c.After != nil && anchors[*c.After] == "" {
					return fmt.Errorf("unknown occurrence anchor")
				}
				if anchors[c.Anchor] != "" {
					return fmt.Errorf("duplicate occurrence anchor")
				}
				anchors[c.Anchor] = c.Claim.Event
			case "absent", "stable":
				if anchors[c.Anchor] == "" {
					return fmt.Errorf("unknown occurrence anchor")
				}
				if c.Kind == "stable" {
					if err := admitLiveClaimValues(c.Required, c.Shape, captures); err != nil {
						return err
					}
					for a := range c.Required {
						for b := range c.Claim.Matches {
							if a == b || strings.HasPrefix(a, b+".") || strings.HasPrefix(b, a+".") {
								return fmt.Errorf("stability claim overlaps scope filter")
							}
						}
					}
				}
			case "ordered":
				if anchors[c.Before] == "" || c.After == nil || anchors[*c.After] == "" {
					return fmt.Errorf("unknown occurrence anchor")
				}
			}
		}
	}
	return nil
}

func liveIntegerLeaf(leaf Held) bool {
	return leaf.Holds == "primitive" && leaf.Kind == "integer"
}

func liveInteger(value Node) (int64, bool) {
	number, ok := responseNumber(value)
	if !ok || !number.IsInt() || !number.Num().IsInt64() { return 0, false }
	return number.Num().Int64(), true
}

func admitLiveClaimValues(values map[string]Value, shape map[string]Held, captures map[string]bool) error {
	if len(values) == 0 {
		return fmt.Errorf("empty live payload claim")
	}
	for path, value := range values {
		leaf, present := shape[path]
		if !present {
			return fmt.Errorf("live path has no declared type")
		}
		if value.Kind == "literal" && liveLeafAdmits(leaf, value.Value, true) {
			continue
		}
		if value.Kind == "instance" && captures[value.Instance] {
			continue
		}
		return fmt.Errorf("live claim requires typed literal or preceding capture")
	}
	return nil
}

func (r *run) checkLive(index int, c *LiveCheck) bool {
	target, ok := r.target.(LiveObservationTarget)
	if !ok {
		r.skip("target cannot provide complete occurrence observations")
		return false
	}
	fail := func(err error) bool {
		if errors.Is(err, ErrUnsupported) {
			r.skip("live observation unsupported: %v", err)
			return false
		}
		return r.fail(index, "live observation: %v", err)
	}
	if c.Kind == "open" {
		lifetime, err := target.OpenObservations(append([]string(nil), c.Events...))
		if err != nil {
			return fail(err)
		}
		ledger, err := NewObservationLedger(lifetime)
		if err != nil {
			return fail(err)
		}
		r.trace = &liveTraceState{ledger: ledger, anchors: map[string]uint64{}, events: map[string]bool{}}
		for _, event := range c.Events {
			r.trace.events[event] = true
		}
		return true
	}
	state := r.trace
	var matcher OccurrenceMatcher
	if c.Claim != nil {
		values, ok := r.resolveAll(index, c.Claim.Matches)
		if !ok {
			return false
		}
		matcher = OccurrenceMatcher{Event: c.Claim.Event, Payload: values}
	}
	required, ok := r.resolveAll(index, c.Required)
	if !ok {
		return false
	}
	_, bounded := context.Background().Deadline()
	if r.ctx != nil {
		_, bounded = r.ctx.Deadline()
	}
	for attempt := 0; ; attempt++ {
		if r.ctx != nil && r.ctx.Err() != nil {
			return fail(r.ctx.Err())
		}
		var err error
		switch c.Kind {
		case "capture", "integer_bounds":
			var item *Occurrence
			item, err = state.ledger.anchor(state.anchors[c.Anchor])
			if err != nil { break }
			value, present := lookup(item.Payload, c.Path)
			valid := item.Event == c.Event && present && value != nil
			if c.Kind == "integer_bounds" {
				count, ok := liveInteger(value)
				valid = valid && ok && c.Min <= count && count <= c.Max
			} else {
				valid = valid && holdsLive(item.Payload, c.Shape) == ""
			}
			if !valid {
				err = &TemporalError{Kind: "counterexample", Counterexample: copyOccurrence(*item)}
				break
			}
			if c.Kind == "capture" {
				if _, exists := r.instances[c.Instance]; exists {
					err = fmt.Errorf("duplicate live capture")
					break
				}
				var copied map[string]Node
				copied, err = temporalFields(map[string]Node{"value": value})
				if err == nil { r.instances[c.Instance] = copied["value"] }
			}
		case "offset":
			base, ok := liveInteger(r.instances[c.Source])
			_, exists := r.instances[c.Instance]
			if !ok || exists {
				err = fmt.Errorf("offset requires a present signed integer and fresh binding")
			} else if (c.Plus > 0 && base > math.MaxInt64-c.Plus) ||
				(c.Plus < 0 && base < math.MinInt64-c.Plus) {
				err = fmt.Errorf("signed integer offset overflow")
			} else {
				r.instances[c.Instance] = json.Number(strconv.FormatInt(base+c.Plus, 10))
			}
		case "quiet":
			if state.quietAnchor != c.Anchor {
				state.quietAnchor, state.quietFloor = c.Anchor, nil
			}
			if state.quietFloor == nil {
				err = &TemporalError{Kind: "unfinished"}
				break
			}
			var found *Occurrence
			found, err = state.ledger.Quiet(matcher, *state.quietFloor, c.DurationMS)
			if err == nil {
				if violation := holdsLive(found.Payload, c.Claim.Shape); violation != "" {
					return r.fail(index, "%s", violation)
				}
				state.anchors[c.Anchor] = found.Sequence
			}
		case "await":
			after := uint64(0)
			if c.After != nil {
				after = state.anchors[*c.After]
			}
			var found *Occurrence
			found, err = state.ledger.Find(matcher, after)
			if err == nil && found == nil {
				err = &TemporalError{Kind: "unfinished"}
			}
			if err == nil {
				if violation := holdsLive(found.Payload, c.Claim.Shape); violation != "" {
					return r.fail(index, "%s", violation)
				}
				state.anchors[c.Anchor] = found.Sequence
			}
		case "absent":
			err = state.ledger.Absent(matcher, state.anchors[c.Anchor], c.DurationMS)
		case "stable":
			err = state.ledger.Stable(matcher, required, state.anchors[c.Anchor], c.DurationMS)
		case "ordered":
			err = state.ledger.Ordered(state.anchors[c.Before], state.anchors[*c.After])
		}
		if err == nil {
			return true
		}
		var detail *TemporalError
		if !errors.As(err, &detail) || detail.Kind != "unfinished" {
			return fail(err)
		}
		if !bounded && attempt >= 8 {
			return fail(err)
		}
		batch, err := target.ObserveOccurrences(LiveObservationRequest{Lifetime: state.ledger.lifetime, After: state.ledger.Cursor(), ThroughMS: detail.RequiredMS})
		if err != nil {
			return fail(err)
		}
		for _, occurrence := range batch.Occurrences {
			if !state.events[occurrence.Event] {
				return fail(state.ledger.Invalidate("unsubscribed event in observation batch"))
			}
		}
		if err := state.ledger.Accept(batch); err != nil {
			return fail(err)
		}
		if c.Kind == "quiet" && state.quietFloor == nil {
			floor := state.ledger.completeBeforeMS
			state.quietFloor = &floor
		}
	}
}
