import os
import unittest

from orchestrata import SolveError, Solver


class SolverTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.solver = Solver(os.environ["ORCHESTRATA_LIBRARY"])

    def test_optimal_solution_preserves_large_integers(self):
        result = self.solver.solve(
            {
                "variables": [{"name": "x", "domain": [2**63 - 1]}],
                "constraints": [],
            },
            1,
        )
        self.assertEqual(result["values"], [2**63 - 1])

    def test_infeasible_solution_is_none(self):
        problem = {
            "variables": [{"name": "x", "domain": [0]}],
            "constraints": [
                {"kind": "not_equal_offset", "left": 0, "right": 0, "offset": 0, "hard_weight": 1}
            ],
        }
        self.assertIsNone(self.solver.solve(problem, 1))

    def test_bad_json_and_limit_have_distinct_errors(self):
        for problem, limit, expected in (
            (b"{", 1, 1),
            (b'{"variables":[],"constraints":[]}', 0, 2),
            (b"", 1, 4),
        ):
            with self.subTest(expected=expected), self.assertRaises(SolveError) as cm:
                self.solver.solve_json(problem, limit)
            self.assertEqual(cm.exception.status, expected)

    def test_python_integer_must_fit_u64(self):
        for invalid in (-1, 2**64, True):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                self.solver.solve_json(b"{}", invalid)
