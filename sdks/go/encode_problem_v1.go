package orchestrata

import (
	"encoding/json"
	"fmt"
	"unicode/utf8"
)

// EncodeProblemV1 encodes a typed request as the numeric, tagged ABI v1 wire
// document. Only the three known constraint value types are accepted.
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
	default:
		return nil, fmt.Errorf("expected an AllDifferent, NotEqualOffset, or PreferValue value")
	}
}

func nonNullSlice[T any](slice []T) []T {
	if slice == nil {
		return []T{}
	}
	return slice
}
