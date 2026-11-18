package io.orchestrata;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.Arrays;

public final class PackagedSmoke {
    public static void main(String[] args) {
        var solver = new Solver(Path.of(args[0]));
        try (solver) {
            String two = """
                {"variables":[{"name":"exact","domain":[-9223372036854775808,9223372036854775807]}],
                 "constraints":[{"kind":"prefer_value","variable":0,"value":9223372036854775807,"soft_weight":1}]}
                """;
            if (!solver.solveJson(two, -1L).contains("\"values\":[9223372036854775807]")) {
                throw new AssertionError("Unsigned budget bits or exact signed values were lost");
            }
            expectStatus(2, () -> solver.solveJson(two, 1));
            expectStatus(1, () -> solver.solveJson("{", 1));
            expectStatus(4, () -> solver.solveJson(new byte[0], 1));
            String impossible = """
                {"variables":[{"name":"a","domain":[0]},{"name":"b","domain":[0]}],
                 "constraints":[{"kind":"all_different","variables":[0,1],"hard_weight":1}]}
                """;
            if (!solver.solveJson(impossible, 1).equals("{\"solution\":null}")) {
                throw new AssertionError("Proven infeasibility is not a budget error");
            }

            byte[] request = new byte[1_048_576];
            Arrays.fill(request, (byte) ' ');
            byte[] empty = "{\"variables\":[],\"constraints\":[]}".getBytes(StandardCharsets.UTF_8);
            System.arraycopy(empty, 0, request, 0, empty.length);
            if (!solver.solveJson(request, 1).contains("\"evaluated\":1")) {
                throw new AssertionError("Exact request limit was not accepted");
            }
            expectStatus(4, () -> solver.solveJson(Arrays.copyOf(request, request.length + 1), 1));
        }
        solver.close();
        try {
            solver.solveJson("{", 1);
            throw new AssertionError("Expected use-after-close to fail");
        } catch (IllegalStateException e) {
            if (!"Solver is closed".equals(e.getMessage())) throw e;
        }
        try (var incomplete = new Solver(Path.of(args[1]))) {
            throw new AssertionError("Expected a library missing a symbol to fail: " + incomplete);
        } catch (IllegalStateException e) {
            if (!"Missing Orchestrata native symbol: orchestrata_solve_json_v1".equals(e.getMessage())) throw e;
        }
    }

    private static void expectStatus(int status, Runnable call) {
        try { call.run(); }
        catch (SolveException failure) {
            if (failure.status() != status) throw failure;
            return;
        }
        throw new AssertionError("Expected native status " + status);
    }
}
