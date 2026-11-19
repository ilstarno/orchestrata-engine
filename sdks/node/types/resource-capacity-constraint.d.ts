import type { ResourceLoad } from './resource-load';
import type { ResourceLimit } from './resource-limit';
export type ResourceCapacityConstraint = { readonly kind: 'resource_capacity'; readonly items: readonly ResourceLoad[]; readonly limits: readonly ResourceLimit[] };
