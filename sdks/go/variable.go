package orchestrata

// Variable selects one signed 64-bit value in a nonempty ordered domain.
type Variable struct {
	Name   string  `json:"name"`
	Domain []int64 `json:"domain"`
}
