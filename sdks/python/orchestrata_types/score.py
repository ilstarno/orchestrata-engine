from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Score:
    hard: int
    soft: int
