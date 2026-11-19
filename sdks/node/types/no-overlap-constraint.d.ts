import type { ScheduledInterval } from './scheduled-interval';
export type NoOverlapConstraint = { readonly kind: 'no_overlap'; readonly intervals: readonly ScheduledInterval[] };
