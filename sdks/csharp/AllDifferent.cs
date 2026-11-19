namespace Orchestrata;

public sealed record AllDifferent(IReadOnlyList<int> Variables, long HardWeight) : Constraint;
