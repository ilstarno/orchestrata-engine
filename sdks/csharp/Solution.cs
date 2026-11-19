namespace Orchestrata;

public sealed record Solution(IReadOnlyList<long> Values, Score Score, ulong Evaluated);
