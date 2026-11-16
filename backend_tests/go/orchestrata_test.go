package orchestrata_test

import (
	"errors"
	"math"
	"testing"

	orchestrata "github.com/ilstarno/Orchestrata/sdks/go"
)

func TestOptimalAndExactInteger(t *testing.T) {
	problem := []byte(`{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}`)
	solution, err := orchestrata.Solve(problem, 1)
	if err != nil || solution == nil || len(solution.Values) != 1 || solution.Values[0] != math.MaxInt64 {
		t.Fatalf("unexpected solution %+v, error %v", solution, err)
	}
}

func TestSolveLimit(t *testing.T) {
	_, err := orchestrata.Solve([]byte(`{"variables":[],"constraints":[]}`), 0)
	var solverError *orchestrata.SolveError
	if !errors.As(err, &solverError) || solverError.Status != orchestrata.SearchLimit {
		t.Fatalf("unexpected error: %v", err)
	}
}
