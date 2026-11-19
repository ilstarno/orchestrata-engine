import type { Solution } from './solution';
export type SolveOutcome =
    | { readonly kind: 'solved'; readonly solution: Solution }
    | { readonly kind: 'infeasible' };
