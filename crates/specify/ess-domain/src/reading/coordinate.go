// ClockReadingEvidence contains adapter-observed facts for one occurrence. Empty identity refuses.
type ClockReadingEvidence struct {
	Correlation     string
	Occurrence      string
	ProcessInstance string
	Epoch           string
	Origin          string
	Formatter       string
	OffsetMinutes   *int16
}

// ClockCoordinate is a bounded coordinate within an observed process epoch, not physical elapsed time.
type ClockCoordinate struct {
	UnixMillis      int64
	ProcessInstance string
	Epoch           string
}

// ResolveClockReading checks occurrence, source, declared origin and observed formatter before normalization.
func ResolveClockReading(encoding string, origins [][2]string, value string, evidence ClockReadingEvidence, correlation, occurrence string) (ClockCoordinate, error) {
	fail := func(message string) (ClockCoordinate, error) {
		return ClockCoordinate{}, fmt.Errorf("clock reading: %s", message)
	}
	if evidence.Correlation != correlation || evidence.Occurrence != occurrence {
		return fail("MismatchedOccurrence")
	}
	for _, identity := range []string{evidence.Correlation, evidence.Occurrence, evidence.ProcessInstance, evidence.Epoch} {
		if len(identity) == 0 || len(identity) > 512 {
			return fail("UnknownEvidence")
		}
	}
	if evidence.Origin != "producer_process" && evidence.Origin != "consumer_process" {
		return fail("UnknownEvidence")
	}
	if len(origins) == 0 || len(origins) > 3 {
		return fail("InvalidContract")
	}
	for index, origin := range origins {
		valid := origin[0] == "producer_process" || origin[0] == "consumer_process" || origin[0] == "unknown"
		valid = valid && ((encoding == "offset_date_time_text" && origin[1] == "encoded_offset") || (encoding == "unix_seconds" && origin[1] == "encoding_defined_epoch") || (encoding == "local_date_time_millis_literal_z" && (origin[1] == "requires_observation" || origin[1] == "unknown")))
		for _, previous := range origins[:index] {
			if previous == origin {
				valid = false
			}
		}
		if !valid {
			return fail("InvalidContract")
		}
	}
	authority := ""
	switch {
	case encoding == "offset_date_time_text" && evidence.Formatter == "encoded_offset" && evidence.OffsetMinutes == nil:
		authority = "encoded_offset"
	case encoding == "unix_seconds" && evidence.Formatter == "unix_seconds" && evidence.OffsetMinutes == nil:
		authority = "encoding_defined_epoch"
	case encoding == "local_date_time_millis_literal_z" && evidence.Formatter == "fixed_offset" && evidence.OffsetMinutes != nil:
		authority = "requires_observation"
	case evidence.Formatter == "unknown":
		return fail("UnknownEvidence")
	default:
		return fail("IncompatibleFormatter")
	}
	admitted := false
	for _, origin := range origins {
		if origin == [2]string{evidence.Origin, authority} {
			admitted = true
		}
	}
	if !admitted {
		return fail("IncompatibleOrigin")
	}
	var millis int64
	if encoding == "unix_seconds" {
		seconds, err := strconv.ParseInt(value, 10, 64)
		if err != nil || strconv.FormatInt(seconds, 10) != value {
			return fail("InvalidValue")
		}
		if seconds < 0 || seconds > 253402300799 {
			return fail("OutOfRange")
		}
		millis = seconds * 1000
	} else {
		var err error
		millis, err = normalizeClockText(value, evidence.OffsetMinutes)
		if err != nil {
			return ClockCoordinate{}, err
		}
	}
	return ClockCoordinate{millis, evidence.ProcessInstance, evidence.Epoch}, nil
}

func normalizeClockText(text string, localOffset *int16) (int64, error) {
	invalid := func() (int64, error) { return 0, fmt.Errorf("clock reading: InvalidValue") }
	if len(text) < 20 || len(text) > 29 {
		return invalid()
	}
	for _, c := range []byte(text) {
		if c > 127 {
			return invalid()
		}
	}
	if text[4] != '-' || text[7] != '-' || text[10] != 'T' || text[13] != ':' || text[16] != ':' {
		return invalid()
	}
	digits := func(part string) int {
		value := 0
		for _, c := range []byte(part) {
			if c < '0' || c > '9' {
				return -1
			}
			value = value*10 + int(c-'0')
		}
		return value
	}
	year, month, day := digits(text[:4]), digits(text[5:7]), digits(text[8:10])
	hour, minute, second := digits(text[11:13]), digits(text[14:16]), digits(text[17:19])
	if year < 1970 || year > 9999 {
		return 0, fmt.Errorf("clock reading: OutOfRange")
	}
	if month < 1 || month > 12 || day < 1 || day > 31 || hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 {
		return invalid()
	}
	millis, position := 0, 19
	if text[position] == '.' {
		if len(text) < 24 {
			return invalid()
		}
		millis = digits(text[20:23])
		if millis < 0 {
			return invalid()
		}
		position = 23
	}
	suffix := text[position:]
	offset := 0
	if localOffset != nil {
		if position != 23 || suffix != "Z" {
			return invalid()
		}
		offset = int(*localOffset)
	} else if suffix != "Z" {
		if len(suffix) != 6 || (suffix[0] != '+' && suffix[0] != '-') || suffix[3] != ':' {
			return invalid()
		}
		h, m := digits(suffix[1:3]), digits(suffix[4:6])
		if h < 0 || m < 0 {
			return invalid()
		}
		if h > 14 || m > 59 || (h == 14 && m != 0) {
			return 0, fmt.Errorf("clock reading: OutOfRange")
		}
		offset = h*60 + m
		if suffix[0] == '-' {
			offset = -offset
		}
	}
	if offset < -840 || offset > 840 {
		return 0, fmt.Errorf("clock reading: OutOfRange")
	}
	instant := time.Date(year, time.Month(month), day, hour, minute, second, millis*1000000, time.UTC)
	if instant.Year() != year || int(instant.Month()) != month || instant.Day() != day {
		return invalid()
	}
	result := instant.UnixMilli() - int64(offset)*60000
	if result < 0 || result > 253402300799999 {
		return 0, fmt.Errorf("clock reading: OutOfRange")
	}
	return result, nil
}

// CompareClockReadings orders coordinates only when both observed source identities and epochs agree.
func CompareClockReadings(left, right ClockCoordinate) (int, error) {
	if left.ProcessInstance == "" || left.Epoch == "" || right.ProcessInstance == "" || right.Epoch == "" {
		return 0, fmt.Errorf("clock reading: UnknownEvidence")
	}
	if left.ProcessInstance != right.ProcessInstance || left.Epoch != right.Epoch {
		return 0, fmt.Errorf("clock reading: DifferentClock")
	}
	if left.UnixMillis < right.UnixMillis {
		return -1, nil
	}
	if left.UnixMillis > right.UnixMillis {
		return 1, nil
	}
	return 0, nil
}
