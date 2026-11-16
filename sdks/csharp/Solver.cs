using System.Runtime.InteropServices;
using System.Text;

namespace Orchestrata;

/// <summary>A binding to one native Orchestrata v1 library.</summary>
public sealed class Solver : IDisposable
{
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
    public unsafe string SolveJson(ReadOnlySpan<byte> problem, ulong maxEvaluations)
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

                return Marshal.PtrToStringUTF8(result)
                    ?? throw new InvalidOperationException("Native solver returned no response.");
            }
            finally
            {
                free(result);
            }
        }
    }

    public string SolveJson(string problem, ulong maxEvaluations) =>
        SolveJson(Encoding.UTF8.GetBytes(problem), maxEvaluations);

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
