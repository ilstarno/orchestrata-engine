export const maxRequestBytes = 128 * 1024;
export const maxEvaluations = 4096;
export const maxCapacityRequestBytes = 256 * 1024;

const messages = {
  1: ['invalid_problem', 'The planning input is invalid.'],
  2: ['search_limit', 'The exact search exceeds its budget. No result was established.'],
  3: ['engine_error', 'The engine could not return a result.'],
  4: ['invalid_request', 'The planning input must fit within 128 KiB.'],
  5: ['invalid_budget', 'Choose a whole-number budget from 1 to 4096.'],
  6: ['invalid_capacity_problem', 'Check worker capacity and eligible workers for each job. Use no more than 200 jobs and 200 workers.'],
  7: ['invalid_request', 'Capacity planning input must fit within 256 KiB.'],
};

function failure(status) {
  const [code, message] = messages[status] ?? messages[3];
  return Object.assign(new Error(message), { code });
}

/** A module made from trusted project assets; instantiated with no imports. */
export async function createEngine(source) {
  const module = source instanceof WebAssembly.Module ? source : await WebAssembly.compile(source);
  const { exports } = await WebAssembly.instantiate(module);
  const { memory, demo_alloc, demo_free, demo_solve, demo_match_capacity, demo_free_result } = exports;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder('utf-8', { fatal: true });

  function runRequest(json, limit, invalidRequestStatus, compute) {
    if (typeof json !== 'string' || json.length > limit) throw failure(invalidRequestStatus);
    const bytes = encoder.encode(json);
    if (!bytes.length || bytes.length > limit) throw failure(invalidRequestStatus);
    const ptr = demo_alloc(bytes.length);
    if (!ptr) throw failure(3);
    try {
      new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
      const result = compute(ptr, bytes.length);
      if (result < 0n && result >= -7n) throw failure(Number(-result));
      const address = Number(BigInt.asUintN(32, result >> 32n));
      const length = Number(BigInt.asUintN(32, result));
      if (!address || !length) throw failure(3);
      try {
        return decoder.decode(new Uint8Array(memory.buffer, address, length));
      } finally {
        demo_free_result(address);
      }
    } finally {
      demo_free(ptr, bytes.length);
    }
  }

  /** Returns the exact native JSON without rounding integers. */
  function solveRequest(json, budget = maxEvaluations) {
    if (!Number.isSafeInteger(budget) || budget < 1 || budget > maxEvaluations) throw failure(5);
    return runRequest(json, maxRequestBytes, 4, (ptr, size) => demo_solve(ptr, size, budget));
  }

  /** Small demo object helper. Use solveRequest for full-range JSON integers. */
  function solveProblem(problem, budget = maxEvaluations) {
    const checkNumber = (_key, value) => {
      if (typeof value === 'number' && !Number.isSafeInteger(value)) {
        throw Object.assign(new Error('Use exact JSON text for integers outside the safe JavaScript range.'), { code: 'unsafe_number' });
      }
      return value;
    };
    return JSON.parse(solveRequest(JSON.stringify(problem, checkNumber), budget), checkNumber);
  }

  /** Maximize assigned jobs subject only to eligible workers and their capacities. */
  function matchCapacity(problem) {
    return JSON.parse(runRequest(JSON.stringify(problem), maxCapacityRequestBytes, 7, demo_match_capacity));
  }

  return Object.freeze({ solveRequest, solveProblem, matchCapacity });
}
