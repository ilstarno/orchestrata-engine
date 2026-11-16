package orchestrata

// CountValue counts the variables assigned Value. Min and Max are inclusive
// bounds. Variable references must be distinct, and Weight must be positive.
type CountValue struct {
	Variables []uint32       `json:"variables"`
	Value     int64          `json:"value"`
	Min       uint32         `json:"min"`
	Max       uint32         `json:"max"`
	Weight    int64          `json:"weight"`
	Tier      CountValueTier `json:"tier"`
}

func (CountValue) finiteDomainConstraint() {}
