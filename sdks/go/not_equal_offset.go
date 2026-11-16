package orchestrata

// NotEqualOffset penalizes an assignment in which left minus right equals Offset.
// HardWeight must be positive.
type NotEqualOffset struct {
	Left       uint32 `json:"left"`
	Right      uint32 `json:"right"`
	Offset     int64  `json:"offset"`
	HardWeight int64  `json:"hard_weight"`
}

func (NotEqualOffset) finiteDomainConstraint() {}
