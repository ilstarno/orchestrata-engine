package orchestrata

type Solution struct {
	Values    []int64 `json:"values"`
	Score     Score   `json:"score"`
	Evaluated uint64  `json:"evaluated"`
}
