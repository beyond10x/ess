package essconform

import (
	"encoding/json"
	"os"
	"strconv"
)

func init() {
	countReportNow = func() int64 {
		value, err := strconv.ParseInt(os.Getenv("ESS_FIXTURE_CLOCK"), 10, 64)
		if err != nil {
			panic(err)
		}
		file, err := os.OpenFile(os.Getenv("ESS_FIXTURE_CLOCK_LOG"), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0600)
		if err != nil {
			panic(err)
		}
		defer file.Close()
		if err := json.NewEncoder(file).Encode(value); err != nil {
			panic(err)
		}
		return value
	}
}
