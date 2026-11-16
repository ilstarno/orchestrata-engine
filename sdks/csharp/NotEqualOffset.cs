namespace Orchestrata;

public sealed record NotEqualOffset(int Left, int Right, long Offset, long HardWeight) : Constraint;
