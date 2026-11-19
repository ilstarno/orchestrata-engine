#nullable enable

using Orchestrata;

internal static class TypedTests
{
    public static void Run(Solver solver)
    {
        Problem wide = new(
            [new Variable("min", [long.MinValue]), new Variable("max", [long.MaxValue])],
            [new PreferValue(0, 0, long.MaxValue), new PreferValue(0, 0, 1)]);
        Solved solved = solver.Solve(wide, ulong.MaxValue) as Solved
            ?? throw new Exception("Expected an exact solution.");
        Assert(solved.Solution.Values.SequenceEqual([long.MinValue, long.MaxValue]), "Lost signed precision.");
        Assert(solved.Solution.Score == new Score(0, long.MinValue), "Lost score precision.");
        Assert(solved.Solution.Evaluated == 1UL, "Incorrect evaluation count.");
        Expect<SolveException>(() => solver.Solve(wide, 0), error => error.Kind == SolveStatus.SearchLimit);
        Problem overflow = wide with
        {
            Constraints = [new PreferValue(0, 0, long.MaxValue), new PreferValue(0, 0, 2)],
        };
        Expect<SolveException>(() => solver.Solve(overflow, 1), error => error.Kind == SolveStatus.ExecutionFailed);

        IReadOnlyList<Variable> variables =
            [new Variable("first", [0]), new Variable("next", [10]), new Variable("resource", [2])];
        Constraint[] valid =
        [
            new AllDifferent([0, 1], 1),
            new NotEqualOffset(0, 1, 0, 1),
            new PreferValue(1, 10, 1),
            new CountValue([2], 2, 1, 1, 1, ConstraintTier.Hard),
            new Precedence(0, 1, 10),
            new NoOverlap([new ScheduledInterval(0, 10), new ScheduledInterval(1, 1)]),
            new ResourceCapacity([new ResourceLoad(2, ulong.MaxValue)], [new ResourceLimit(2, ulong.MaxValue)]),
        ];
        Assert(solver.Solve(new Problem(variables, valid), 1) is Solved all
            && all.Solution.Values.SequenceEqual([0L, 10L, 2L])
            && all.Solution.Score == new Score(0, 0), "Seven-rule positive problem should solve.");

        Constraint[] conflicting =
        [
            new AllDifferent([0, 0], 1),
            new NotEqualOffset(0, 1, -10, 1),
            new CountValue([2], 3, 1, 1, 1, ConstraintTier.Hard),
            new Precedence(0, 1, 11),
            new NoOverlap([new ScheduledInterval(0, 11), new ScheduledInterval(1, 1)]),
            new ResourceCapacity([new ResourceLoad(2, ulong.MaxValue)], [new ResourceLimit(2, ulong.MaxValue - 1)]),
        ];
        foreach (Constraint rule in conflicting)
            Assert(solver.Solve(new Problem(variables, [rule]), 1) is Infeasible,
                $"Expected {rule.GetType().Name} to change feasibility.");

        Assert(solver.Solve(new Problem(variables, [new PreferValue(1, 11, 3)]), 1) is Solved preferred
            && preferred.Solution.Score.Soft == -3L, "Expected PreferValue to change the score.");

        Expect<ArgumentException>(() => solver.Solve(new Problem([new Variable("empty", [])], []), 1));
        Expect<ArgumentException>(() => solver.Solve(new Problem(variables, [new PreferValue(-1, 0, 1)]), 1));
        Expect<ArgumentException>(() => solver.Solve(
            new Problem(variables, [new CountValue([2], 2, 1, 1, 1, (ConstraintTier)99)]), 1));
        Expect<SolveException>(() => solver.Solve(new Problem(variables, [new Precedence(0, 0, 0)]), 1),
            error => error.Kind == SolveStatus.InvalidProblem);
    }

    public static void RejectMalformedResponses(string library)
    {
        using var solver = new Solver(library);
        Problem one = new([new Variable("fixed", [0])], []);
        // Duplicate root and nested fields, value outside domain, noninteger
        // amount, count above budget, null output, and oversized output.
        for (ulong budget = 1; budget <= 7; budget++)
        {
            ulong invalidBudget = budget;
            Expect<SolveResponseException>(() => solver.Solve(one, invalidBudget));
        }

        Expect<InvalidOperationException>(() => solver.SolveJson("{}", 6));
    }

    public static void Expect<T>(Action action, Func<T, bool>? predicate = null) where T : Exception
    {
        try
        {
            action();
        }
        catch (T error) when (predicate is null || predicate(error))
        {
            return;
        }
        throw new Exception($"Expected {typeof(T).Name}.");
    }

    private static void Assert(bool test, string message)
    {
        if (!test)
            throw new Exception(message);
    }
}
