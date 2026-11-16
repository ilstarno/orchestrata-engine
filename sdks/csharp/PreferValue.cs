namespace Orchestrata;

public sealed record PreferValue(int Variable, long Value, long SoftWeight) : Constraint;
