package io.orchestrata;

import java.math.BigInteger;

public record ResourceLimit(long resource, BigInteger capacity) {
    public ResourceLimit {
        UnsignedValue.check(capacity, "capacity");
    }
}
