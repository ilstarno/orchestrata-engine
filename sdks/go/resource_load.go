package orchestrata

// ResourceLoad contributes Load to the resource selected by Variable. Loads
// and resource capacities use one consistent dimension and unit.
type ResourceLoad struct {
	Variable uint32 `json:"variable"`
	Load     uint64 `json:"load"`
}
