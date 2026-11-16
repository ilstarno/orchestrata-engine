import json
import os
import unittest

from orchestrata import (
    NoOverlap,
    Precedence,
    Problem,
    ScheduledInterval,
    Score,
    Solution,
    SolveError,
    Solver,
    Variable,
    encode_problem_v1,
)


class SchedulingEncoderTests(unittest.TestCase):
    def test_exact_abi_tags_fields_and_integer_bounds(self):
        maximum = 2**64 - 1
        problem = Problem(
            (Variable("before", (-2**63,)), Variable("after", (2**63 - 1,))),
            (Precedence(0, 1, maximum),
             NoOverlap((ScheduledInterval(0, maximum), ScheduledInterval(1, 0)))),
        )
        self.assertEqual(json.loads(encode_problem_v1(problem))["constraints"], [
            {"kind": "precedence", "before": 0, "after": 1, "min_gap": maximum},
            {"kind": "no_overlap", "intervals": [
                {"start": 0, "duration": maximum}, {"start": 1, "duration": 0},
            ]},
        ])

    def test_invalid_schedule_encoder_inputs(self):
        variables = (Variable("first", (0,)), Variable("second", (2,)))
        for rule in (
            Precedence(0, 0, 0),
            Precedence(0, 2, 0),
            Precedence(-1, 1, 0),
            Precedence(False, 1, 0),
            Precedence(0, 1, -1),
            Precedence(0, 1, False),
            Precedence(0, 1, 2**64),
            NoOverlap(()),
            NoOverlap((ScheduledInterval(0, 1),)),
            NoOverlap((ScheduledInterval(0, 0), ScheduledInterval(0, 1))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(2, 1))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(False, 1))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(1, -1))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(1, True))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(1, 2**64))),
        ):
            with self.subTest(rule=rule), self.assertRaises(ValueError):
                encode_problem_v1(Problem(variables, (rule,)))
        with self.assertRaises(TypeError):
            encode_problem_v1(Problem(variables, (
                NoOverlap((ScheduledInterval(0, 1), {"start": 1, "duration": 1})),
            )))


class NativeSchedulingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.solver = Solver(os.environ["ORCHESTRATA_LIBRARY"])

    def test_touching_intervals_and_precedence_share_one_native_request(self):
        variables = (
            Variable("first_use", (0,)),
            Variable("next_use", (1, 2, 3)),
        )
        constraints = (
            Precedence(0, 1, 1),
            NoOverlap((ScheduledInterval(0, 2), ScheduledInterval(1, 1))),
        )
        problem = Problem(variables, constraints)
        self.assertEqual(self.solver.solve_problem(problem, 3),
                         Solution((0, 2), Score(0, 0), 3))
        raw = json.loads(encode_problem_v1(problem))
        self.assertEqual(self.solver.solve(raw, 3)["values"], [0, 2])
        with self.assertRaises(SolveError) as error:
            self.solver.solve_problem(problem, 2)
        self.assertEqual(error.exception.status, 2)

    def test_native_valid_unreachable_ranges_and_extreme_endpoints(self):
        first, second = Variable("a", (0,)), Variable("b", (1,))
        # The first fixed pair has no feasible start; local encoding is not solving.
        for rule in (
            Precedence(0, 1, 2),
            NoOverlap((ScheduledInterval(0, 2), ScheduledInterval(1, 1))),
        ):
            self.assertIsNone(self.solver.solve_problem(Problem((first, second), (rule,)), 1))
        lo, hi, duration = -2**63, 2**63 - 1, 2**64 - 1
        extremes = Problem((Variable("lo", (lo,)), Variable("hi", (hi,))), (
            Precedence(0, 1, duration),
            NoOverlap((ScheduledInterval(0, duration), ScheduledInterval(1, 1))),
        ))
        self.assertEqual(self.solver.solve_problem(extremes, 1),
                         Solution((lo, hi), Score(0, 0), 1))
