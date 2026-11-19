package orchestrata

import (
	"encoding/json"
	"fmt"
	"unicode/utf8"
)

// EncodeProblemV1 encodes a typed request as the numeric, tagged ABI v1 wire
// document. The native solver remains the authority for model validity.
func EncodeProblemV1(problem Problem) ([]byte, error) {
	variables := nonNullSlice(problem.Variables)
	for index, variable := range variables {
		if !utf8.ValidString(variable.Name) || len(variable.Domain) == 0 {
			return nil, fmt.Errorf("variable %d requires a valid UTF-8 name and nonempty domain", index)
		}
	}

	constraints := make([]any, 0, len(problem.Constraints))
	for index, constraint := range problem.Constraints {
		wire, err := encodeConstraint(constraint, len(variables))
		if err != nil {
			return nil, fmt.Errorf("constraint %d: %w", index, err)
		}
		constraints = append(constraints, wire)
	}
	encoded, err := json.Marshal(struct {
		Variables   []Variable `json:"variables"`
		Constraints []any      `json:"constraints"`
	}{variables, constraints})
	if err != nil {
		return nil, err
	}
	if len(encoded) > maxV1RequestBytes {
		return nil, fmt.Errorf("encoded request exceeds the ABI v1 byte limit")
	}
	return encoded, nil
}

func encodeConstraint(constraint Constraint, variableCount int) (any, error) {
	validIndex := func(index uint32) bool {
		return uint64(index) < uint64(variableCount)
	}
	switch value := constraint.(type) {
	case AllDifferent:
		if value.HardWeight <= 0 {
			return nil, fmt.Errorf("hard weight must be positive")
		}
		for _, index := range value.Variables {
			if !validIndex(index) {
				return nil, fmt.Errorf("variable index %d is out of range", index)
			}
		}
		value.Variables = nonNullSlice(value.Variables)
		return struct {
			Kind string `json:"kind"`
			AllDifferent
		}{"all_different", value}, nil
	case NotEqualOffset:
		if value.HardWeight <= 0 || !validIndex(value.Left) || !validIndex(value.Right) {
			return nil, fmt.Errorf("requires valid left and right indices and a positive hard weight")
		}
		return struct {
			Kind string `json:"kind"`
			NotEqualOffset
		}{"not_equal_offset", value}, nil
	case PreferValue:
		if value.SoftWeight <= 0 || !validIndex(value.Variable) {
			return nil, fmt.Errorf("requires a valid variable index and a positive soft weight")
		}
		return struct {
			Kind string `json:"kind"`
			PreferValue
		}{"prefer_value", value}, nil
	case CountValue:
		if len(value.Variables) == 0 || value.Min > value.Max || uint64(value.Max) > uint64(len(value.Variables)) ||
			value.Weight <= 0 || (value.Tier != CountValueHard && value.Tier != CountValueSoft) {
			return nil, fmt.Errorf("requires a nonempty scope, inclusive bounds, positive weight, and hard or soft tier")
		}
		if err := validateDistinctIndices(value.Variables, variableCount); err != nil {
			return nil, err
		}
		return struct {
			Kind string `json:"kind"`
			CountValue
		}{"count_value", value}, nil
	case Precedence:
		if !validIndex(value.Before) || !validIndex(value.After) || value.Before == value.After {
			return nil, fmt.Errorf("requires distinct valid before and after indices")
		}
		return struct {
			Kind string `json:"kind"`
			Precedence
		}{"precedence", value}, nil
	case NoOverlap:
		if len(value.Intervals) < 2 {
			return nil, fmt.Errorf("requires at least two intervals")
		}
		indices := make([]uint32, len(value.Intervals))
		for i, interval := range value.Intervals {
			indices[i] = interval.Start
		}
		if err := validateDistinctIndices(indices, variableCount); err != nil {
			return nil, err
		}
		return struct {
			Kind string `json:"kind"`
			NoOverlap
		}{"no_overlap", value}, nil
	case ResourceCapacity:
		if len(value.Items) == 0 || len(value.Limits) == 0 {
			return nil, fmt.Errorf("requires at least one item and resource limit")
		}
		indices := make([]uint32, len(value.Items))
		for i, item := range value.Items {
			indices[i] = item.Variable
		}
		if err := validateDistinctIndices(indices, variableCount); err != nil {
			return nil, err
		}
		limits := make(map[int64]struct{}, len(value.Limits))
		for _, limit := range value.Limits {
			if _, exists := limits[limit.Resource]; exists {
				return nil, fmt.Errorf("duplicate resource limit %d", limit.Resource)
			}
			limits[limit.Resource] = struct{}{}
		}
		return struct {
			Kind string `json:"kind"`
			ResourceCapacity
		}{"resource_capacity", value}, nil
	default:
		return nil, fmt.Errorf("expected a supported constraint value (not a pointer)")
	}
}

func validateDistinctIndices(indices []uint32, variableCount int) error {
	seen := make(map[uint32]struct{}, len(indices))
	for _, index := range indices {
		if uint64(index) >= uint64(variableCount) {
			return fmt.Errorf("variable index %d is out of range", index)
		}
		if _, exists := seen[index]; exists {
			return fmt.Errorf("duplicate variable index %d", index)
		}
		seen[index] = struct{}{}
	}
	return nil
}

func nonNullSlice[T any](slice []T) []T {
	if slice == nil {
		return []T{}
	}
	return slice
}
