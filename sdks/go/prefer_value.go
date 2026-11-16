package orchestrata

// PreferValue penalizes an indexed variable when it differs from Value.
// SoftWeight must be positive.
type PreferValue struct {
	Variable   uint32 `json:"variable"`
	Value      int64  `json:"value"`
	SoftWeight int64  `json:"soft_weight"`
}

func (PreferValue) finiteDomainConstraint() {}
