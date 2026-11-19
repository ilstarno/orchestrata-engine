package io.orchestrata;

import java.math.BigInteger;

public record ResourceLoad(int variable, BigInteger load) {
    public ResourceLoad {
        UnsignedValue.check(load, "load");
    }
}
