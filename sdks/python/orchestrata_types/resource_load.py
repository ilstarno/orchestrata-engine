from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ResourceLoad:
    variable: int
    load: int
