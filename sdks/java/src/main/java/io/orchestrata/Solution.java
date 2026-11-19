package io.orchestrata;

import java.math.BigInteger;
import java.util.List;

public record Solution(List<Long> values, Score score, BigInteger evaluated) {
    public Solution {
        values = List.copyOf(values);
    }
}
