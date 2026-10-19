// Suite/52 and /53: a command the system issues itself is awaited, never invoked
// (beyond10x/ess#542, docs/design/system-issued-commands.md). The same checks, in the same order,
// as ess_conformance's runner/await_outcome.rs:
//
//	answer                                    await_outcome                    expect_not_issued
//	reached, reading within the bound         passes; the current result      fails
//	reached, reading above it, or none        fails; still the current result fails
//	nothing, reading at least the bound       fails                            passes
//	nothing, reading short of it, or none     fails                            fails
//	nothing, no declared bound                asked again to the deadline,     asked again to the deadline,
//	                                          then fails                       then passes

// awaitSeconds is a declared `await_within:` bound, `PT<seconds>S`: positive whole seconds with no
// leading zero, as ess_domain's Period reads it.
func awaitSeconds(value string) (int, error) {
	if !strings.HasPrefix(value, "PT") || !strings.HasSuffix(value, "S") || len(value) < 4 {
		return 0, fmt.Errorf("an await bound is written PT<seconds>S")
	}
	digits := value[2 : len(value)-1]
	if len(digits) > 1 && digits[0] == '0' {
		return 0, fmt.Errorf("an await bound has no leading zero")
	}
	for _, c := range digits {
		if c < '0' || c > '9' {
			return 0, fmt.Errorf("an await bound uses decimal digits only")
		}
	}
	n, err := strconv.ParseUint(digits, 10, 32)
	if err != nil || n == 0 {
		return 0, fmt.Errorf("an await bound is a positive number of seconds that fits u32")
	}
	return int(n), nil
}

// admitAwaitInstance admits the record an await step addresses: a captured instance or a literal
// identity.
func admitAwaitInstance(value any, major int) error {
	object, ok := value.(map[string]any)
	if !ok || (object["kind"] != "instance" && object["kind"] != "literal") {
		return fmt.Errorf("an awaited instance is a captured instance or a literal identity")
	}
	return admitValues(map[string]any{"instance": value}, major, false)
}

// admitUnpublished admits the events an `expect_not_issued` observes after its window.
func admitUnpublished(value any) error {
	events, err := array(value)
	if err != nil {
		return err
	}
	for _, event := range events {
		if err := name(event, false); err != nil {
			return err
		}
	}
	return nil
}

// askAwait asks the target for the outcome its system reached, or answers ErrUnsupported for a
// target that does not implement AwaitTarget.
func (r *run) askAwait(request AwaitRequest) (AwaitAnswer, error) {
	awaiter, ok := r.target.(AwaitTarget)
	if !ok {
		return AwaitAnswer{}, fmt.Errorf("%w: this target cannot observe, or hold, a step its system takes itself", ErrUnsupported)
	}
	return awaiter.AwaitOutcome(request)
}

// awaitBound is the step's declared bound in seconds, zero where it declares none.
func (r *run) awaitBound(index int, step Step) (int, bool) {
	if step.Within == "" {
		return 0, true
	}
	seconds, err := awaitSeconds(step.Within)
	if err != nil {
		r.recordStatus(statusError)
		r.t.Errorf("step %d: ESS-CF-SUITE: %v", index, err)
		return 0, false
	}
	return seconds, true
}

// awaitInstance resolves the record an await step addresses.
func (r *run) awaitInstance(index int, step Step) (Node, bool) {
	if step.Awaited == nil {
		r.recordStatus(statusError)
		r.t.Errorf("step %d: ESS-CF-SUITE: the await of `%s` names no instance", index, step.Command)
		return nil, false
	}
	instance, err := r.resolve(*step.Awaited)
	if err != nil {
		r.resolutionFailure(index, "instance", err)
		return nil, false
	}
	return instance, true
}

func (r *run) awaitOutcome(index int, step Step) bool {
	instance, ok := r.awaitInstance(index, step)
	if !ok {
		return false
	}
	stated, ok := r.resolveAll(index, step.Input)
	if !ok {
		return false
	}
	within, ok := r.awaitBound(index, step)
	if !ok {
		return false
	}
	compared := make([]string, 0, len(stated))
	for field := range stated {
		compared = append(compared, field)
	}
	sort.Strings(compared)
	attempts := r.harness.Deadline().Attempts
	for attempt := 0; ; attempt++ {
		answer, err := r.askAwait(AwaitRequest{
			Command: step.Command, Instance: instance, Within: within,
			Deadline: Deadline{Attempts: attempts - attempt}, Compared: compared,
			Correlation: r.correlation,
		})
		if err != nil {
			return r.targetFailure(index, err, "awaiting `"+step.Command+"`")
		}
		if answer.Reached {
			return r.awaitReached(index, step, instance, stated, within, answer)
		}
		// Not believed before the deadline: a target may answer before it has held anything.
		if within == 0 && attempt+1 < attempts {
			continue
		}
		observed := fmt.Sprintf("nothing reached by the runner's deadline, after %d await(s)", attempt+1)
		if within > 0 {
			switch {
			case answer.ElapsedMillis == nil:
				observed = fmt.Sprintf("nothing reached, with no reading of the target's clock against the PT%dS bound", within)
			case *answer.ElapsedMillis < int64(within)*1000:
				observed = fmt.Sprintf("nothing reached, answered after %d ms, short of the PT%dS bound", *answer.ElapsedMillis, within)
			default:
				observed = fmt.Sprintf("nothing reached within PT%dS; the target's clock read %d ms", within, *answer.ElapsedMillis)
			}
		}
		return r.fail(index, "ESS-CF-AWAIT: `%s` awaited on %s: %s", step.Command, render(instance), observed)
	}
}

// awaitReached holds a reached answer to the step and makes it the scenario's current command
// result. The steps after it run whatever it found, as they do after an invoked command.
func (r *run) awaitReached(index int, step Step, instance Node, stated map[string]Node, within int, answer AwaitAnswer) bool {
	wrong := []string{}
	if within > 0 {
		switch {
		case answer.ElapsedMillis == nil:
			wrong = append(wrong, fmt.Sprintf("no reading of the target's clock against the PT%dS bound", within))
		case *answer.ElapsedMillis > int64(within)*1000:
			wrong = append(wrong, fmt.Sprintf("reached after %d ms, later than PT%dS", *answer.ElapsedMillis, within))
		}
	}
	identity, _ := step.Identity.(string)
	if carried, present := answer.Input[identity]; !present {
		wrong = append(wrong, fmt.Sprintf("the answer carries no `%s`, the input naming the instance", identity))
	} else if !equal(carried, instance) {
		wrong = append(wrong, fmt.Sprintf("the answer is about %s = %s, not the awaited instance", identity, render(carried)))
	}
	fields := make([]string, 0, len(stated))
	for field := range stated {
		fields = append(fields, field)
	}
	sort.Strings(fields)
	for _, field := range fields {
		carried, present := answer.Input[field]
		switch {
		case !present:
			wrong = append(wrong, fmt.Sprintf("%s was not carried, where the scenario states %s", field, render(stated[field])))
		case !equal(carried, stated[field]):
			wrong = append(wrong, fmt.Sprintf("%s = %s, where the scenario states %s", field, render(carried), render(stated[field])))
		}
	}
	input := answer.Input
	if input == nil {
		input = map[string]Node{}
	}
	if !r.took(index, step.Command, answer.Result) {
		return false
	}
	r.lastInput, r.lastActor = input, ""
	r.awaited = input
	if len(wrong) > 0 {
		r.fail(index, "ESS-CF-AWAIT: `%s` awaited on %s: %s", step.Command, render(instance), strings.Join(wrong, "; "))
	}
	if reason := holds(input, step.Shape); reason != "" {
		r.fail(index, "ESS-CF-PAYLOAD: the input of `%s`, as the system issued it: %s", step.Command, reason)
	}
	return true
}

func (r *run) expectNotIssued(index int, step Step) bool {
	instance, ok := r.awaitInstance(index, step)
	if !ok {
		return false
	}
	if !r.countBefore(index, step.Unpublished) {
		return false
	}
	within, ok := r.awaitBound(index, step)
	if !ok {
		return false
	}
	attempts := r.harness.Deadline().Attempts
	failure := ""
	for attempt := 0; ; attempt++ {
		answer, err := r.askAwait(AwaitRequest{
			Command: step.Command, Instance: instance, Within: within,
			Deadline: Deadline{Attempts: attempts - attempt}, Compared: []string{},
			Correlation: r.correlation,
		})
		if err != nil {
			return r.targetFailure(index, err, "awaiting `"+step.Command+"`")
		}
		if answer.Reached {
			for _, event := range answer.Result.DirectEvents {
				r.remember(event)
			}
			failure = "the system issued it: outcome = " + orNone(answer.Result.Outcome)
			break
		}
		if within == 0 {
			if attempt+1 < attempts {
				continue
			}
			break
		}
		switch {
		case answer.ElapsedMillis == nil:
			failure = fmt.Sprintf("nothing reached, with no reading of the target's clock against the PT%dS bound", within)
		case *answer.ElapsedMillis < int64(within)*1000:
			failure = fmt.Sprintf("nothing reached, answered after %d ms, before the PT%dS window closed", *answer.ElapsedMillis, within)
		}
		break
	}
	if failure != "" {
		r.fail(index, "ESS-CF-AWAIT: `%s` on %s, which the specification says is not issued: %s", step.Command, render(instance), failure)
	}
	// What the step must not have set in motion anywhere: the log may hold no more occurrences of
	// each event after the window than it held before it.
	for _, event := range step.Unpublished {
		after, ok := r.logCount(index, event, "after")
		if !ok {
			return false
		}
		if before := r.logBefore[event]; after > before {
			r.fail(index, "ESS-CF-EVENT: %d occurrence(s) of `%s` in the target's log after `%s` was not to be issued, %d before it", after, event, step.Command, before)
		}
	}
	return true
}
