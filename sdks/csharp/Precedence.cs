namespace Orchestrata;

public sealed record Precedence(int Before, int After, ulong MinGap) : Constraint;
