from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Precedence:
    """Require the after start to be at least min_gap units after before."""

    before: int
    after: int
    min_gap: int
