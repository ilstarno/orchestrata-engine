namespace Orchestrata;

public sealed record Variable(string Name, IReadOnlyList<long> Domain);
