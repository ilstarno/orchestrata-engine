package orchestrata

import "fmt"

type SolveError struct {
	Status Status
}

func (err *SolveError) Error() string {
	switch err.Status {
	case InvalidProblem:
		return "invalid problem"
	case SearchLimit:
		return "search limit exceeded"
	case InternalError:
		return "solver execution failed"
	case InvalidArgument:
		return "invalid native argument"
	default:
		return fmt.Sprintf("unknown native status %d", err.Status)
	}
}
