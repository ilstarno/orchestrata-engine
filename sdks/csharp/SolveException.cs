namespace Orchestrata;

public sealed class SolveException : Exception
{
    public int Status { get; }

    /// <summary>A typed form of the native status; Status retains the numeric value.</summary>
    public SolveStatus Kind => Status switch
    {
        1 => SolveStatus.InvalidProblem,
        2 => SolveStatus.SearchLimit,
        3 => SolveStatus.ExecutionFailed,
        4 => SolveStatus.InvalidArgument,
        _ => SolveStatus.Unknown,
    };

    internal SolveException(int status) : base(status switch
    {
        1 => "Invalid problem.",
        2 => "Search limit exceeded.",
        3 => "Solver execution failed.",
        4 => "Invalid native argument.",
        _ => $"Unknown native status: {status}."
    })
    {
        Status = status;
    }
}
