from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ResourceLimit:
    resource: int
    capacity: int
