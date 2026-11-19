package io.orchestrata;

public record PreferValue(int variable, long value, long softWeight) implements Constraint {
    public PreferValue {
        if (softWeight <= 0) throw new IllegalArgumentException("Expected positive soft weight");
    }
}
