package io.orchestrata;

import java.util.List;
import java.util.Objects;

public record CountValue(
    List<Integer> variables, long value, int min, int max, long weight, ConstraintTier tier
) implements Constraint {
    public CountValue {
        variables = List.copyOf(variables);
        Objects.requireNonNull(tier, "tier");
        if (variables.isEmpty() || min < 0 || min > max || max > variables.size() || weight <= 0) {
            throw new IllegalArgumentException("Invalid count value bounds or weight");
        }
    }
}
