package orchestrata

// ResourceLimit states the inclusive Capacity for one signed Resource ID.
type ResourceLimit struct {
	Resource int64  `json:"resource"`
	Capacity uint64 `json:"capacity"`
}
