from dataclasses import dataclass
from .variable import Variable
from .constraint import Constraint


@dataclass(frozen=True, slots=True)
class Problem:
    variables: tuple[Variable, ...]
    constraints: tuple[Constraint, ...] = ()
