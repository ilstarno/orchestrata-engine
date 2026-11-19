namespace Orchestrata;

public sealed record Problem(IReadOnlyList<Variable> Variables, IReadOnlyList<Constraint> Constraints);
