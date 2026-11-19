package orchestrata

// CountValueTier selects which score component is penalized by CountValue.
type CountValueTier string

const (
	CountValueHard CountValueTier = "hard"
	CountValueSoft CountValueTier = "soft"
)
