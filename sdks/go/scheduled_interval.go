package orchestrata

// ScheduledInterval is a half-open booking. Start is a zero-based variable
// index; Duration uses the same unit as its domain. A zero duration is empty.
type ScheduledInterval struct {
	Start    uint32 `json:"start"`
	Duration uint64 `json:"duration"`
}
