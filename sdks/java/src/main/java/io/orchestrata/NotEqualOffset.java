package io.orchestrata;

public record NotEqualOffset(int left, int right, long offset, long hardWeight) implements Constraint {
    public NotEqualOffset {
        if (hardWeight <= 0) throw new IllegalArgumentException("Expected positive hard weight");
    }
}
