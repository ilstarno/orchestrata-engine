package orchestrata

// Constraint is an Orchestrata finite-domain rule: AllDifferent, CountValue,
// NoOverlap, NotEqualOffset, Precedence, PreferValue, or ResourceCapacity.
// Pass a value (not a pointer) to a typed problem.
type Constraint interface {
	finiteDomainConstraint()
}
