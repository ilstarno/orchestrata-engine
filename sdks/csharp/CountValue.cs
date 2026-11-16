namespace Orchestrata;

public sealed record CountValue(IReadOnlyList<int> Variables, long Value, int Min, int Max, long Weight, ConstraintTier Tier) : Constraint;
