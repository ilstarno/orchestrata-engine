package io.orchestrata;

import java.util.List;

public record ResourceCapacity(List<ResourceLoad> items, List<ResourceLimit> limits) implements Constraint {
    public ResourceCapacity {
        items = List.copyOf(items);
        limits = List.copyOf(limits);
        if (items.isEmpty() || limits.isEmpty()) {
            throw new IllegalArgumentException("Expected at least one item and resource limit");
        }
    }
}
