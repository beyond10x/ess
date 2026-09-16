// Native live observation semantics. These types do not admit a new suite format.

// Occurrence retains a real observation's identity on one scenario-local monotonic clock.
type Occurrence struct {
	Sequence uint64
	AtMS     uint64
	Event    string
	Payload  map[string]Node
}

// ObservationBatch extends a subscribed lifetime with every occurrence strictly before
// CompleteBeforeMS. Sequence ordinals are contiguous across its subscribed sources.
type ObservationBatch struct {
	Lifetime         string
	After            uint64
	CompleteBeforeMS uint64
	Occurrences      []Occurrence
}

// OccurrenceMatcher compares an event name and each named actual payload field.
type OccurrenceMatcher struct {
	Event   string
	Payload map[string]Node
}

func (m OccurrenceMatcher) matches(item Occurrence) bool {
	if m.Event != item.Event {
		return false
	}
	for field, value := range m.Payload {
		actual, exists := lookup(item.Payload, field)
		if !exists || !responseEqual(actual, value) {
			return false
		}
	}
	return true
}

func temporalFields(fields map[string]Node) (map[string]Node, error) {
	raw, err := json.Marshal(fields)
	if err != nil {
		return nil, &TemporalError{Kind: "invalid_claim", Reason: "payload is not a typed wire value"}
	}
	var copied map[string]Node
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&copied); err != nil {
		return nil, &TemporalError{Kind: "invalid_claim", Reason: "payload cannot be retained"}
	}
	return copied, nil
}

func (m OccurrenceMatcher) checked() (OccurrenceMatcher, error) {
	if !qualifiedName.MatchString(m.Event) {
		return m, &TemporalError{Kind: "invalid_claim", Reason: "invalid event name"}
	}
	fields, err := temporalFields(m.Payload)
	return OccurrenceMatcher{Event: m.Event, Payload: fields}, err
}

// TemporalError preserves incomplete observation separately from a real counterexample.
type TemporalError struct {
	Kind           string
	Reason         string
	Anchor         uint64
	RequiredMS     uint64
	ObservedMS     uint64
	Before         uint64
	After          uint64
	Counterexample *Occurrence
}

func (e *TemporalError) Error() string {
	return fmt.Sprintf("temporal %s: %s (anchor=%d, required=%d, observed=%d)", e.Kind, e.Reason, e.Anchor, e.RequiredMS, e.ObservedMS)
}

// ObservationLedger is native-owned history. A source failure permanently invalidates it.
// It is owned by one evaluator goroutine; adapters synchronize their own incoming sources.
type ObservationLedger struct {
	lifetime         string
	occurrences      []Occurrence
	completeBeforeMS uint64
	broken           string
}

// NewObservationLedger starts observation before any stimulus is issued.
func NewObservationLedger(lifetime string) (*ObservationLedger, error) {
	if strings.TrimSpace(lifetime) == "" {
		return nil, &TemporalError{Kind: "incomplete", Reason: "empty observation lifetime"}
	}
	return &ObservationLedger{lifetime: lifetime}, nil
}

// Invalidate records the first gap, including disconnect, overflow, decode or cancellation.
func (l *ObservationLedger) Invalidate(reason string) error {
	if l.broken == "" {
		if reason == "" {
			reason = "unspecified observation failure"
		}
		l.broken = reason
	}
	return l.healthy()
}

func (l *ObservationLedger) healthy() error {
	if l.broken != "" {
		return &TemporalError{Kind: "incomplete", Reason: l.broken}
	}
	return nil
}

// Cursor is the last admitted ordinal. It cannot advance on a partly valid batch.
func (l *ObservationLedger) Cursor() uint64 {
	if len(l.occurrences) == 0 {
		return 0
	}
	return l.occurrences[len(l.occurrences)-1].Sequence
}

// Accept snapshots and admits a complete batch atomically. Malformed batches poison the
// lifetime; a later good frame cannot conceal an earlier gap.
func (l *ObservationLedger) Accept(batch ObservationBatch) error {
	if err := l.healthy(); err != nil {
		return err
	}
	raw, err := json.Marshal(batch)
	if err != nil {
		return l.Invalidate("observation is not a typed wire value")
	}
	var copied ObservationBatch
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&copied); err != nil {
		return l.Invalidate("observation cannot be retained")
	}
	if copied.Lifetime != l.lifetime {
		return l.Invalidate("observation lifetime changed")
	}
	if copied.After != l.Cursor() {
		return l.Invalidate("observation cursor is not the preceding receipt")
	}
	if copied.CompleteBeforeMS < l.completeBeforeMS {
		return l.Invalidate("observation watermark regressed")
	}
	previous, at := l.Cursor(), l.completeBeforeMS
	for _, item := range copied.Occurrences {
		if !qualifiedName.MatchString(item.Event) {
			return l.Invalidate("invalid event name")
		}
		if previous == ^uint64(0) || item.Sequence != previous+1 {
			return l.Invalidate("missing or duplicate occurrence ordinal")
		}
		if item.AtMS < at || item.AtMS >= copied.CompleteBeforeMS {
			return l.Invalidate("occurrence falls outside the complete monotonic interval")
		}
		previous, at = item.Sequence, item.AtMS
	}
	l.occurrences = append(l.occurrences, copied.Occurrences...)
	l.completeBeforeMS = copied.CompleteBeforeMS
	return nil
}

func (l *ObservationLedger) anchor(sequence uint64) (*Occurrence, error) {
	if err := l.healthy(); err != nil {
		return nil, err
	}
	for index := range l.occurrences {
		if l.occurrences[index].Sequence == sequence {
			return &l.occurrences[index], nil
		}
	}
	return nil, &TemporalError{Kind: "unknown_anchor", Anchor: sequence}
}

func copyOccurrence(item Occurrence) *Occurrence {
	// The source is an already admitted wire snapshot. Copy again so diagnostics and public
	// lookup results cannot mutate the evaluator's retained history.
	raw, err := json.Marshal(item)
	if err != nil {
		panic("native occurrence snapshot lost wire shape")
	}
	var copied Occurrence
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&copied); err != nil {
		panic("native occurrence snapshot cannot be copied")
	}
	return &copied
}

// Find returns the first actual matching occurrence strictly after an earlier ordinal,
// or after subscription when after is zero. The caller receives an independent snapshot.
func (l *ObservationLedger) Find(matcher OccurrenceMatcher, after uint64) (*Occurrence, error) {
	if err := l.healthy(); err != nil {
		return nil, err
	}
	if after != 0 {
		if _, err := l.anchor(after); err != nil {
			return nil, err
		}
	}
	matcher, err := matcher.checked()
	if err != nil {
		return nil, err
	}
	for _, item := range l.occurrences {
		if item.Sequence > after && matcher.matches(item) {
			return copyOccurrence(item), nil
		}
	}
	return nil, nil
}

// Ordered requires distinct actual occurrences in strict order, including one clock tick.
func (l *ObservationLedger) Ordered(before, after uint64) error {
	if _, err := l.anchor(before); err != nil {
		return err
	}
	if _, err := l.anchor(after); err != nil {
		return err
	}
	if before >= after {
		return &TemporalError{Kind: "order", Before: before, After: after}
	}
	return nil
}

func (l *ObservationLedger) end(anchor, durationMS uint64) (uint64, error) {
	start, err := l.anchor(anchor)
	if err != nil {
		return 0, err
	}
	if durationMS == 0 || durationMS > ^uint64(0)-start.AtMS {
		return 0, &TemporalError{Kind: "invalid_window"}
	}
	return start.AtMS + durationMS, nil
}

func (l *ObservationLedger) complete(requiredMS uint64) error {
	if err := l.healthy(); err != nil {
		return err
	}
	if l.completeBeforeMS < requiredMS {
		return &TemporalError{Kind: "unfinished", RequiredMS: requiredMS, ObservedMS: l.completeBeforeMS}
	}
	return nil
}

// Absent checks the entire interval after the observed anchor and before its exclusive
// end. Early counterexamples fail immediately; success requires a complete watermark.
func (l *ObservationLedger) Absent(matcher OccurrenceMatcher, anchor, durationMS uint64) error {
	end, err := l.end(anchor, durationMS)
	if err != nil {
		return err
	}
	matcher, err = matcher.checked()
	if err != nil {
		return err
	}
	for _, item := range l.occurrences {
		if item.Sequence > anchor && item.AtMS < end && matcher.matches(item) {
			return &TemporalError{Kind: "counterexample", Counterexample: copyOccurrence(item)}
		}
	}
	return l.complete(end)
}

// Stable checks the observed baseline and every scoped snapshot until the exclusive end.
// Scope selects a stream such as one account/queue; required holds the claimed metric values.
func (l *ObservationLedger) Stable(scope OccurrenceMatcher, required map[string]Node, anchor, durationMS uint64) error {
	baseline, err := l.anchor(anchor)
	if err != nil {
		return err
	}
	end, err := l.end(anchor, durationMS)
	if err != nil {
		return err
	}
	if len(required) == 0 {
		return &TemporalError{Kind: "empty_claim"}
	}
	for field := range required {
		for scoped := range scope.Payload {
			if field == scoped || strings.HasPrefix(field, scoped+".") || strings.HasPrefix(scoped, field+".") {
				return &TemporalError{Kind: "filtered_claim"}
			}
		}
	}
	scope, err = scope.checked()
	if err != nil {
		return err
	}
	required, err = temporalFields(required)
	if err != nil {
		return err
	}
	if !scope.matches(*baseline) {
		return &TemporalError{Kind: "counterexample", Counterexample: copyOccurrence(*baseline)}
	}
	for _, item := range l.occurrences {
		if item.Sequence < anchor || item.AtMS >= end || !scope.matches(item) {
			continue
		}
		for field, value := range required {
			actual, exists := lookup(item.Payload, field)
			if !exists || !responseEqual(actual, value) {
				return &TemporalError{Kind: "counterexample", Counterexample: copyOccurrence(item)}
			}
		}
	}
	return l.complete(end)
}
