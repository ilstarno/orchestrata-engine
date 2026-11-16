package io.orchestrata;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.Objects;

/** Synchronous bindings for the native Orchestrata ABI v1 on 64-bit Java 22+. */
public final class Solver implements AutoCloseable {
    private static final long MAX_RESULT_BYTES = 2L * 1024 * 1024;

    private final Arena lifetime;
    private final MethodHandle solve;
    private final MethodHandle free;
    private boolean closed;

    public Solver(Path library) {
        if (ValueLayout.ADDRESS.byteSize() != Long.BYTES) {
            throw new UnsupportedOperationException("Orchestrata requires a 64-bit JVM");
        }

        lifetime = Arena.ofShared();
        try {
            var symbols = SymbolLookup.libraryLookup(library.toAbsolutePath(), lifetime);
            var linker = Linker.nativeLinker();
            var version = downcall(linker, symbols, "orchestrata_abi_version",
                FunctionDescriptor.of(ValueLayout.JAVA_INT));
            int abiVersion = (int) version.invokeExact();
            if (abiVersion != 1) {
                throw new UnsupportedOperationException("Unsupported Orchestrata ABI version: " + abiVersion);
            }

            solve = downcall(linker, symbols, "orchestrata_solve_json_v1",
                FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
            free = downcall(linker, symbols, "orchestrata_free_json_v1",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        } catch (RuntimeException | Error error) {
            lifetime.close();
            throw error;
        } catch (Throwable error) {
            lifetime.close();
            throw new IllegalStateException("Cannot initialize Orchestrata bindings", error);
        }
    }

    /** The bits in maxEvaluations are passed as a native unsigned 64-bit value. */
    public synchronized String solveJson(byte[] problem, long maxEvaluations) {
        Objects.requireNonNull(problem, "problem");
        if (closed) {
            throw new IllegalStateException("Solver is closed");
        }
        if (problem.length == 0 || problem.length > 1_048_576) {
            throw new SolveException(4);
        }

        try (var call = Arena.ofConfined()) {
            var request = call.allocateFrom(ValueLayout.JAVA_BYTE, problem);
            var output = call.allocate(ValueLayout.ADDRESS);
            int status = (int) solve.invokeExact(request, (long) problem.length, maxEvaluations, output);
            var result = output.get(ValueLayout.ADDRESS, 0);
            try {
                if (status != 0) {
                    throw new SolveException(status);
                }
                if (result.equals(MemorySegment.NULL)) {
                    throw new IllegalStateException("Native solver returned no response");
                }
                // The native ABI guarantees a NUL terminator in its owned allocation.
                return result.reinterpret(MAX_RESULT_BYTES).getString(0);
            } finally {
                free.invokeExact(result);
            }
        } catch (RuntimeException | Error error) {
            throw error;
        } catch (Throwable error) {
            throw new IllegalStateException("Cannot invoke Orchestrata solver", error);
        }
    }

    public String solveJson(String problem, long maxEvaluations) {
        return solveJson(problem.getBytes(StandardCharsets.UTF_8), maxEvaluations);
    }

    @Override
    public synchronized void close() {
        if (!closed) {
            lifetime.close();
            closed = true;
        }
    }

    private static MethodHandle downcall(
        Linker linker, SymbolLookup symbols, String name, FunctionDescriptor signature
    ) {
        return linker.downcallHandle(symbols.find(name).orElseThrow(), signature);
    }
}
