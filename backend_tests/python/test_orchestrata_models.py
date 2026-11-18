import json
import os
import unittest
from unittest.mock import patch

from orchestrata import (
    AllDifferent, CountValue, NotEqualOffset, PreferValue, Problem,
    ResourceCapacity, ResourceLimit, ResourceLoad, ResultFormatError, Score,
    Solution, SolveError, Solver, Variable, decode_result_v1, encode_problem_v1,
)


class ModelTests(unittest.TestCase):
    def test_all_tags_match_abi_v1(self):
        problem = Problem(
            (Variable("first", (0, 1)), Variable("second", (0, 1))),
            (
                AllDifferent((0, 1), 1),
                NotEqualOffset(0, 1, -1, 2),
                PreferValue(0, 1, 3),
                CountValue((0, 1), 1, 1, 2, 4, "soft"),
                ResourceCapacity(
                    (ResourceLoad(0, 2), ResourceLoad(1, 3)),
                    (ResourceLimit(0, 4), ResourceLimit(1, 5)),
                ),
            ),
        )
        self.assertEqual(json.loads(encode_problem_v1(problem)), {
            "variables": [{"name": "first", "domain": [0, 1]}, {"name": "second", "domain": [0, 1]}],
            "constraints": [
                {"kind": "all_different", "variables": [0, 1], "hard_weight": 1},
                {"kind": "not_equal_offset", "left": 0, "right": 1, "offset": -1, "hard_weight": 2},
                {"kind": "prefer_value", "variable": 0, "value": 1, "soft_weight": 3},
                {"kind": "count_value", "variables": [0, 1], "value": 1, "min": 1, "max": 2,
                 "weight": 4, "tier": "soft"},
                {"kind": "resource_capacity", "items": [{"variable": 0, "load": 2},
                    {"variable": 1, "load": 3}], "limits": [{"resource": 0, "capacity": 4},
                    {"resource": 1, "capacity": 5}]},
            ],
        })

    def test_encoder_rejects_invalid_representations_before_running(self):
        for domain in ((), (True,), (1.0,), ("1",), (2**63,), (-2**63 - 1,)):
            with self.subTest(domain=domain), self.assertRaises(ValueError):
                encode_problem_v1(Problem((Variable("x", domain),)))
        variables = (Variable("x", (0,)),)
        for constraint in (
            PreferValue(0, 0, 0), PreferValue(True, 0, 1), PreferValue(1, 0, 1),
            CountValue((0,), 0, 1, 0, 1, "hard"), CountValue((0,), 0, 0, 1, 1, "medium"),
            ResourceCapacity((ResourceLoad(0, -1),), (ResourceLimit(0, 1),)),
            ResourceCapacity((ResourceLoad(0, 1),), (ResourceLimit(0, 2**64),)),
        ):
            with self.subTest(constraint=constraint), self.assertRaises(ValueError):
                encode_problem_v1(Problem(variables, (constraint,)))
        with self.assertRaises(TypeError):
            encode_problem_v1(Problem(variables, ({"kind": "prefer_value"},)))

    def test_decoder_preserves_integer_bounds_and_explicit_null(self):
        message = b'{"solution":{"values":[-9223372036854775808,9223372036854775807],' \
            b'"score":{"hard":0,"soft":-9223372036854775808},"evaluated":18446744073709551615}}'
        self.assertEqual(decode_result_v1(message),
                         Solution((-2**63, 2**63 - 1), Score(0, -2**63), 2**64 - 1))
        self.assertIsNone(decode_result_v1(b'{"solution":null}'))
        self.assertEqual(decode_result_v1(
            b'{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":1}}'
        ), Solution((), Score(0, 0), 1))

    def test_decoder_rejects_missing_duplicate_and_out_of_range_fields(self):
        possibilities = (
            b'{}', b'{"solution":null,"solution":null}', b'{"solution":null,"future":0}',
            b'{"solution":null}\xff', b'{"solution":null}{}',
            b'{"solution":{"values":[true],"score":{"hard":0,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[1.0],"score":{"hard":0,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[null],"score":{"hard":0,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[NaN],"score":{"hard":0,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[9223372036854775808],"score":{"hard":0,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[],"score":{"hard":-1,"soft":0},"evaluated":1}}',
            b'{"solution":{"values":[],"score":{"hard":0,"soft":1},"evaluated":1}}',
            b'{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":0}}',
            b'{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":18446744073709551616}}',
            b' ' * 2_097_152,
        )
        for message in possibilities:
            with self.subTest(prefix=message[:32]), self.assertRaises(ResultFormatError):
                decode_result_v1(message)


class NativeTypedTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.solver = Solver(os.environ["ORCHESTRATA_LIBRARY"])

    def test_starts_from_typed_problem_and_retains_raw_calls(self):
        problem = Problem((Variable("a", (0, 1)), Variable("b", (0, 1))), (
            CountValue((0, 1), 1, 1, 1, 1, "hard"),
            ResourceCapacity((ResourceLoad(0, 5), ResourceLoad(1, 3)),
                             (ResourceLimit(0, 3), ResourceLimit(1, 5))),
        ))
        expected = Solution((1, 0), Score(0, 0), 4)
        payload = encode_problem_v1(problem)
        self.assertEqual(self.solver.solve_problem(problem, 4), expected)
        self.assertEqual(decode_result_v1(self.solver.solve_json(payload, 4)), expected)
        self.assertEqual(self.solver.solve(json.loads(payload), 4)["values"], [1, 0])

    def test_integer_bounds_and_empty_solution(self):
        problem = Problem((Variable("a", (-2**63,)), Variable("b", (2**63 - 1,))))
        result = self.solver.solve_problem(problem, 1)
        self.assertEqual(result, Solution((-2**63, 2**63 - 1), Score(0, 0), 1))
        self.assertEqual(self.solver.solve_problem(Problem(()), 1), Solution((), Score(0, 0), 1))

    def test_budget_failure_is_not_null_and_native_validation_stays_native(self):
        infeasible = Problem((Variable("x", (0,)),), (NotEqualOffset(0, 0, 0, 1),))
        self.assertIsNone(self.solver.solve_problem(infeasible, 1))
        with self.assertRaises(SolveError) as exceeded:
            self.solver.solve_problem(infeasible, 0)
        self.assertEqual(exceeded.exception.status, 2)
        no_limit = Problem((Variable("x", (0,)),), (ResourceCapacity((ResourceLoad(0, 1),), ()),))
        with self.assertRaises(SolveError) as malformed:
            self.solver.solve_problem(no_limit, 1)
        self.assertEqual(malformed.exception.status, 1)

    def test_typed_response_must_correspond_to_request(self):
        problem = Problem((Variable("x", (1,)),))
        for values in ([], [2]):
            reply = {"solution": {"values": values, "score": {"hard": 0, "soft": 0}, "evaluated": 1}}
            with self.subTest(values=values), patch.object(
                self.solver, "solve_json", return_value=json.dumps(reply).encode()
            ), self.assertRaises(ResultFormatError):
                self.solver.solve_problem(problem, 1)

    def test_typed_response_must_not_exceed_requested_budget(self):
        problem = Problem((Variable("x", (1,)),))
        reply = b'{"solution":{"values":[1],"score":{"hard":0,"soft":0},"evaluated":2}}'
        with patch.object(self.solver, "solve_json", return_value=reply), self.assertRaisesRegex(
            ResultFormatError, "exceeds the requested evaluation budget"
        ):
            self.solver.solve_problem(problem, 1)
