package orchestrata

// AllDifferent penalizes equal values among the indexed variables.
// HardWeight must be positive.
type AllDifferent struct {
	Variables  []uint32 `json:"variables"`
	HardWeight int64    `json:"hard_weight"`
}

func (AllDifferent) finiteDomainConstraint() {}
