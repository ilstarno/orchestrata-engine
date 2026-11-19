package io.orchestrata;

public sealed interface Constraint permits AllDifferent, NotEqualOffset, PreferValue,
    CountValue, Precedence, NoOverlap, ResourceCapacity {}
