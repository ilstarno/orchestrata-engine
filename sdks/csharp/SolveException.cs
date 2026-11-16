namespace Orchestrata;

public sealed class SolveException : Exception
{
    public int Status { get; }

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
