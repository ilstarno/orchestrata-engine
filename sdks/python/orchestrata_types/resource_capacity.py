from dataclasses import dataclass
from .resource_load import ResourceLoad
from .resource_limit import ResourceLimit


@dataclass(frozen=True, slots=True)
class ResourceCapacity:
    items: tuple[ResourceLoad, ...]
    limits: tuple[ResourceLimit, ...]
