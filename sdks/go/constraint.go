package orchestrata

// Constraint is one of the built-in finite-domain constraint values:
// AllDifferent, NotEqualOffset, or PreferValue.
type Constraint interface {
	finiteDomainConstraint()
}
