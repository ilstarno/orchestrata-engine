from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Variable:
    name: str
    domain: tuple[int, ...]
