// ReadingReference names an already observed event occurrence/member and a contract the adapter must validate.
type ReadingReference struct {
	Event        string `json:"event"`
	Occurrence   uint32 `json:"occurrence"`
	Field        string `json:"field"`
	DeclaredType string `json:"declared_type"`
	Contract     struct {
		Encoding string `json:"encoding"`
		Origins  []struct {
			Role   string `json:"role"`
			Offset string `json:"offset"`
		} `json:"origins"`
	} `json:"contract"`
}

// ReadingObservationRequest asks for facts, never a normalized coordinate or assertion result.
type ReadingObservationRequest struct {
	Reading     ReadingReference
	Correlation string
}

// ClockReadingTarget validates the actual declared member contract and observes its source epoch and formatter.
// Unsupported evidence must return ErrUnsupported; an authored contract is never a certificate.
type ClockReadingTarget interface {
	ObserveClockReading(ReadingObservationRequest) (ClockReadingEvidence, error)
}

func (r ReadingReference) occurrenceKey() string {
	return fmt.Sprintf("%s#%d.%s", r.Event, r.Occurrence, r.Field)
}
func admitReading(value any) error {
	f, err := closed(value, "event occurrence field declared_type contract", "")
	if err != nil {
		return err
	}
	if err = name(f["event"], false); err != nil {
		return err
	}
	if err = name(f["declared_type"], false); err != nil {
		return err
	}
	n, err := unsigned(f["occurrence"])
	if err != nil || n > 65535 {
		return fmt.Errorf("invalid reading occurrence")
	}
	field, err := text(f["field"])
	if err != nil || len(field) > 128 || !regexp.MustCompile(`^[A-Za-z][A-Za-z0-9_]*$`).MatchString(field) {
		return fmt.Errorf("invalid reading member")
	}
	event, _ := text(f["event"])
	if len(fmt.Sprintf("%s#%d.%s", event, n, field)) > 512 {
		return fmt.Errorf("reading occurrence exceeds bound")
	}
	contract, err := closed(f["contract"], "encoding origins", "")
	if err != nil {
		return err
	}
	encoding, err := text(contract["encoding"])
	if err != nil {
		return err
	}
	origins, ok := contract["origins"].([]any)
	if !ok || len(origins) == 0 || len(origins) > 3 {
		return fmt.Errorf("invalid reading origins")
	}
	previous := map[[2]string]bool{}
	for _, v := range origins {
		origin, err := closed(v, "role offset", "")
		if err != nil {
			return err
		}
		role, err := text(origin["role"])
		if err != nil {
			return err
		}
		offset, err := text(origin["offset"])
		if err != nil {
			return err
		}
		pair := [2]string{role, offset}
		if previous[pair] {
			return fmt.Errorf("duplicate reading origin")
		}
		previous[pair] = true
		if role != "producer_process" && role != "consumer_process" && role != "unknown" {
			return fmt.Errorf("invalid reading origin")
		}
		if !((encoding == "offset_date_time_text" && offset == "encoded_offset") || (encoding == "unix_seconds" && offset == "encoding_defined_epoch") || (encoding == "local_date_time_millis_literal_z" && (offset == "requires_observation" || offset == "unknown"))) {
			return fmt.Errorf("contradictory reading encoding and authority")
		}
	}
	return nil
}

func (r *run) expectReadingOrder(index int, step Step) bool {
	target, ok := r.target.(ClockReadingTarget)
	if !ok {
		r.skip("clock reading evidence unsupported")
		return false
	}
	resolve := func(reference *ReadingReference) (ClockCoordinate, error) {
		if reference == nil {
			return ClockCoordinate{}, fmt.Errorf("missing reading reference")
		}
		occurrence := uint32(0)
		var value any
		found := false
		for _, event := range r.seen {
			if event.Event == reference.Event {
				if occurrence == reference.Occurrence {
					value, found = event.Payload[reference.Field]
					break
				}
				occurrence++
			}
		}
		if !found {
			return ClockCoordinate{}, fmt.Errorf("reading event occurrence/member not observed")
		}
		scalar, ok := value.(string)
		if reference.Contract.Encoding == "unix_seconds" {
			switch v := value.(type) {
			case json.Number:
				scalar, ok = clockIntegerText(v.String())
			case float64:
				ok = !math.IsNaN(v) && !math.IsInf(v, 0) && v >= 0 && v <= 253402300799 && v == math.Trunc(v)
				if ok {
					scalar = strconv.FormatInt(int64(v), 10)
				}
			case int64:
				scalar = strconv.FormatInt(v, 10)
				ok = true
			case int:
				scalar = strconv.Itoa(v)
				ok = true
			default:
				ok = false
			}
		}
		if !ok {
			return ClockCoordinate{}, fmt.Errorf("reading scalar does not match encoding")
		}
		requestReference := *reference
		// The target owns its request, including nested origins; it cannot rewrite
		// the admitted contract used by this or a later assertion.
		requestReference.Contract.Origins = append(requestReference.Contract.Origins[:0:0], requestReference.Contract.Origins...)
		evidence, err := target.ObserveClockReading(ReadingObservationRequest{requestReference, r.correlation})
		if err != nil {
			return ClockCoordinate{}, err
		}
		origins := make([][2]string, 0, len(reference.Contract.Origins))
		for _, origin := range reference.Contract.Origins {
			origins = append(origins, [2]string{origin.Role, origin.Offset})
		}
		result, err := ResolveClockReading(reference.Contract.Encoding, origins, scalar, evidence, r.correlation, reference.occurrenceKey())
		if err != nil && strings.Contains(err.Error(), "UnknownEvidence") {
			return ClockCoordinate{}, fmt.Errorf("%w: %v", ErrUnsupported, err)
		}
		return result, err
	}
	left, err := resolve(step.ReadingLeft)
	if err == nil {
		var right ClockCoordinate
		right, err = resolve(step.ReadingRight)
		if err == nil {
			var order int
			order, err = CompareClockReadings(left, right)
			if err == nil {
				if (step.ReadingOrder == "before" && order < 0) || (step.ReadingOrder == "equal" && order == 0) || (step.ReadingOrder == "after" && order > 0) {
					return true
				}
				return r.fail(index, "clock coordinate order differs from %s", step.ReadingOrder)
			}
		}
	}
	if errors.Is(err, ErrUnsupported) || (err != nil && strings.Contains(err.Error(), "DifferentClock")) {
		r.skip("clock comparison unsupported: %v", err)
		return false
	}
	return r.fail(index, "clock reading: %v", err)
}

// Canonicalize exact mathematical integer tokens without a floating-point round trip.
func clockIntegerText(raw string) (string, bool) {
	if len(raw) > 512 || !regexp.MustCompile(`^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?$`).MatchString(raw) {
		return "", false
	}
	negative := strings.HasPrefix(raw, "-")
	if negative {
		raw = raw[1:]
	}
	exponent := 0
	if index := strings.IndexAny(raw, "eE"); index >= 0 {
		var err error
		exponent, err = strconv.Atoi(raw[index+1:])
		if err != nil || exponent < -512 || exponent > 512 {
			return "", false
		}
		raw = raw[:index]
	}
	if index := strings.IndexByte(raw, '.'); index >= 0 {
		exponent -= len(raw) - index - 1
		raw = raw[:index] + raw[index+1:]
	}
	raw = strings.TrimLeft(raw, "0")
	if raw == "" {
		return "0", true
	}
	for exponent < 0 && strings.HasSuffix(raw, "0") {
		raw = strings.TrimSuffix(raw, "0")
		exponent++
	}
	if exponent < 0 || negative || len(raw)+exponent > 12 {
		return "", false
	}
	return raw + strings.Repeat("0", exponent), true
}
