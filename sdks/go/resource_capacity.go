package orchestrata

// ResourceCapacity limits the sum of assigned loads in one dimension.
// Every resource in an item variable's domain needs exactly one limit.
type ResourceCapacity struct {
	Items  []ResourceLoad  `json:"items"`
	Limits []ResourceLimit `json:"limits"`
}

func (ResourceCapacity) finiteDomainConstraint() {}
