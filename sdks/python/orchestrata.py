"""Synchronous bindings for the Orchestrata standalone solver ABI v1."""

import ctypes
import json
from os import PathLike
from pathlib import Path
from typing import Any, Mapping

ABI_VERSION = 1
MAX_EVALUATIONS = 2**64 - 1


class SolveError(RuntimeError):
    """The native solver rejected the call; status is the stable C status code."""

    def __init__(self, status: int):
        self.status = status
        message = {
            1: "invalid problem",
            2: "search limit exceeded",
            3: "solver execution failed",
            4: "invalid native argument",
        }.get(status, "unknown native error")
        super().__init__(message)


class Solver:
    def __init__(self, library: str | PathLike[str]):
        self._library = ctypes.CDLL(str(Path(library).resolve(strict=True)))
        self._library.orchestrata_abi_version.argtypes = []
        self._library.orchestrata_abi_version.restype = ctypes.c_uint32
        actual = self._library.orchestrata_abi_version()
        if actual != ABI_VERSION:
            raise RuntimeError(f"unsupported Orchestrata ABI version: {actual}")

        self._library.orchestrata_solve_json_v1.argtypes = [
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_void_p),
        ]
        self._library.orchestrata_solve_json_v1.restype = ctypes.c_int32
        self._library.orchestrata_free_json_v1.argtypes = [ctypes.c_void_p]
        self._library.orchestrata_free_json_v1.restype = None

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
