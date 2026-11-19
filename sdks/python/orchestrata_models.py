"""Typed problem and result values for the Orchestrata ABI v1 JSON contract."""

import json
from orchestrata_types.variable import Variable
from orchestrata_types.all_different import AllDifferent
from orchestrata_types.not_equal_offset import NotEqualOffset
from orchestrata_types.prefer_value import PreferValue
from orchestrata_types.count_value import CountValue
from orchestrata_types.resource_load import ResourceLoad
from orchestrata_types.resource_limit import ResourceLimit
from orchestrata_types.resource_capacity import ResourceCapacity
from orchestrata_types.precedence import Precedence
from orchestrata_types.scheduled_interval import ScheduledInterval
from orchestrata_types.no_overlap import NoOverlap
from orchestrata_types.problem import Problem
from orchestrata_types.score import Score
from orchestrata_types.solution import Solution
from orchestrata_types.result_format_error import ResultFormatError
from orchestrata_types.constraint import Constraint
from typing import cast

_I64_MIN = -(2 ** 63)
_I64_MAX = 2 ** 63 - 1
_U64_MAX = 2 ** 64 - 1
_MAX_RESULT_BYTES = 2_097_152


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


def _encode_no_overlap(value: NoOverlap, count: int) -> dict[str, object]:
    intervals = _sequence(value.intervals, "intervals")
    if len(intervals) < 2:
        raise ValueError("no_overlap requires at least two intervals")
    seen: set[int] = set()
    encoded: list[dict[str, int]] = []
    for interval in intervals:
        if not isinstance(interval, ScheduledInterval):
            raise TypeError("intervals must contain ScheduledInterval values")
        start = _index(interval.start, count)
        if start in seen:
            raise ValueError("no_overlap start variables must be distinct")
        seen.add(start)
        encoded.append({
            "start": start,
            "duration": _integer(interval.duration, 0, _U64_MAX, "duration"),
        })
    return {"kind": "no_overlap", "intervals": encoded}


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
    if isinstance(value, Precedence):
        before = _index(value.before, count)
        after = _index(value.after, count)
        if before == after:
            raise ValueError("precedence before and after variables must be distinct")
        return {
            "kind": "precedence",
            "before": before,
            "after": after,
            "min_gap": _integer(value.min_gap, 0, _U64_MAX, "min_gap"),
        }
    if isinstance(value, NoOverlap):
        return _encode_no_overlap(value, count)
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
