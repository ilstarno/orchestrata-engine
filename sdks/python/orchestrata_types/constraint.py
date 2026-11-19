from typing import TypeAlias
from .all_different import AllDifferent
from .not_equal_offset import NotEqualOffset
from .prefer_value import PreferValue
from .count_value import CountValue
from .resource_capacity import ResourceCapacity
from .precedence import Precedence
from .no_overlap import NoOverlap

Constraint: TypeAlias = AllDifferent | NotEqualOffset | PreferValue | CountValue | ResourceCapacity | Precedence | NoOverlap
