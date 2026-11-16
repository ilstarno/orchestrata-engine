package io.orchestrata;

import java.nio.file.Path;

public final class Smoke {
    public static void main(String[] args) {
        try (var solver = new Solver(Path.of(args[0]))) {
            String problem = """
                {"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}
                """;
            if (!solver.solveJson(problem, 1).contains("\"values\":[9223372036854775807]")) {
                throw new AssertionError("The solver lost integer precision");
            }

            try {
                solver.solveJson(problem, 0);
            } catch (SolveException exception) {
                if (exception.status() == 2) {
                    return;
                }

                throw exception;
            }
            throw new AssertionError("Expected a search limit error");
        }
    }
}
