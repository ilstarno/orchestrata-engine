import type { AllDifferentConstraint } from './all-different-constraint';
import type { NotEqualOffsetConstraint } from './not-equal-offset-constraint';
import type { PreferValueConstraint } from './prefer-value-constraint';
import type { CountValueConstraint } from './count-value-constraint';
import type { PrecedenceConstraint } from './precedence-constraint';
import type { NoOverlapConstraint } from './no-overlap-constraint';
import type { ResourceCapacityConstraint } from './resource-capacity-constraint';
export type Constraint =
    | AllDifferentConstraint
    | NotEqualOffsetConstraint
    | PreferValueConstraint
    | CountValueConstraint
    | PrecedenceConstraint
    | NoOverlapConstraint
    | ResourceCapacityConstraint;
