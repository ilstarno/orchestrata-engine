package orchestrata

type Status int32

const (
	InvalidProblem  Status = 1
	SearchLimit     Status = 2
	InternalError   Status = 3
	InvalidArgument Status = 4
)
