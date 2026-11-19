namespace Orchestrata;

/// <summary>The native response did not match the exact-solve V1 result contract.</summary>
public sealed class SolveResponseException : Exception
{
    internal SolveResponseException() : base("The native solver returned an invalid V1 response.") { }
}
