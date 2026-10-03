// Private scenario-local captures. Neither values nor target-supplied diagnostics cross this
// observer's reporting boundary. This does not record a replayable history.
type oneTimeLogger interface {
	Errorf(string, ...any)
	Fatalf(string, ...any)
	Skipf(string, ...any)
	Log(...any)
}

type protectedOneTimeLogger struct{ oneTimeLogger }

func protectedOneTimeMessage(format string) string {
	if strings.HasPrefix(format, "begin:") || strings.HasPrefix(format, "end:") || strings.HasPrefix(format, "fixture values:") {
		return "ESS-CF-TARGET: protected observation"
	}
	for _, code := range []string{"ESS-CF-DISCLOSURE", "ESS-CF-PAYLOAD", "ESS-CF-TARGET", "ESS-CF-SUITE", "ESS-CF-INVOCATION"} {
		if strings.Contains(format, code) {
			return code + ": protected observation"
		}
	}
	return "ESS-CF-DISCLOSURE: protected observation"
}
func (t protectedOneTimeLogger) Errorf(format string, _ ...any) {
	t.oneTimeLogger.Errorf("%s", protectedOneTimeMessage(format))
}
func (t protectedOneTimeLogger) Fatalf(format string, _ ...any) {
	t.oneTimeLogger.Fatalf("%s", protectedOneTimeMessage(format))
}
func (t protectedOneTimeLogger) Skipf(format string, _ ...any) {
	t.oneTimeLogger.Skipf("%s", protectedOneTimeMessage(format))
}
func (t protectedOneTimeLogger) Log(_ ...any) {
	t.oneTimeLogger.Log("ESS-CF-DISCLOSURE: protected observation")
}

func attachOneTimePolicies(suite *Suite) error {
	suite.oneTime = map[string]*oneTimeTrace{}
	for id, raw := range suite.document["scenarios"].(map[string]any) {
		policy, err := admitOneTimeTrace(raw.(map[string]any), suiteMajor(suite.Provenance.SuiteVersion))
		if err != nil {
			return err
		}
		if policy != nil {
			suite.oneTime[id] = policy
		}
	}
	return nil
}

type oneTimeCaptures struct {
	policy   *oneTimeTrace
	values   []string
	bytes    int
	observed map[OutcomeRef]bool
}

func newOneTimeCaptures(policy *oneTimeTrace) *oneTimeCaptures {
	if policy == nil {
		return nil
	}
	return &oneTimeCaptures{policy: policy, observed: map[OutcomeRef]bool{}}
}

func oneTimeText(text string, values []string) bool {
	for _, value := range values {
		if strings.Contains(text, value) {
			return false
		}
	}
	return true
}

// The canonical encoded bound is independent of the captured-text budget. Depth and member
// accounting stay bounded even when no secret has yet been captured.
func oneTimeScan(value Node, values []string, depth int, members *int) string {
	if depth > 128 || *members > 65536 {
		return "ESS-CF-TARGET"
	}
	switch value := value.(type) {
	case string:
		if !oneTimeText(value, values) {
			return "ESS-CF-DISCLOSURE"
		}
	case []any:
		*members += len(value)
		if *members > 65536 {
			return "ESS-CF-TARGET"
		}
		for _, child := range value {
			if code := oneTimeScan(child, values, depth+1, members); code != "" {
				return code
			}
		}
	case map[string]any:
		*members += len(value)
		if *members > 65536 {
			return "ESS-CF-TARGET"
		}
		for _, key := range meaningKeys(value) {
			if !oneTimeText(key, values) {
				return "ESS-CF-DISCLOSURE"
			}
			if code := oneTimeScan(value[key], values, depth+1, members); code != "" {
				return code
			}
		}
	}
	return ""
}

func oneTimeObservation(value Node, values []string) string {
	budget := oneTimeJSONBudget{}
	if !budget.node(value, 0) {
		return "ESS-CF-TARGET"
	}
	members := 0
	if code := oneTimeScan(value, values, 0, &members); code != "" {
		return code
	}
	return ""
}

// Count the native Node serializer's compact UTF-8 encoding without constructing an
// unbounded JSON buffer. In particular U+2028/U+2029 stay UTF-8 and representable domain
// integers serialize as binary64 (1.0), while exact larger integers retain their digits.
type oneTimeJSONBudget struct{ bytes, members int }

func (b *oneTimeJSONBudget) add(size int) bool {
	if size < 0 || size > 1048576-b.bytes {
		return false
	}
	b.bytes += size
	return true
}
func (b *oneTimeJSONBudget) text(text string) bool {
	if !utf8.ValidString(text) || len(text) > 1048576-b.bytes || !b.add(2) {
		return false
	}
	for _, char := range text {
		size := utf8.RuneLen(char)
		switch char {
		case '"', '\\', '\b', '\f', '\n', '\r', '\t':
			size = 2
		default:
			if char < 32 {
				size = 6
			}
		}
		if !b.add(size) {
			return false
		}
	}
	return true
}
func (b *oneTimeJSONBudget) node(value Node, depth int) bool {
	if depth > 128 {
		return false
	}
	switch value := value.(type) {
	case nil:
		return b.add(4)
	case bool:
		if value {
			return b.add(4)
		}
		return b.add(5)
	case string:
		return b.text(value)
	case []any:
		if len(value) > 65536-b.members || !b.add(2) {
			return false
		}
		b.members += len(value)
		for index, child := range value {
			if index > 0 && !b.add(1) {
				return false
			}
			if !b.node(child, depth+1) {
				return false
			}
		}
		return true
	case map[string]any:
		if len(value) > 65536-b.members || !b.add(2) {
			return false
		}
		b.members += len(value)
		index := 0
		for key, child := range value {
			if index > 0 && !b.add(1) {
				return false
			}
			index++
			if !b.text(key) || !b.add(1) || !b.node(child, depth+1) {
				return false
			}
		}
		return true
	default:
		if raw, ok := value.(json.Number); ok && len(raw) > 1048576-b.bytes {
			return false
		}
		value = oneTimeSourceNumbers(value)
		if _, ok := numberValue(value); !ok {
			return false
		}
		number, ok := oneTimeLiteralWire(value).(json.Number)
		return ok && b.add(len(number))
	}
}
func oneTimeMaps(maps []map[string]Node, values []string) string {
	array := make([]any, len(maps))
	for index, value := range maps {
		array[index] = value
	}
	return oneTimeObservation(array, values)
}

func (c *oneTimeCaptures) command(command string, result CommandResult) string {
	if result.Response != nil {
		if code := oneTimeObservation(result.Response, c.values); code != "" {
			return code
		}
	}
	for _, origin := range c.policy.Origins {
		if origin.Command != command || origin.Outcome.Outcome != result.Outcome {
			continue
		}
		if result.Error != "" {
			return "ESS-CF-PAYLOAD"
		}
		shape := directResponseObservation{Fields: origin.Response.Fields, Declarations: origin.Response.Declarations}
		if err := shape.compare(result.Response); err != nil {
			return "ESS-CF-PAYLOAD"
		}
		for _, field := range origin.Response.Fields {
			if value, ok := result.Response[field.Name]; ok {
				if code := oneTimeCheckConstraints(origin.Response, field.Type, value); code != "" {
					return code
				}
			}
		}
		added := []string{}
		addedBytes := 0
		for _, field := range origin.Fields {
			value, ok := result.Response[field].(string)
			if !ok || value == "" {
				return "ESS-CF-PAYLOAD"
			}
			for _, old := range added {
				if strings.Contains(old, value) || strings.Contains(value, old) {
					return "ESS-CF-DISCLOSURE"
				}
			}
			added = append(added, value)
			addedBytes += len(value)
		}
		if len(c.values)+len(added) > 256 || c.bytes+addedBytes > 1048576 {
			return "ESS-CF-TARGET"
		}
		for _, key := range meaningKeys(result.Response) {
			if !oneTimeText(key, added) {
				return "ESS-CF-DISCLOSURE"
			}
			marked := false
			for _, field := range origin.Fields {
				marked = marked || key == field
			}
			if !marked {
				count := 0
				if code := oneTimeScan(result.Response[key], added, 0, &count); code != "" {
					return code
				}
			}
		}
		c.observed[origin.Outcome] = true
		c.bytes += addedBytes
		c.values = append(c.values, added...)
		break
	}
	events := make([]map[string]Node, len(result.DirectEvents))
	for index, event := range result.DirectEvents {
		events[index] = event.Payload
	}
	if code := oneTimeMaps(events, c.values); code != "" {
		return code
	}
	if result.ErrorPayload != nil {
		return oneTimeObservation(result.ErrorPayload, c.values)
	}
	return ""
}

func oneTimeCheckConstraints(authority oneTimeResponse, ty string, value Node) string {
	if inner, ok := oneTimeGenericType(ty, "Optional"); ok {
		if value == nil {
			return ""
		}
		return oneTimeCheckConstraints(authority, inner, value)
	}
	if inner, ok := oneTimeGenericType(ty, "List"); ok {
		for _, child := range value.([]any) {
			if code := oneTimeCheckConstraints(authority, inner, child); code != "" {
				return code
			}
		}
		return ""
	}
	if body, ok := oneTimeGenericType(ty, "Map"); ok {
		_, inner, ok := strings.Cut(body, ",")
		if ok {
			for _, child := range value.(map[string]any) {
				if code := oneTimeCheckConstraints(authority, strings.TrimSpace(inner), child); code != "" {
					return code
				}
			}
		}
		return ""
	}
	declaration, named := authority.Declarations[ty]
	if !named {
		return ""
	}
	if rules, ok := authority.Constraints[ty]; ok {
		text, ok := value.(string)
		if !ok {
			return "ESS-CF-PAYLOAD"
		}
		if rules.Prefix != nil && !strings.HasPrefix(text, *rules.Prefix) {
			return "ESS-CF-PAYLOAD"
		}
		if rules.Alphabet != nil {
			for _, char := range text {
				if !strings.ContainsRune(*rules.Alphabet, char) {
					return "ESS-CF-PAYLOAD"
				}
			}
		}
		for _, predicate := range rules.parsed {
			switch predicate.evaluate(factSource{"value": text, "value.count": json.Number(strconv.Itoa(utf8.RuneCountInString(text)))}) {
			case truthFalse:
				return "ESS-CF-PAYLOAD"
			case truthUnknown:
				return "ESS-CF-TARGET"
			}
		}
	}
	switch declaration.Kind {
	case "newtype":
		return oneTimeCheckConstraints(authority, declaration.Of, value)
	case "struct":
		for _, field := range declaration.Fields {
			if child, ok := value.(map[string]any)[field.Name]; ok {
				if code := oneTimeCheckConstraints(authority, field.Type, child); code != "" {
					return code
				}
			}
		}
	case "union":
		var variants map[string]string
		if json.Unmarshal(declaration.Variants, &variants) != nil {
			return "ESS-CF-PAYLOAD"
		}
		row := value.(map[string]any)
		tag, _ := row[declaration.Tag].(string)
		content := "value"
		if declaration.Tag == "value" {
			content = "content"
		}
		if child, ok := row[content]; ok {
			return oneTimeCheckConstraints(authority, variants[tag], child)
		}
	}
	return ""
}
func oneTimeGenericType(ty, wrapper string) (string, bool) {
	if strings.HasPrefix(ty, wrapper+"<") && strings.HasSuffix(ty, ">") {
		return strings.TrimSpace(ty[len(wrapper)+1 : len(ty)-1]), true
	}
	return "", false
}

func (r *run) disclosureViolation(code string) bool {
	r.disclosureStopped = true
	if code == "ESS-CF-TARGET" {
		r.disclosureUnavailable = true
		r.recordStatus(statusUnsupported)
	} else {
		r.recordStatus(statusFailed)
	}
	r.t.Errorf(code + ": protected observation")
	return false
}
func (r *run) disclosureMaps(maps []map[string]Node) bool {
	if r.disclosure == nil {
		return true
	}
	if r.disclosureStopped {
		return false
	}
	if code := oneTimeMaps(maps, r.disclosure.values); code != "" {
		return r.disclosureViolation(code)
	}
	return true
}
func (r *run) disclosureRows(rows []Row) bool {
	maps := make([]map[string]Node, len(rows))
	for index, row := range rows {
		maps[index] = row
	}
	return r.disclosureMaps(maps)
}
func (r *run) disclosureObserved(events []ObservedEvent) bool {
	maps := make([]map[string]Node, len(events))
	for index, event := range events {
		maps[index] = event.Payload
	}
	return r.disclosureMaps(maps)
}
func (r *run) disclosureComplete() {
	if r.disclosure == nil || r.disclosureUnavailable {
		return
	}
	for _, origin := range r.disclosure.policy.RequiredOrigins {
		if !r.disclosure.observed[origin] {
			r.disclosureViolation("ESS-CF-DISCLOSURE")
			return
		}
	}
	r.t.Log("ESS-CF-DISCLOSURE: complete protected trace")
}
func (r *run) disclosureEvents(event string, attempts int) bool {
	observed, err := r.target.ObserveEvents(EventObservationRequest{Event: event, Correlation: r.correlation, Deadline: Deadline{Attempts: attempts}})
	if err != nil {
		r.disclosureUnavailable = true
		return r.targetFailure(0, err, "independent disclosure observation")
	}
	if !r.disclosureObserved(observed) {
		return false
	}
	for _, event := range observed {
		r.remember(event)
	}
	return !r.disclosureStopped
}
func (r *run) disclosureFinal() {
	if r.disclosure == nil || r.disclosureStopped {
		return
	}
	for _, event := range r.disclosure.policy.Events {
		if !r.disclosureEvents(event.Event, 0) {
			return
		}
	}
}
func (r *run) disclosureWindows(index int, scenario Scenario) bool {
	if r.disclosure == nil {
		return true
	}
	// All windows at one operation share the native runner's start instant. Each
	// polling read advances its deterministic clock by stepMillis, across windows.
	elapsedTicks := uint64(0)
	for _, window := range r.disclosure.policy.EventWindows {
		if window.AfterStep != uint64(index) {
			continue
		}
		if window.WithinMillis == 0 {
			if !r.disclosureEvents(window.Event, 0) {
				return false
			}
			continue
		}
		clock, ok := r.target.(Clock)
		if !ok {
			r.disclosureUnavailable = true
			return r.targetFailure(index, ErrUnsupported, "opening disclosure window")
		}
		instant := ""
		for serial := 0; serial < 65536; serial++ {
			name := fmt.Sprintf("one-time-window-%d-%d", index, serial)
			used := false
			for _, step := range scenario.Steps {
				used = used || step.Step == "mark_instant" && step.Instant == name
			}
			if !used {
				instant = name
				break
			}
		}
		if instant == "" {
			return r.disclosureViolation("ESS-CF-TARGET")
		}
		if err := clock.MarkInstant(InstantMark{Instant: instant, Correlation: r.correlation}); err != nil {
			r.disclosureUnavailable = true
			return r.targetFailure(index, err, "opening disclosure window")
		}
		// A first harmless observation is not completion. The target's existing elapsed
		// capability must prove the full rounded-up hold before the final scan.
		attempts := window.WithinMillis / stepMillis
		if window.WithinMillis%stepMillis != 0 {
			attempts++
		}
		if attempts > elapsedTicks {
			attempts -= elapsedTicks
		} else {
			attempts = 1
		}
		for attempt := uint64(0); attempt < attempts; attempt++ {
			remaining := attempts - attempt
			if remaining > 65536 {
				remaining = 65536
			}
			if !r.disclosureEvents(window.Event, int(remaining)) {
				return false
			}
			if attempt == 65535 {
				return r.disclosureViolation("ESS-CF-TARGET")
			}
			elapsedTicks++
		}
		seconds := window.WithinMillis / 1000
		if window.WithinMillis%1000 != 0 {
			seconds++
		}
		if seconds > 4294967295 {
			return r.disclosureViolation("ESS-CF-TARGET")
		}
		elapsed, err := clock.ObserveElapsed(ElapsedRequest{Instant: instant, Hold: int(seconds), Watching: window.Event, Correlation: r.correlation})
		if err != nil {
			r.disclosureUnavailable = true
			return r.targetFailure(index, err, "closing disclosure window")
		}
		if elapsed.ElapsedMillis < 0 || uint64(elapsed.ElapsedMillis) < window.WithinMillis {
			r.disclosureUnavailable = true
			return r.targetFailure(index, ErrUnsupported, "incomplete disclosure window")
		}
		if !r.disclosureEvents(window.Event, 0) {
			return false
		}
	}
	return true
}
