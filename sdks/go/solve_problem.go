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
	return DecodeResultForProblemV1(response, problem)
}

// DecodeResultForProblemV1 also checks that each verified result value belongs
// to the corresponding request domain. Only an explicit null means infeasible.
func DecodeResultForProblemV1(response []byte, problem Problem) (*Solution, error) {
	solution, err := DecodeResultV1(response)
	if err != nil {
		return nil, fmt.Errorf("decode Orchestrata response: %w", err)
	}
	if solution == nil {
		return nil, nil
	}
	if len(solution.Values) != len(problem.Variables) {
		return nil, fmt.Errorf("Orchestrata response value count does not match request")
	}
	for index, value := range solution.Values {
		found := false
		for _, candidate := range problem.Variables[index].Domain {
			if candidate == value {
				found = true
				break
			}
		}
		if !found {
			return nil, fmt.Errorf("Orchestrata response value at index %d is outside its request domain", index)
		}
	}
	return solution, nil
}
