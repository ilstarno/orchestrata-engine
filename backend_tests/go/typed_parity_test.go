package orchestrata_test

import (
	"errors"
	"math"
	"reflect"
	"strings"
	"testing"

	orchestrata "github.com/ilstarno/Orchestrata/sdks/go"
)

func TestAdditionalConstraintWireTypesAndUnits(t *testing.T) {
	problem := orchestrata.Problem{
		Variables: []orchestrata.Variable{
			{Name: "first", Domain: []int64{-1}},
			{Name: "second", Domain: []int64{2}},
		},
		Constraints: []orchestrata.Constraint{
			orchestrata.CountValue{Variables: []uint32{0, 1}, Value: math.MinInt64, Min: 0, Max: 2, Weight: 5, Tier: orchestrata.CountValueSoft},
			orchestrata.Precedence{Before: 0, After: 1, MinGap: math.MaxUint64},
			orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 0, Duration: math.MaxUint64}, {Start: 1, Duration: 0}}},
			orchestrata.ResourceCapacity{
				Items:  []orchestrata.ResourceLoad{{Variable: 0, Load: math.MaxUint64}},
				Limits: []orchestrata.ResourceLimit{{Resource: -1, Capacity: math.MaxUint64}},
			},
		},
	}
	wire, err := orchestrata.EncodeProblemV1(problem)
	if err != nil {
		t.Fatal(err)
	}
	const expected = `{"variables":[{"name":"first","domain":[-1]},{"name":"second","domain":[2]}],"constraints":[{"kind":"count_value","variables":[0,1],"value":-9223372036854775808,"min":0,"max":2,"weight":5,"tier":"soft"},{"kind":"precedence","before":0,"after":1,"min_gap":18446744073709551615},{"kind":"no_overlap","intervals":[{"start":0,"duration":18446744073709551615},{"start":1,"duration":0}]},{"kind":"resource_capacity","items":[{"variable":0,"load":18446744073709551615}],"limits":[{"resource":-1,"capacity":18446744073709551615}]}]}`
	if string(wire) != expected {
		t.Fatalf("wire = %s; expected %s", wire, expected)
	}
}

func TestNativeSolveSupportsAllFourAddedConstraintTypes(t *testing.T) {
	tests := []struct {
		name        string
		left, right []int64
		constraints []orchestrata.Constraint
		values      []int64
		evaluated   uint64
	}{
		{
			name: "count_value", left: []int64{0, 1}, right: []int64{0, 1},
			constraints: []orchestrata.Constraint{
				orchestrata.CountValue{Variables: []uint32{0, 1}, Value: 1, Min: 1, Max: 1, Weight: 1, Tier: orchestrata.CountValueHard},
				orchestrata.PreferValue{Variable: 0, Value: 1, SoftWeight: 1},
			}, values: []int64{1, 0}, evaluated: 4,
		},
		{
			name: "precedence", left: []int64{0}, right: []int64{-1, 5},
			constraints: []orchestrata.Constraint{orchestrata.Precedence{Before: 0, After: 1, MinGap: 5}},
			values:      []int64{0, 5}, evaluated: 2,
		},
		{
			name: "no_overlap", left: []int64{0}, right: []int64{3, 5},
			constraints: []orchestrata.Constraint{orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{
				{Start: 0, Duration: 5}, {Start: 1, Duration: 3},
			}}}, values: []int64{0, 5}, evaluated: 2,
		},
		{
			name: "resource_capacity", left: []int64{-7, 8}, right: []int64{-7, 8},
			constraints: []orchestrata.Constraint{orchestrata.ResourceCapacity{
				Items:  []orchestrata.ResourceLoad{{Variable: 0, Load: 5}, {Variable: 1, Load: 3}},
				Limits: []orchestrata.ResourceLimit{{Resource: -7, Capacity: 3}, {Resource: 8, Capacity: 5}},
			}}, values: []int64{8, -7}, evaluated: 4,
		},
	}
	for _, row := range tests {
		t.Run(row.name, func(t *testing.T) {
			input := orchestrata.Problem{Variables: []orchestrata.Variable{
				{Name: "left", Domain: row.left}, {Name: "right", Domain: row.right},
			}, Constraints: row.constraints}
			result, err := orchestrata.SolveProblem(input, row.evaluated)
			if err != nil || result == nil {
				t.Fatalf("result = %+v, error = %v", result, err)
			}
			if !reflect.DeepEqual(result.Values, row.values) || result.Score != (orchestrata.Score{}) || result.Evaluated != row.evaluated {
				t.Fatalf("unexpected result %+v", result)
			}
		})
	}
}

func TestNewTypesRejectMalformedRepresentationsButLeaveCapacitySemanticsNative(t *testing.T) {
	variables := []orchestrata.Variable{{Name: "one", Domain: []int64{0}}, {Name: "two", Domain: []int64{1}}}
	for _, bad := range []orchestrata.Constraint{
		orchestrata.CountValue{Variables: []uint32{0, 0}, Max: 2, Weight: 1, Tier: orchestrata.CountValueHard},
		orchestrata.CountValue{Variables: []uint32{0}, Max: 1, Weight: 1, Tier: "medium"},
		orchestrata.CountValue{Variables: []uint32{0}, Min: 2, Max: 1, Weight: 1, Tier: orchestrata.CountValueSoft},
		orchestrata.Precedence{Before: 0, After: 0},
		orchestrata.Precedence{Before: 2, After: 0},
		orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 0, Duration: 1}}},
		orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 1}, {Start: 1}}},
		orchestrata.ResourceCapacity{Items: []orchestrata.ResourceLoad{{Variable: 2}}, Limits: []orchestrata.ResourceLimit{{Resource: 0}}},
		orchestrata.ResourceCapacity{Items: []orchestrata.ResourceLoad{{Variable: 0}}, Limits: []orchestrata.ResourceLimit{{Resource: 0}, {Resource: 0}}},
	} {
		problem := orchestrata.Problem{Variables: variables, Constraints: []orchestrata.Constraint{bad}}
		if _, err := orchestrata.EncodeProblemV1(problem); err == nil {
			t.Errorf("accepted invalid typed %T: %+v", bad, bad)
		}
	}
	semantic := orchestrata.Problem{Variables: variables, Constraints: []orchestrata.Constraint{orchestrata.ResourceCapacity{
		Items: []orchestrata.ResourceLoad{{Variable: 0, Load: 1}}, Limits: []orchestrata.ResourceLimit{{Resource: 99, Capacity: 1}},
	}}}
	if _, err := orchestrata.EncodeProblemV1(semantic); err != nil {
		t.Fatalf("structurally valid typed input should reach the native validator: %v", err)
	}
	_, err := orchestrata.SolveProblem(semantic, 1)
	var native *orchestrata.SolveError
	if !errors.As(err, &native) || native.Status != orchestrata.InvalidProblem {
		t.Fatalf("expected native invalid model, got %v", err)
	}
}

func TestTypedResultChecksRequestDomainsAndOnlyExplicitNullIsInfeasible(t *testing.T) {
	problem := orchestrata.Problem{Variables: []orchestrata.Variable{{Name: "x", Domain: []int64{math.MinInt64, 0}}}}
	response := func(values string) []byte {
		return []byte(`{"solution":{"values":` + values + `,"score":{"hard":0,"soft":0},"evaluated":1}}`)
	}
	if result, err := orchestrata.DecodeResultForProblemV1(response(`[-9223372036854775808]`), problem); err != nil ||
		result == nil || result.Values[0] != math.MinInt64 {
		t.Fatalf("unexpected verified result %+v / %v", result, err)
	}
	for _, row := range []struct {
		wire []byte
		want string
	}{
		{[]byte(`{}`), "decode Orchestrata response"},
		{response(`[]`), "count does not match"},
		{response(`[1]`), "outside its request domain"},
	} {
		result, err := orchestrata.DecodeResultForProblemV1(row.wire, problem)
		if result != nil || err == nil || !strings.Contains(err.Error(), row.want) {
			t.Errorf("expected %q for %s, got %+v / %v", row.want, row.wire, result, err)
		}
	}
	if result, err := orchestrata.DecodeResultForProblemV1([]byte(`{"solution":null}`), problem); result != nil || err != nil {
		t.Fatalf("explicit null = %+v / %v", result, err)
	}
}
