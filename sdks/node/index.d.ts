import type { Buffer } from 'node:buffer';

/** Returns the raw lossless JSON result. Do not parse 64-bit values with JSON.parse. */
export function solveJson(problem: string | Buffer, maxEvaluations: bigint): string;
