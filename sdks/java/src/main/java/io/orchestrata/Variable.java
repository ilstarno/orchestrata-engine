package io.orchestrata;

import java.util.List;
import java.util.Objects;

public record Variable(String name, List<Long> domain) {
    public Variable {
        Objects.requireNonNull(name, "name");
        domain = List.copyOf(domain);
        if (domain.isEmpty()) {
            throw new IllegalArgumentException("Domain must not be empty");
        }
    }
}
