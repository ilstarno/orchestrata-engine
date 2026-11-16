"""Typed problem and result values for the Orchestrata ABI v1 JSON contract."""

from dataclasses import dataclass
import json
from typing import Literal, TypeAlias, cast

_I64_MIN = -(2 ** 63)
_I64_MAX = 2 ** 63 - 1
_U64_MAX = 2 ** 64 - 1
_MAX_RESULT_BYTES = 2_097_152


@dataclass(frozen=True, slots=True)
class Variable:
    name: str
    domain: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class AllDifferent:
    variables: tuple[int, ...]
    hard_weight: int


@dataclass(frozen=True, slots=True)
class NotEqualOffset:
    left: int
    right: int
    offset: int
    hard_weight: int


@dataclass(frozen=True, slots=True)
class PreferValue:
    variable: int
    value: int
    soft_weight: int


@dataclass(frozen=True, slots=True)
class CountValue:
    variables: tuple[int, ...]
    value: int
    min: int
    max: int
    weight: int
    tier: Literal["hard", "soft"]


@dataclass(frozen=True, slots=True)
class ResourceLoad:
    variable: int
    load: int


@dataclass(frozen=True, slots=True)
class ResourceLimit:
    resource: int
    capacity: int


@dataclass(frozen=True, slots=True)
class ResourceCapacity:
    items: tuple[ResourceLoad, ...]
    limits: tuple[ResourceLimit, ...]


Constraint: TypeAlias = AllDifferent | NotEqualOffset | PreferValue | CountValue | ResourceCapacity


@dataclass(frozen=True, slots=True)
class Problem:
    variables: tuple[Variable, ...]
    constraints: tuple[Constraint, ...] = ()


@dataclass(frozen=True, slots=True)
class Score:
    hard: int
    soft: int


@dataclass(frozen=True, slots=True)
class Solution:
    values: tuple[int, ...]
    score: Score
    evaluated: int


class ResultFormatError(ValueError):
    """The native response was not a valid ABI v1 result."""


def _integer(value: object, low: int, high: int, field: str) -> int:
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"{field} must be an integer in [{low}, {high}]")
    return value


def _i64(value: object, field: str) -> int:
    return _integer(value, _I64_MIN, _I64_MAX, field)


def _weight(value: object, field: str) -> int:
    return _integer(value, 1, _I64_MAX, field)


def _index(value: object, count: int) -> int:
    return _integer(value, 0, count - 1, "variable index")


def _sequence(value: object, field: str) -> tuple[object, ...]:
    if not isinstance(value, tuple):
        raise TypeError(f"{field} must be a tuple")
    return value


def _encode_variable(value: object) -> dict[str, object]:
    if not isinstance(value, Variable) or not isinstance(value.name, str):
        raise TypeError("expected Variable with a string name")
    domain = [_i64(item, "domain value") for item in _sequence(value.domain, "domain")]
    if not domain:
        raise ValueError("domain must contain at least one value")
    return {"name": value.name, "domain": domain}


def _encode_resource_capacity(value: ResourceCapacity, count: int) -> dict[str, object]:
    items: list[dict[str, int]] = []
    for item in _sequence(value.items, "items"):
        if not isinstance(item, ResourceLoad):
            raise TypeError("resource items must be ResourceLoad values")
        items.append({
            "variable": _index(item.variable, count),
            "load": _integer(item.load, 0, _U64_MAX, "load"),
        })
    limits: list[dict[str, int]] = []
    for limit in _sequence(value.limits, "limits"):
        if not isinstance(limit, ResourceLimit):
            raise TypeError("resource limits must be ResourceLimit values")
        limits.append({
            "resource": _i64(limit.resource, "resource"),
            "capacity": _integer(limit.capacity, 0, _U64_MAX, "capacity"),
        })
    return {"kind": "resource_capacity", "items": items, "limits": limits}


def _encode_constraint(value: object, count: int) -> dict[str, object]:
    if isinstance(value, AllDifferent):
        return {
            "kind": "all_different",
            "variables": [_index(i, count) for i in _sequence(value.variables, "variables")],
            "hard_weight": _weight(value.hard_weight, "hard_weight"),
        }
    if isinstance(value, NotEqualOffset):
        return {
            "kind": "not_equal_offset",
            "left": _index(value.left, count),
            "right": _index(value.right, count),
            "offset": _i64(value.offset, "offset"),
            "hard_weight": _weight(value.hard_weight, "hard_weight"),
        }
    if isinstance(value, PreferValue):
        return {
            "kind": "prefer_value",
            "variable": _index(value.variable, count),
            "value": _i64(value.value, "value"),
            "soft_weight": _weight(value.soft_weight, "soft_weight"),
        }
    if isinstance(value, CountValue):
        scope = [_index(i, count) for i in _sequence(value.variables, "variables")]
        lower = _integer(value.min, 0, len(scope), "min")
        upper = _integer(value.max, lower, len(scope), "max")
        if value.tier not in ("hard", "soft"):
            raise ValueError("tier must be hard or soft")
        return {
            "kind": "count_value",
            "variables": scope,
            "value": _i64(value.value, "value"),
            "min": lower,
            "max": upper,
            "weight": _weight(value.weight, "weight"),
            "tier": value.tier,
        }
    if isinstance(value, ResourceCapacity):
        return _encode_resource_capacity(value, count)
    raise TypeError("unsupported constraint value")


def encode_problem_v1(problem: Problem) -> bytes:
    """Encode a typed request; solver-level feasibility checks stay native."""
    if not isinstance(problem, Problem):
        raise TypeError("problem must be a Problem")
    variables = [_encode_variable(v) for v in _sequence(problem.variables, "variables")]
    constraints = [
        _encode_constraint(c, len(variables))
        for c in _sequence(problem.constraints, "constraints")
    ]
    return json.dumps(
        {"variables": variables, "constraints": constraints},
        separators=(",", ":"), allow_nan=False,
    ).encode("utf-8")


def _unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    output: dict[str, object] = {}
    for key, value in pairs:
        if key in output:
            raise ResultFormatError("duplicate JSON field")
        output[key] = value
    return output


def _reject_constant(_: str) -> None:
    raise ResultFormatError("non-finite JSON value")


def _object(value: object, *fields: str) -> dict[str, object]:
    if not isinstance(value, dict) or value.keys() != set(fields):
        raise ResultFormatError("unexpected result object fields")
    return cast(dict[str, object], value)


def decode_result_v1(response: bytes) -> Solution | None:
    """Decode a complete, verified-result envelope without rounding integers."""
    if not isinstance(response, bytes):
        raise TypeError("response must be bytes")
    if len(response) >= _MAX_RESULT_BYTES:
        raise ResultFormatError("result exceeds the ABI v1 byte limit")
    try:
        document: object = json.loads(
            response.decode("utf-8", errors="strict"),
            object_pairs_hook=_unique_object,
            parse_constant=_reject_constant,
        )
        solution = _object(document, "solution")["solution"]
        if solution is None:
            return None
        fields = _object(solution, "values", "score", "evaluated")
        values = fields["values"]
        if not isinstance(values, list):
            raise ResultFormatError("values must be an array")
        score = _object(fields["score"], "hard", "soft")
        return Solution(
            values=tuple(_i64(v, "assignment value") for v in values),
            score=Score(
                hard=_integer(score["hard"], 0, 0, "hard score"),
                soft=_integer(score["soft"], _I64_MIN, 0, "soft score"),
            ),
            evaluated=_integer(fields["evaluated"], 1, _U64_MAX, "evaluated"),
        )
    except ResultFormatError:
        raise
    except (ValueError, RecursionError) as exc:
        raise ResultFormatError("invalid ABI v1 result") from exc
