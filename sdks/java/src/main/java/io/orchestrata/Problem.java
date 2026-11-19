package io.orchestrata;

import java.util.List;

public record Problem(List<Variable> variables, List<Constraint> constraints) {
    public Problem {
        variables = List.copyOf(variables);
        constraints = List.copyOf(constraints);
    }
}
