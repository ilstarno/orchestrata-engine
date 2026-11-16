import type { Buffer } from 'node:buffer';

/** Integer-domain variable. Values are signed 64-bit bigints. */
export interface Variable {
    readonly name: string;
    readonly domain: readonly bigint[];
}

/** Indices and counts are nonnegative safe integer Numbers; 64-bit values use bigint. */
export type Constraint =
    | { readonly kind: 'all_different'; readonly variables: readonly number[]; readonly hardWeight: bigint }
    | { readonly kind: 'not_equal_offset'; readonly left: number; readonly right: number; readonly offset: bigint; readonly hardWeight: bigint }
    | { readonly kind: 'prefer_value'; readonly variable: number; readonly value: bigint; readonly softWeight: bigint }
    | { readonly kind: 'count_value'; readonly variables: readonly number[]; readonly value: bigint; readonly min: number; readonly max: number; readonly weight: bigint; readonly tier: 'hard' | 'soft' }
    | { readonly kind: 'precedence'; readonly before: number; readonly after: number; readonly minGap: bigint }
    | { readonly kind: 'no_overlap'; readonly intervals: readonly ScheduledInterval[] }
    | { readonly kind: 'resource_capacity'; readonly items: readonly ResourceLoad[]; readonly limits: readonly ResourceLimit[] };

export interface ScheduledInterval {
    readonly start: number;
    readonly duration: bigint;
}

export interface ResourceLoad {
    readonly variable: number;
    readonly load: bigint;
}

export interface ResourceLimit {
    readonly resource: bigint;
    readonly capacity: bigint;
}

export interface Problem {
    readonly variables: readonly Variable[];
    readonly constraints: readonly Constraint[];
}

export interface Score {
    readonly hard: bigint;
    readonly soft: bigint;
}

export interface Solution {
    readonly values: readonly bigint[];
    readonly score: Score;
    readonly evaluated: bigint;
}

export type SolveOutcome =
    | { readonly kind: 'solved'; readonly solution: Solution }
    | { readonly kind: 'infeasible' };

export type SolveErrorCode =
    | 'invalid_input'
    | 'search_limit_exceeded'
    | 'execution_failed'
    | 'invalid_response'
    | 'unsupported_runtime';

export declare class SolveError extends Error {
    constructor(code: SolveErrorCode, message: string, nativeStatus?: number);
    readonly code: SolveErrorCode;
    readonly nativeStatus?: number;
}

/** Executes exact search. An infeasible outcome is proven; a budget rejection throws SolveError. */
export function solve(problem: Problem, maxEvaluations: bigint): SolveOutcome;

/** Legacy raw JSON escape hatch. Do not parse its 64-bit values with Number. */
export function solveJson(problem: string | Buffer, maxEvaluations: bigint): string;
