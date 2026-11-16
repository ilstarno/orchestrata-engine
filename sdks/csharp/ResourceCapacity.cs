namespace Orchestrata;

public sealed record ResourceCapacity(IReadOnlyList<ResourceLoad> Items, IReadOnlyList<ResourceLimit> Limits) : Constraint;
