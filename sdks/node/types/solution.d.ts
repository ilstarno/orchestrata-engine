import type { Score } from './score';
export interface Solution {
    readonly values: readonly bigint[];
    readonly score: Score;
    readonly evaluated: bigint;
}
