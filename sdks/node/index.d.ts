import type { Buffer } from 'node:buffer';
import type { Problem } from './types/problem';
import type { SolveOutcome } from './types/solve-outcome';

export type { Variable } from './types/variable';
export type { Constraint } from './types/constraint';
export type { ScheduledInterval } from './types/scheduled-interval';
export type { ResourceLoad } from './types/resource-load';
export type { ResourceLimit } from './types/resource-limit';
export type { Problem } from './types/problem';
export type { Score } from './types/score';
export type { Solution } from './types/solution';
export type { SolveOutcome } from './types/solve-outcome';
export type { SolveErrorCode } from './types/solve-error-code';
export { SolveError } from './types/solve-error';
export type { AllDifferentConstraint } from './types/all-different-constraint';
export type { NotEqualOffsetConstraint } from './types/not-equal-offset-constraint';
export type { PreferValueConstraint } from './types/prefer-value-constraint';
export type { CountValueConstraint } from './types/count-value-constraint';
export type { PrecedenceConstraint } from './types/precedence-constraint';
export type { NoOverlapConstraint } from './types/no-overlap-constraint';
export type { ResourceCapacityConstraint } from './types/resource-capacity-constraint';

/** Executes exact search. An infeasible outcome is proven; a budget rejection throws SolveError. */
export function solve(problem: Problem, maxEvaluations: bigint): SolveOutcome;

/** Legacy raw JSON escape hatch. Do not parse its 64-bit values with Number. */
export function solveJson(problem: string | Buffer, maxEvaluations: bigint): string;
