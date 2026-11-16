import { solve, SolveError, type Problem, type SolveOutcome } from '../../sdks/node';

const problem = {
    variables: [{ name: 'start', domain: [0n, 5n] }],
    constraints: [{ kind: 'prefer_value', variable: 0, value: 5n, softWeight: 1n }],
} satisfies Problem;

const outcome: SolveOutcome = solve(problem, 2n);
if (outcome.kind === 'solved') {
    const value: bigint = outcome.solution.values[0];
    const evaluated: bigint = outcome.solution.evaluated;
    void [value, evaluated];
} else {
    // @ts-expect-error a proven infeasible result does not carry a solution
    void outcome.solution;
}

try {
    // @ts-expect-error required bigint budget
    solve(problem, 2);
    // @ts-expect-error domain values must be exact bigints
    solve({ ...problem, variables: [{ name: 'job', domain: [5] }] }, 2n);
    // @ts-expect-error precedence duration must be bigint and named minGap
    solve({ ...problem, constraints: [{ kind: 'precedence', before: 0, after: 0, min_gap: 1 }] }, 2n);
} catch (error) {
    if (error instanceof SolveError && error.code === 'search_limit_exceeded') {
        const status: number | undefined = error.nativeStatus;
        void status;
    }
}
