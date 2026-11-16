// Package orchestrata provides cgo bindings to the Orchestrata solver ABI v1.
package orchestrata

/*
#cgo LDFLAGS: -lorchestrata_ffi
#include "orchestrata.h"
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

const ABIVersion uint32 = 1

type Status int32

const (
	InvalidProblem  Status = 1
	SearchLimit     Status = 2
	InternalError   Status = 3
	InvalidArgument Status = 4
)

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

type Score struct {
	Hard int64 `json:"hard"`
	Soft int64 `json:"soft"`
}

type Solution struct {
	Values    []int64 `json:"values"`
	Score     Score   `json:"score"`
	Evaluated uint64  `json:"evaluated"`
}

// SolveJSON sends a UTF-8 JSON problem and returns a copied UTF-8 result document.
func SolveJSON(problem []byte, maxEvaluations uint64) ([]byte, error) {
	if current := uint32(C.orchestrata_abi_version()); current != ABIVersion {
		return nil, fmt.Errorf("unsupported Orchestrata ABI version: %d", current)
	}
	if len(problem) == 0 || len(problem) > int(C.ORCHESTRATA_MAX_REQUEST_BYTES) {
		return nil, &SolveError{Status: InvalidArgument}
	}

	// The native call does not retain the request, and it contains no Go pointers.
	request := (*C.uint8_t)(unsafe.Pointer(&problem[0]))
	var result *C.char
	status := C.orchestrata_solve_json_v1(request, C.size_t(len(problem)), C.uint64_t(maxEvaluations), &result)
	defer C.orchestrata_free_json_v1(result)
	if status != 0 {
		return nil, &SolveError{Status: Status(status)}
	}
	if result == nil {
		return nil, fmt.Errorf("native solver returned no response")
	}
	return []byte(C.GoString(result)), nil
}

// Solve returns an optimal solution or nil if exhaustive search proved infeasibility.
func Solve(problem []byte, maxEvaluations uint64) (*Solution, error) {
	response, err := SolveJSON(problem, maxEvaluations)
	if err != nil {
		return nil, err
	}
	var document struct {
		Solution *Solution `json:"solution"`
	}
	if err := json.Unmarshal(response, &document); err != nil {
		return nil, fmt.Errorf("decode Orchestrata response: %w", err)
	}
	return document.Solution, nil
}
