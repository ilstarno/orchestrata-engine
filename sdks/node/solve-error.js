'use strict';

class SolveError extends Error {
    constructor(code, message, nativeStatus) {
        super(message);
        this.name = 'SolveError';
        this.code = code;
        this.nativeStatus = nativeStatus;
    }
}

function nativeFailure(error) {
    const status = Number.isInteger(error?.status) ? error.status : undefined;
    switch (status) {
        case 1: return new SolveError('invalid_input', 'The engine rejected the problem.', status);
        case 2: return new SolveError('search_limit_exceeded', 'The search exceeds the evaluation budget.', status);
        case 4: return new SolveError('invalid_input', 'The engine rejected the request size or arguments.', status);
        default: return new SolveError('execution_failed', 'The native solver could not complete the request.', status);
    }
}

module.exports = { SolveError, nativeFailure };
