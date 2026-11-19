class SolveError(RuntimeError):
    """The native solver rejected the call; status is the stable C status code."""

    def __init__(self, status: int):
        self.status = status
        message = {
            1: "invalid problem",
            2: "search limit exceeded",
            3: "solver execution failed",
            4: "invalid native argument",
        }.get(status, "unknown native error")
        super().__init__(message)
