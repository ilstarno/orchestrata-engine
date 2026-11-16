package io.orchestrata;

public final class SolveException extends RuntimeException {
    private final int status;

    SolveException(int status) {
        super(switch (status) {
            case 1 -> "Invalid problem";
            case 2 -> "Search limit exceeded";
            case 3 -> "Solver execution failed";
            case 4 -> "Invalid native argument";
            default -> "Unknown native status: " + status;
        });
        this.status = status;
    }

    public int status() {
        return status;
    }
}
