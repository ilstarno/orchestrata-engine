from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class AllDifferent:
    variables: tuple[int, ...]
    hard_weight: int
