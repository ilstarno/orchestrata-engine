package io.orchestrata;

public enum ConstraintTier {
    HARD("hard"),
    SOFT("soft");

    final String wireName;

    ConstraintTier(String wireName) {
        this.wireName = wireName;
    }
}
