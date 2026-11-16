package orchestrata

// NoOverlap requires at least two distinct bookings on one exclusive resource.
// An interval ending at another's start does not overlap it.
type NoOverlap struct {
	Intervals []ScheduledInterval `json:"intervals"`
}

func (NoOverlap) finiteDomainConstraint() {}
