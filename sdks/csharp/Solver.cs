using System.Runtime.InteropServices;
using System.Text;

namespace Orchestrata;

/// <summary>A binding to one native Orchestrata v1 library.</summary>
public sealed class Solver : IDisposable
{
    private const int MaxTypedResponseBytes = 2 * 1024 * 1024;
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
    private readonly object gate = new();
    private readonly SolveNative solve;
    private readonly FreeNative free;
    private nint library;

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint VersionNative();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private unsafe delegate int SolveNative(byte* request, nuint length, ulong maxEvaluations, out nint result);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void FreeNative(nint result);

    public unsafe Solver(string libraryPath)
    {
        library = NativeLibrary.Load(Path.GetFullPath(libraryPath));
        try
        {
            uint version = Load<VersionNative>("orchestrata_abi_version")();
            if (version != 1)
                throw new NotSupportedException($"Unsupported Orchestrata ABI version: {version}.");

            solve = Load<SolveNative>("orchestrata_solve_json_v1");
            free = Load<FreeNative>("orchestrata_free_json_v1");
        }
        catch
        {
            NativeLibrary.Free(library);
            library = 0;
            throw;
        }
    }

    /// <summary>Returns the complete UTF-8 solver response as a managed string.</summary>
    public string SolveJson(ReadOnlySpan<byte> problem, ulong maxEvaluations) =>
        InvokeNative(problem, maxEvaluations)
            ?? throw new InvalidOperationException("Native solver returned no response.");

    private unsafe string? InvokeNative(ReadOnlySpan<byte> problem, ulong maxEvaluations, bool typed = false)
    {
        lock (gate)
        {
            ObjectDisposedException.ThrowIf(library == 0, this);
            nint result = 0;
            try
            {
                fixed (byte* request = problem)
                {
                    int status = solve(request, (nuint)problem.Length, maxEvaluations, out result);
                    if (status != 0)
                        throw new SolveException(status);
                }

                return typed ? ReadTypedResponse(result) : Marshal.PtrToStringUTF8(result);
            }
            finally
            {
                free(result);
            }
        }
    }

    public string SolveJson(string problem, ulong maxEvaluations) =>
        SolveJson(Encoding.UTF8.GetBytes(problem), maxEvaluations);

    /// <summary>Runs exact search and validates the native response without losing integer precision.</summary>
    /// <exception cref="ArgumentException">The managed request has an invalid shape.</exception>
    /// <exception cref="SolveException">The native engine rejected or could not solve the request.</exception>
    /// <exception cref="SolveResponseException">The native response could not be validated.</exception>
    public SolveOutcome Solve(Problem problem, ulong maxEvaluations)
    {
        PreparedProblem request = ProblemEncoder.Encode(problem);
        string response = InvokeNative(request.Utf8, maxEvaluations, typed: true) ?? throw new SolveResponseException();
        return ResultDecoder.Decode(response, request.Domains, maxEvaluations);
    }

    private static unsafe string ReadTypedResponse(nint result)
    {
        if (result == 0)
            throw new SolveResponseException();

        // The trusted V1 library owns a NUL-terminated buffer until free.
        byte* response = (byte*)result;
        for (int length = 0; length < MaxTypedResponseBytes; length++)
        {
            if (response[length] != 0)
                continue;

            try
            {
                return StrictUtf8.GetString(new ReadOnlySpan<byte>(response, length));
            }
            catch (DecoderFallbackException)
            {
                throw new SolveResponseException();
            }
        }

        throw new SolveResponseException();
    }

    public void Dispose()
    {
        lock (gate)
        {
            if (library == 0)
                return;

            NativeLibrary.Free(library);
            library = 0;
        }
    }

    private T Load<T>(string symbol) where T : Delegate =>
        Marshal.GetDelegateForFunctionPointer<T>(NativeLibrary.GetExport(library, symbol));
}
