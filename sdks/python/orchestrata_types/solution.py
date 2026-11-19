from dataclasses import dataclass
from .score import Score


@dataclass(frozen=True, slots=True)
class Solution:
    values: tuple[int, ...]
    score: Score
    evaluated: int
