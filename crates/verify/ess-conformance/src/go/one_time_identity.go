// Generated disclosure identity preparation; Run retains the suite33 execution cap.
type oneTimeCell struct {
	Origin  OutcomeRef
	Field   string
	Aspect  string
	Command string
	Outcome string
	View    string
	Actor   *string
}

func oneTimeCellIdentity(id string) (*oneTimeCell, error) {
	fail := func() (*oneTimeCell, error) { return nil, fmt.Errorf("invalid disclosure identity") }
	parts := strings.Split(id, "/")
	if len(parts) < 7 || parts[1] != "disclosure" || name(parts[0], false) != nil || name(parts[2], true) != nil || !regexp.MustCompile(`^_*[A-Za-z][A-Za-z0-9_]*$`).MatchString(parts[3]) {
		return fail()
	}
	cell := &oneTimeCell{Origin: OutcomeRef{Command: parts[0], Outcome: parts[2]}, Field: parts[3]}
	rest := parts[4:]
	end := len(rest)
	if end >= 2 && rest[end-2] == "as" && rest[end-1] == "anonymous" {
		rest = rest[:end-2]
	} else if end >= 3 && rest[end-3] == "as" && rest[end-2] == "actor" && name(rest[end-1], false) == nil {
		actor := rest[end-1]
		cell.Actor = &actor
		rest = rest[:end-3]
	} else {
		return fail()
	}
	if len(rest) == 0 {
		return fail()
	}
	cell.Aspect = rest[0]
	switch cell.Aspect {
	case "origin", "retry", "rotation":
		if len(rest) != 1 {
			return fail()
		}
	case "read":
		if len(rest) != 2 || name(rest[1], false) != nil {
			return fail()
		}
		cell.View = rest[1]
	case "command":
		if len(rest) != 3 || name(rest[1], false) != nil || name(rest[2], true) != nil {
			return fail()
		}
		cell.Command, cell.Outcome = rest[1], rest[2]
	case "denied":
		if len(rest) != 2 || name(rest[1], false) != nil {
			return fail()
		}
		cell.Command = rest[1]
	default:
		return fail()
	}
	return cell, nil
}

func oneTimeInvocation(step map[string]any) bool {
	return step["step"] == "execute_command" || step["step"] == "execute_command_without_input"
}
func oneTimeActorMatches(step map[string]any, actor *string) bool {
	if actor == nil {
		return step["actor"] == nil
	}
	return step["actor"] == *actor
}

// One actual invocation and its selected outcome; the subsequent search begins after that
// assertion, so a repeated assertion cannot supply another originating invocation.
func oneTimeSelected(steps []any, expected OutcomeRef, filterActor bool, actor *string) (int, int, bool) {
	last := -1
	for index, raw := range steps {
		step := raw.(map[string]any)
		if oneTimeInvocation(step) {
			last = index
		}
		if last < 0 || step["step"] != "expect_outcome" {
			continue
		}
		invoked := steps[last].(map[string]any)
		outcome, ok := step["outcome"].(map[string]any)
		if ok && invoked["command"] == expected.Command && outcome["command"] == expected.Command && outcome["outcome"] == expected.Outcome && (!filterActor || oneTimeActorMatches(invoked, actor)) {
			return last, index, true
		}
	}
	return 0, 0, false
}

func oneTimeSameInvocation(left, right map[string]any) bool {
	meaning := func(step map[string]any) any {
		return scenarioMeaning(map[string]any{"source": []any{}, "steps": []any{step}})
	}
	return reflect.DeepEqual(meaning(left), meaning(right))
}

func admitOneTimeCell(id string, scenario map[string]any, policy *oneTimeTrace, major int) error {
	parts := strings.Split(id, "/")
	if len(parts) < 2 || parts[1] != "disclosure" {
		return scenarioIdentity(id)
	}
	cell, err := oneTimeCellIdentity(id)
	if err != nil {
		return err
	}
	if major < 34 {
		return fmt.Errorf("disclosure identity requires suite34/35")
	}
	if policy == nil {
		return fmt.Errorf("disclosure identity has no trace policy")
	}
	required, marked := false, false
	for _, origin := range policy.RequiredOrigins {
		if origin == cell.Origin {
			required = true
		}
	}
	for _, origin := range policy.Origins {
		if origin.Outcome == cell.Origin {
			for _, field := range origin.Fields {
				if field == cell.Field {
					marked = true
				}
			}
		}
	}
	if !required || !marked {
		return fmt.Errorf("disclosure identity contradicts required marked origin")
	}
	steps, err := array(scenario["steps"])
	if err != nil {
		return err
	}
	invoked, selected, ok := oneTimeSelected(steps, cell.Origin, false, nil)
	if !ok {
		return fmt.Errorf("disclosure identity has no selected originating invocation")
	}
	origin := steps[invoked].(map[string]any)
	tail := steps[selected+1:]
	fulfilled := false
	switch cell.Aspect {
	case "origin":
		fulfilled = oneTimeActorMatches(origin, cell.Actor)
	case "retry":
		if oneTimeActorMatches(origin, cell.Actor) {
			for _, raw := range tail {
				step := raw.(map[string]any)
				if oneTimeInvocation(step) && oneTimeSameInvocation(origin, step) {
					fulfilled = true
					break
				}
			}
		}
	case "rotation":
		_, _, fulfilled = oneTimeSelected(tail, cell.Origin, true, cell.Actor)
	case "read":
		if oneTimeActorMatches(origin, cell.Actor) {
			for _, raw := range tail {
				step := raw.(map[string]any)
				if (step["step"] == "query_view" || step["step"] == "eventually_view") && step["view"] == cell.View {
					fulfilled = true
					break
				}
			}
		}
	case "command":
		_, _, fulfilled = oneTimeSelected(tail, OutcomeRef{Command: cell.Command, Outcome: cell.Outcome}, true, cell.Actor)
	case "denied":
		var last map[string]any
		if cell.Actor != nil {
			for _, raw := range tail {
				step := raw.(map[string]any)
				if oneTimeInvocation(step) {
					last = step
				}
				if step["step"] == "expect_not_granted" && step["actor"] == *cell.Actor && last != nil && last["command"] == cell.Command && oneTimeActorMatches(last, cell.Actor) {
					fulfilled = true
					break
				}
			}
		}
	}
	if !fulfilled {
		return fmt.Errorf("disclosure identity has no matching follow-up obligation")
	}
	return nil
}
