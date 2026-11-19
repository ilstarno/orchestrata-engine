"""Synchronous bindings for the Orchestrata standalone solver ABI v1."""

import ctypes
import json
from os import PathLike
from pathlib import Path
from typing import Any, Mapping

from orchestrata_models import (
    AllDifferent as AllDifferent,
    Constraint as Constraint,
    CountValue as CountValue,
    Precedence as Precedence,
    ScheduledInterval as ScheduledInterval,
    NoOverlap as NoOverlap,
    NotEqualOffset as NotEqualOffset,
    PreferValue as PreferValue,
    Problem as Problem,
    ResourceCapacity as ResourceCapacity,
    ResourceLimit as ResourceLimit,
    ResourceLoad as ResourceLoad,
    ResultFormatError as ResultFormatError,
    Score as Score,
    Solution as Solution,
    Variable as Variable,
    decode_result_v1 as decode_result_v1,
    encode_problem_v1 as encode_problem_v1,
)

from orchestrata_types.solve_error import SolveError as SolveError

ABI_VERSION = 1
MAX_EVALUATIONS = 2**64 - 1


class Solver:
    def __init__(self, library: str | PathLike[str]):
        self._library = ctypes.CDLL(str(Path(library).resolve(strict=True)))
        abi_version = self._require_export("orchestrata_abi_version")
        abi_version.argtypes = []
        abi_version.restype = ctypes.c_uint32
        actual = abi_version()
        if actual != ABI_VERSION:
            raise RuntimeError(f"unsupported Orchestrata ABI version: {actual}")

        solve_json = self._require_export("orchestrata_solve_json_v1")
        free_json = self._require_export("orchestrata_free_json_v1")
        solve_json.argtypes = [
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_void_p),
        ]
        solve_json.restype = ctypes.c_int32
        free_json.argtypes = [ctypes.c_void_p]
        free_json.restype = None

    def _require_export(self, name: str) -> ctypes._CFuncPtr:
        try:
            return getattr(self._library, name)
        except AttributeError as exc:
            raise RuntimeError(
                f"not a compatible Orchestrata native library: missing {name}"
            ) from exc

    def solve_json(self, problem: bytes, max_evaluations: int) -> bytes:
        """Return the UTF-8 result document, freeing native storage before returning."""
        if type(max_evaluations) is not int or not 0 <= max_evaluations <= MAX_EVALUATIONS:
            raise ValueError("max_evaluations must be an unsigned 64-bit integer")
        if not isinstance(problem, bytes):
            raise TypeError("problem must be UTF-8 JSON bytes")

        result = ctypes.c_void_p()
        status = self._library.orchestrata_solve_json_v1(
            problem, len(problem), max_evaluations, ctypes.byref(result)
        )
        try:
            if status:
                raise SolveError(status)
            if not result.value:
                raise RuntimeError("native solver returned no response")
            return ctypes.string_at(result)
        finally:
            self._library.orchestrata_free_json_v1(result)

    def solve(
        self, problem: Mapping[str, Any], max_evaluations: int
    ) -> dict[str, Any] | None:
        """Return an optimal solution or None if exhaustive search proved infeasibility."""
        encoded = json.dumps(dict(problem), allow_nan=False).encode("utf-8")
        return json.loads(self.solve_json(encoded, max_evaluations))["solution"]

    def solve_problem(self, problem: Problem, max_evaluations: int) -> Solution | None:
        """Solve an immutable typed model without changing the ABI or raw APIs."""
        result = decode_result_v1(self.solve_json(encode_problem_v1(problem), max_evaluations))
        if result is not None:
            if result.evaluated > max_evaluations:
                raise ResultFormatError("result exceeds the requested evaluation budget")
            if len(result.values) != len(problem.variables):
                raise ResultFormatError("result value count does not match the problem")
            if any(value not in variable.domain for variable, value in zip(problem.variables, result.values)):
                raise ResultFormatError("result value is outside its variable domain")
        return result
