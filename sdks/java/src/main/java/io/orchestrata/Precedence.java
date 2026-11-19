package io.orchestrata;

import java.math.BigInteger;

public record Precedence(int before, int after, BigInteger minGap) implements Constraint {
    public Precedence {
        UnsignedValue.check(minGap, "minGap");
    }
}
