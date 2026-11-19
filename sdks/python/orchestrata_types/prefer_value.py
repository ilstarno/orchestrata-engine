from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class PreferValue:
    variable: int
    value: int
    soft_weight: int
