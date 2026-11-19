package io.orchestrata;

import java.util.List;

public record NoOverlap(List<ScheduledInterval> intervals) implements Constraint {
    public NoOverlap {
        intervals = List.copyOf(intervals);
        if (intervals.size() < 2) throw new IllegalArgumentException("Expected at least two intervals");
    }
}
