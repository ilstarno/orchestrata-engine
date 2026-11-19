from dataclasses import dataclass
from typing import Literal


@dataclass(frozen=True, slots=True)
class CountValue:
    variables: tuple[int, ...]
    value: int
    min: int
    max: int
    weight: int
    tier: Literal["hard", "soft"]
