from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class NotEqualOffset:
    left: int
    right: int
    offset: int
    hard_weight: int
