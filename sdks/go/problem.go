package orchestrata

// Problem describes a finite-domain solver request. Constraint indices address
// Variables by position. No JSON representation is required by SolveProblem.
type Problem struct {
	Variables   []Variable
	Constraints []Constraint
}
