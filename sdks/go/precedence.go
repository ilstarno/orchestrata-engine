package orchestrata

// Precedence requires After >= Before + MinGap. Before and After are distinct
// zero-based variable indices. Domain values and MinGap use the same unit.
type Precedence struct {
	Before uint32 `json:"before"`
	After  uint32 `json:"after"`
	MinGap uint64 `json:"min_gap"`
}

func (Precedence) finiteDomainConstraint() {}
