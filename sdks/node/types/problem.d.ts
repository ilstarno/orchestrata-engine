import type { Variable } from './variable';
import type { Constraint } from './constraint';
export interface Problem {
    readonly variables: readonly Variable[];
    readonly constraints: readonly Constraint[];
}
