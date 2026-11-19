import type { SolveErrorCode } from './solve-error-code';
export declare class SolveError extends Error {
    constructor(code: SolveErrorCode, message: string, nativeStatus?: number);
    readonly code: SolveErrorCode;
    readonly nativeStatus?: number;
}
