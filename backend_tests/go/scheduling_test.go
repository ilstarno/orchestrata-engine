package orchestrata_test

import (
	orchestrata "github.com/ilstarno/Orchestrata/sdks/go"
	"math"
	"reflect"
	"testing"
)

func TestSchedulingMatchesIndependentExhaustiveOracle(t *testing.T) {
	for duration := int64(1); duration <= 5; duration++ {
		for first := int64(1); first <= 4; first++ {
			for second := int64(1); second <= 4; second++ {
				durations := [3]int64{duration, 1, 1}
				weights := [3]int64{first, second, 3}
				p := orchestrata.Problem{Variables: []orchestrata.Variable{{Name: "cut", Domain: []int64{0, 1, 2, 3, 4}}, {Name: "inspect", Domain: []int64{0, 1, 2, 3, 4}}, {Name: "pack", Domain: []int64{0, 1, 2, 3, 4}}}, Constraints: []orchestrata.Constraint{
					orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 0, Duration: uint64(duration)}, {Start: 1, Duration: 1}, {Start: 2, Duration: 1}}},
					orchestrata.Precedence{Before: 0, After: 2, MinGap: 2},
					orchestrata.PreferValue{Variable: 0, Value: 0, SoftWeight: first}, orchestrata.PreferValue{Variable: 1, Value: 0, SoftWeight: second}, orchestrata.PreferValue{Variable: 2, Value: 2, SoftWeight: 3},
				}}
				best := int64(math.MaxInt64)
				var values []int64
				for a := int64(0); a < 5; a++ {
					for b := int64(0); b < 5; b++ {
						for c := int64(0); c < 5; c++ {
							v := []int64{a, b, c}
							if c < a+2 {
								continue
							}
							valid := true
							for i := 0; i < 3; i++ {
								for j := i + 1; j < 3; j++ {
									if v[i] < v[j]+durations[j] && v[j] < v[i]+durations[i] {
										valid = false
									}
								}
							}
							if !valid {
								continue
							}
							cost := int64(0)
							for i, target := range []int64{0, 0, 2} {
								if v[i] != target {
									cost += weights[i]
								}
							}
							if cost < best {
								best = cost
								values = v
							}
						}
					}
				}
				result, err := orchestrata.SolveProblem(p, 125)
				if err != nil {
					t.Fatal(err)
				}
				if values == nil {
					if result != nil {
						t.Fatalf("expected infeasible: %+v", result)
					}
					continue
				}
				if result == nil || !reflect.DeepEqual(result.Values, values) || result.Score.Hard != 0 || result.Score.Soft != -best {
					t.Fatalf("got %+v expected %v cost %d", result, values, best)
				}
			}
		}
	}
}

func TestCountAndWeightedCapacityTypedValues(t *testing.T) {
	p := orchestrata.Problem{Variables: []orchestrata.Variable{{Name: "a", Domain: []int64{0, 1}}, {Name: "b", Domain: []int64{0, 1}}, {Name: "c", Domain: []int64{0, 1}}}, Constraints: []orchestrata.Constraint{
		orchestrata.CountValue{Variables: []uint32{0, 1, 2}, Value: 0, Min: 1, Max: 2, Weight: 1, Tier: orchestrata.CountValueHard},
		orchestrata.ResourceCapacity{Items: []orchestrata.ResourceLoad{{Variable: 0, Load: 2}, {Variable: 1, Load: 3}, {Variable: 2, Load: 4}}, Limits: []orchestrata.ResourceLimit{{Resource: 0, Capacity: 6}, {Resource: 1, Capacity: 6}}},
		orchestrata.PreferValue{Variable: 0, Value: 0, SoftWeight: 7}, orchestrata.PreferValue{Variable: 1, Value: 0, SoftWeight: 6}, orchestrata.PreferValue{Variable: 2, Value: 0, SoftWeight: 5},
	}}
	r, err := orchestrata.SolveProblem(p, 8)
	if err != nil || r == nil || !reflect.DeepEqual(r.Values, []int64{0, 0, 1}) || r.Score.Soft != -5 {
		t.Fatalf("result %+v error %v", r, err)
	}
}

func TestSchedulingBoundsAndRejectedTypedRules(t *testing.T) {
	p := orchestrata.Problem{Variables: []orchestrata.Variable{{Name: "a", Domain: []int64{math.MinInt64}}, {Name: "b", Domain: []int64{math.MaxInt64}}}, Constraints: []orchestrata.Constraint{
		orchestrata.Precedence{Before: 0, After: 1, MinGap: math.MaxUint64}, orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 0, Duration: math.MaxUint64}, {Start: 1, Duration: 0}}},
	}}
	r, err := orchestrata.SolveProblem(p, 1)
	if err != nil || r == nil || !reflect.DeepEqual(r.Values, []int64{math.MinInt64, math.MaxInt64}) {
		t.Fatalf("result %+v error %v", r, err)
	}
	for _, rule := range []orchestrata.Constraint{
		orchestrata.Precedence{Before: 0, After: 0}, orchestrata.Precedence{Before: 0, After: 2},
		orchestrata.NoOverlap{}, orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{{Start: 0}, {Start: 0}}},
		orchestrata.CountValue{Variables: []uint32{0}, Min: 2, Max: 1, Weight: 1, Tier: orchestrata.CountValueHard},
		orchestrata.CountValue{Variables: []uint32{0}, Max: 1, Weight: 1, Tier: "unknown"},
		orchestrata.ResourceCapacity{}, orchestrata.ResourceCapacity{Items: []orchestrata.ResourceLoad{{Variable: 2}}},
	} {
		p.Constraints = []orchestrata.Constraint{rule}
		if _, err := orchestrata.EncodeProblemV1(p); err == nil {
			t.Fatalf("accepted %#v", rule)
		}
	}
}
