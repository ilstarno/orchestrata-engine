package io.orchestrata;

import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;

public final class PackagedSmoke {
    public static void main(String[] args) {
        verifyResultDecoder();
        var solver = new Solver(Path.of(args[0]));
        try (solver) {
            verifyAdditionalConstraints(solver);
            var emptyResult = solver.solve(new Problem(List.of(), List.of()), 1).orElseThrow();
            if (!emptyResult.evaluated().equals(BigInteger.ONE)) {
                throw new AssertionError("The native empty fixture must take exactly one evaluation");
            }
            var typed = new Problem(
                List.of(new Variable("exact\"\n", List.of(Long.MIN_VALUE, Long.MAX_VALUE))),
                List.of(new PreferValue(0, Long.MAX_VALUE, 1))
            );
            var computed = solver.solve(typed, -1L).orElseThrow();
            if (!computed.values().equals(List.of(Long.MAX_VALUE))
                || computed.score().hard() != 0 || computed.score().soft() != 0
                || computed.evaluated().signum() <= 0) {
                throw new AssertionError("Typed solve lost values, scores, or evaluation count");
            }
            expectStatus(2, () -> solver.solve(typed, 1));
            var noSolution = new Problem(
                List.of(new Variable("a", List.of(0L)), new Variable("b", List.of(0L))),
                List.of(new AllDifferent(List.of(0, 1), 1))
            );
            if (solver.solve(noSolution, 1).isPresent()) {
                throw new AssertionError("Proven infeasibility must not return a solution");
            }

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

    private static void verifyAdditionalConstraints(Solver solver) {
        var zero = BigInteger.ZERO;
        var one = BigInteger.ONE;
        var u64Max = new BigInteger("18446744073709551615");
        var variables = List.of(new Variable("a", List.of(0L)), new Variable("b", List.of(1L)));
        var problem = new Problem(variables, List.of(
            new CountValue(List.of(0, 1), 1L, 1, 1, 7, ConstraintTier.HARD),
            new Precedence(0, 1, one),
            new NoOverlap(List.of(new ScheduledInterval(0, one), new ScheduledInterval(1, zero))),
            new ResourceCapacity(
                List.of(new ResourceLoad(0, one), new ResourceLoad(1, one)),
                List.of(new ResourceLimit(0, one), new ResourceLimit(1, one))
            )
        ));
        var result = solver.solve(problem, 1).orElseThrow();
        if (!result.values().equals(List.of(0L, 1L)) || !result.score().equals(new Score(0, 0))
            || !result.evaluated().equals(one)) {
            throw new AssertionError("Unexpected native result for the four typed constraint variants");
        }
        var maximum = new Problem(variables, List.of(new Precedence(0, 1, u64Max)));
        var wire = new String(new ProblemEncoder().encode(maximum), StandardCharsets.UTF_8);
        if (!wire.contains("\"min_gap\":18446744073709551615")) {
            throw new AssertionError("Unsigned wire value was not encoded as an exact JSON number");
        }
        try {
            new Precedence(0, 1, u64Max.add(one));
            throw new AssertionError("Accepted an out-of-range unsigned 64-bit gap");
        } catch (IllegalArgumentException expected) {
            // The number cannot be represented by the Rust ABI.
        }
    }

    private static void verifyResultDecoder() {
        String reordered = """
            {"solution":{"evaluated":18446744073709551615,
             "score":{"soft":-1,"hard":0},"values":[-9223372036854775808]}}
            """;
        var result = new ResultDecoder(reordered).decode(1).orElseThrow();
        if (!result.values().equals(List.of(Long.MIN_VALUE))
            || !result.evaluated().equals(new BigInteger("18446744073709551615"))) {
            throw new AssertionError("Reordered result did not preserve exact integers");
        }
        for (String malformed : List.of(
            reordered.replace("18446744073709551615", "18446744073709551616"),
            reordered.replace("-9223372036854775808", "9223372036854775808"),
            reordered.replace("\"hard\":0", "\"hard\":0,\"hard\":0"),
            reordered.replace("\"hard\":0", "\"unknown\":0"),
            reordered.replace("\"hard\":0", "\"hard\":0,"),
            reordered + "null"
        )) {
            try {
                new ResultDecoder(malformed).decode(1);
                throw new AssertionError("Accepted malformed native result");
            } catch (IllegalStateException expected) {
                // Invalid native result, distinct from a valid infeasible solve.
            }
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
