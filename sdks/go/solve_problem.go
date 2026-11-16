package orchestrata

import "fmt"

// SolveProblem sends a typed finite-domain problem through the existing ABI
// v1. A nil solution means exhaustive search proved infeasibility.
func SolveProblem(problem Problem, maxEvaluations uint64) (*Solution, error) {
	request, err := EncodeProblemV1(problem)
	if err != nil {
		return nil, fmt.Errorf("encode Orchestrata request: %w", err)
	}
	response, err := SolveJSON(request, maxEvaluations)
	if err != nil {
		return nil, err
	}
	solution, err := DecodeResultV1(response)
	if err != nil {
		return nil, fmt.Errorf("decode Orchestrata response: %w", err)
	}
	if solution != nil && len(solution.Values) != len(problem.Variables) {
		return nil, fmt.Errorf("Orchestrata response value count does not match request")
	}
	return solution, nil
}
