from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ScheduledInterval:
    """A start-variable index and duration in the model's time unit."""

    start: int
    duration: int
