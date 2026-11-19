import itertools
import json
import os
import unittest

from orchestrata import (
    NoOverlap, Precedence, PreferValue, Problem, ScheduledInterval,
    SolveError, Solver, Variable, encode_problem_v1,
)


class SchedulingModelsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.solver = Solver(os.environ["ORCHESTRATA_LIBRARY"])

    def test_all_schedule_variants_match_independent_exact_oracle(self):
        for duration in range(1, 6):
            for first in range(1, 5):
                for second in range(1, 5):
                    durations = (duration, 1, 1)
                    weights = (first, second, 3)
                    problem = Problem(
                        tuple(Variable(name, tuple(range(5))) for name in ("cut", "inspect", "pack")),
                        (NoOverlap(tuple(ScheduledInterval(i, length) for i, length in enumerate(durations))),
                         Precedence(0, 2, 2),
                         *(PreferValue(i, value, weight) for i, (value, weight) in enumerate(zip((0, 0, 2), weights)))),
                    )
                    best = None
                    for values in itertools.product(range(5), repeat=3):
                        if values[2] < values[0] + 2:
                            continue
                        if any(values[i] < values[j] + durations[j] and values[j] < values[i] + durations[i]
                               for i in range(3) for j in range(i + 1, 3)):
                            continue
                        cost = sum(weight for value, target, weight in zip(values, (0, 0, 2), weights) if value != target)
                        if best is None or cost < best[1]:
                            best = (values, cost)
                    result = self.solver.solve_problem(problem, 125)
                    actual = None if result is None else (result.values, -result.score.soft)
                    self.assertEqual(actual, best)
                    raw = json.loads(self.solver.solve_json(encode_problem_v1(problem), 125))["solution"]
                    self.assertEqual(None if raw is None else tuple(raw["values"]), None if result is None else result.values)

    def test_rejects_wrong_types_references_and_bounds_before_native(self):
        variables = (Variable("x", (0,)), Variable("y", (1,)))
        for constraint in (
            Precedence(0, 0, 1), Precedence(0, 2, 1), Precedence(True, 1, 1),
            Precedence(0, 1, -1), Precedence(0, 1, 2**64), Precedence(0, 1, 1.0),
            NoOverlap((ScheduledInterval(0, 1),)),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(0, 2))),
            NoOverlap((ScheduledInterval(0, -1), ScheduledInterval(1, 2))),
            NoOverlap((ScheduledInterval(0, True), ScheduledInterval(1, 2))),
            NoOverlap((ScheduledInterval(0, 2**64), ScheduledInterval(1, 2))),
            NoOverlap((ScheduledInterval(0, 1), ScheduledInterval(2, 2))),
        ):
            with self.subTest(constraint=constraint), self.assertRaises(ValueError):
                encode_problem_v1(Problem(variables, (constraint,)))
        with self.assertRaises(TypeError):
            encode_problem_v1(Problem(variables, (NoOverlap(({}, {})),)))

    def test_boundary_integer_values_and_search_error_remain_lossless(self):
        problem = Problem((Variable("left", (-2**63,)), Variable("right", (2**63-1,))),
                          (Precedence(0, 1, 2**64-1),
                           NoOverlap((ScheduledInterval(0, 2**64-1), ScheduledInterval(1, 0)))))
        result = self.solver.solve_problem(problem, 1)
        self.assertEqual(result.values, (-2**63, 2**63-1))
        with self.assertRaises(SolveError) as caught:
            self.solver.solve_problem(problem, 0)
        self.assertEqual(caught.exception.status, 2)

    def test_public_imports_and_pickled_values_remain_compatible(self):
        import pickle
        import orchestrata_models
        import orchestrata
        for name in ("Variable", "Problem", "AllDifferent", "NotEqualOffset", "PreferValue", "CountValue", "ResourceCapacity", "ResourceLoad", "ResourceLimit", "Score", "Solution", "ResultFormatError"):
            self.assertIs(getattr(orchestrata, name), getattr(orchestrata_models, name))
        variable = Variable("x", (1,))
        self.assertEqual(pickle.loads(pickle.dumps(variable)), variable)
