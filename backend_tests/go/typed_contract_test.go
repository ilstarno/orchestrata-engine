package orchestrata_test

import (
	"errors"
	"math"
	"reflect"
	"strings"
	"testing"

	orchestrata "github.com/ilstarno/Orchestrata/sdks/go"
)

func TestEncodeTypedV1Golden(t *testing.T) {
	problem := typedProblem()
	encoded, err := orchestrata.EncodeProblemV1(problem)
	if err != nil {
		t.Fatal(err)
	}
	expected := `{"variables":[{"name":"a","domain":[-9223372036854775808,9223372036854775807]},{"name":"b","domain":[-9223372036854775808,9223372036854775807]}],"constraints":[{"kind":"all_different","variables":[0,1],"hard_weight":1},{"kind":"not_equal_offset","left":0,"right":1,"offset":-9223372036854775808,"hard_weight":1},{"kind":"prefer_value","variable":0,"value":9223372036854775807,"soft_weight":9223372036854775807}]}`
	if string(encoded) != expected {
		t.Fatalf("encoded = %s; expected %s", encoded, expected)
	}
	empty, err := orchestrata.EncodeProblemV1(orchestrata.Problem{})
	if err != nil || string(empty) != `{"variables":[],"constraints":[]}` {
		t.Fatalf("empty = %s, error = %v", empty, err)
	}
}

func TestNativeTypedSolvePreservesValuesAndStatusDistinctions(t *testing.T) {
	problem := typedProblem()
	solution, err := orchestrata.SolveProblem(problem, math.MaxUint64)
	if err != nil || solution == nil {
		t.Fatalf("solution = %+v, error = %v", solution, err)
	}
	if !reflect.DeepEqual(solution.Values, []int64{math.MaxInt64, math.MinInt64}) ||
		solution.Score != (orchestrata.Score{}) || solution.Evaluated != 4 {
		t.Fatalf("unexpected solution %+v", solution)
	}

	_, err = orchestrata.SolveProblem(problem, 3)
	var native *orchestrata.SolveError
	if !errors.As(err, &native) || native.Status != orchestrata.SearchLimit {
		t.Fatalf("expected native search limit, got %v", err)
	}

	impossible := orchestrata.Problem{
		Variables: []orchestrata.Variable{{Name: "x", Domain: []int64{0}}},
		Constraints: []orchestrata.Constraint{
			orchestrata.NotEqualOffset{Left: 0, Right: 0, Offset: 0, HardWeight: 1},
		},
	}
	solution, err = orchestrata.SolveProblem(impossible, 1)
	if err != nil || solution != nil {
		t.Fatalf("expected proved infeasibility, got %+v %v", solution, err)
	}
}

func TestTypedCodecRejectsOutOfDomainReferencesAndNullConstraints(t *testing.T) {
	for _, constraint := range []orchestrata.Constraint{
		nil,
		(*orchestrata.PreferValue)(nil),
		orchestrata.PreferValue{Variable: 2, Value: 1, SoftWeight: 1},
		orchestrata.NotEqualOffset{Left: 0, Right: math.MaxUint32, HardWeight: 1},
		orchestrata.AllDifferent{Variables: []uint32{0}, HardWeight: 0},
	} {
		problem := typedProblem()
		problem.Constraints = []orchestrata.Constraint{constraint}
		if _, err := orchestrata.EncodeProblemV1(problem); err == nil {
			t.Fatalf("expected typed input error for %#v", constraint)
		}
	}
	problem := typedProblem()
	problem.Variables[0].Name = string([]byte{0xff})
	if _, err := orchestrata.EncodeProblemV1(problem); err == nil {
		t.Fatal("invalid UTF-8 must not be normalized or replaced")
	}
}

func TestTypedCodecChecksABIByteLimits(t *testing.T) {
	problem := typedProblem()
	problem.Variables[0].Name = strings.Repeat("x", 1<<20)
	if _, err := orchestrata.EncodeProblemV1(problem); err == nil {
		t.Fatal("oversized typed request was accepted")
	}
	invalid := []byte(`{"solution":null}` + strings.Repeat(" ", 2<<20))
	if _, err := orchestrata.DecodeResultV1(invalid); err == nil {
		t.Fatal("oversized typed result was accepted")
	}
}

func TestDecodeTypedV1GoldensKeepEveryIntegerExact(t *testing.T) {
	wire := `{"solution":{"values":[-9223372036854775808,9223372036854775807,9007199254740993],"score":{"hard":0,"soft":-9223372036854775808},"evaluated":18446744073709551615}}`
	solution, err := orchestrata.DecodeResultV1([]byte(wire))
	if err != nil || solution == nil ||
		!reflect.DeepEqual(solution.Values, []int64{math.MinInt64, math.MaxInt64, 9_007_199_254_740_993}) ||
		solution.Score.Soft != math.MinInt64 || solution.Evaluated != math.MaxUint64 {
		t.Fatalf("solution = %+v, error = %v", solution, err)
	}
	if none, err := orchestrata.DecodeResultV1([]byte(`{"solution": null}`)); none != nil || err != nil {
		t.Fatalf("explicit null decode = %+v, error = %v", none, err)
	}
}

func TestDecodeTypedV1RejectsAmbiguousOrInvalidOutcomes(t *testing.T) {
	for _, wire := range []string{
		`{}`,
		`{"solution":null,"solution":null}`,
		`{"solution":null,"unknown":0}`,
		`{"solution":null} []`,
		`{"solution":{"score":{"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":null,"score":{"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[null],"score":{"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[9223372036854775808],"score":{"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[1.0],"score":{"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[],"score":{"hard":null,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[],"score":{"hard":0,"hard":0,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[],"score":{"hard":-1,"soft":0},"evaluated":1}}`,
		`{"solution":{"values":[],"score":{"hard":0,"soft":1},"evaluated":1}}`,
		`{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":0}}`,
		`{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":-1}}`,
		`{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":18446744073709551616}}`,
	} {
		t.Run(wire, func(t *testing.T) {
			if solution, err := orchestrata.DecodeResultV1([]byte(wire)); solution != nil || err == nil {
				t.Fatalf("expected protocol error, got %+v %v", solution, err)
			}
		})
	}
}

func typedProblem() orchestrata.Problem {
	return orchestrata.Problem{
		Variables: []orchestrata.Variable{
			{Name: "a", Domain: []int64{math.MinInt64, math.MaxInt64}},
			{Name: "b", Domain: []int64{math.MinInt64, math.MaxInt64}},
		},
		Constraints: []orchestrata.Constraint{
			orchestrata.AllDifferent{Variables: []uint32{0, 1}, HardWeight: 1},
			orchestrata.NotEqualOffset{Left: 0, Right: 1, Offset: math.MinInt64, HardWeight: 1},
			orchestrata.PreferValue{Variable: 0, Value: math.MaxInt64, SoftWeight: math.MaxInt64},
		},
	}
}
