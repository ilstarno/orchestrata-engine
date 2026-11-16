namespace Orchestrata;

public sealed record NoOverlap(IReadOnlyList<ScheduledInterval> Intervals) : Constraint;
