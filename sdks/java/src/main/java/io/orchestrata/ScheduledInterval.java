package io.orchestrata;

import java.math.BigInteger;

public record ScheduledInterval(int start, BigInteger duration) {
    public ScheduledInterval {
        UnsignedValue.check(duration, "duration");
    }
}
