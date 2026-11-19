package io.orchestrata;

import java.math.BigInteger;
import java.util.Objects;

final class UnsignedValue {
    private UnsignedValue() {}

    static void check(BigInteger value, String field) {
        Objects.requireNonNull(value, field);
        if (value.signum() < 0 || value.bitLength() > 64) {
            throw new IllegalArgumentException(field + " must be an unsigned 64-bit integer");
        }
    }
}
