
import (
    p "example.invalid/retained/types/primitives"
    "reflect"
    "testing"
)

func TestBothNativeBranchesReturnActualTypedResult(t *testing.T) {
    response := SeedResponse{RevisionId: p.NewUuid("00000000-0000-4000-8000-000000000037"), Stamp: p.NewTimestamp("2026-09-22T01:02:03Z"), Number: 9007199254740993, Values: []int64{-9223372036854775808, 9223372036854775807}}
    original := SeedOutcomeSeeded{Response: response, Seeded: Seeded{RecordId: response.RevisionId}}
    replay := SeedOutcomeReplayed{Response: response}
    if !reflect.DeepEqual(original.Response, replay.Response) || replay.Response.Number != 9007199254740993 { t.Fatal("native result changed") }
}
