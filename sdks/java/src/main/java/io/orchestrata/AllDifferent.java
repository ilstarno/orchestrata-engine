package io.orchestrata;

import java.util.List;

public record AllDifferent(List<Integer> variables, long hardWeight) implements Constraint {
    public AllDifferent {
        variables = List.copyOf(variables);
        if (hardWeight <= 0) throw new IllegalArgumentException("Expected positive hard weight");
    }
}
