from dataclasses import dataclass
from .scheduled_interval import ScheduledInterval


@dataclass(frozen=True, slots=True)
class NoOverlap:
    """Share one exclusive resource among half-open fixed-duration intervals."""

    intervals: tuple[ScheduledInterval, ...]
