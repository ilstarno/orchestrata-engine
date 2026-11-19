export type PrecedenceConstraint = { readonly kind: 'precedence'; readonly before: number; readonly after: number; readonly minGap: bigint };
