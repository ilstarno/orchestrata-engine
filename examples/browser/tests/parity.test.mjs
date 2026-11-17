import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { createEngine, maxEvaluations, maxRequestBytes } from '../public/try-engine.mjs';

const bytes = readFileSync(new URL('../public/try-engine.wasm', import.meta.url));
const module = await WebAssembly.compile(bytes);
const engine = await createEngine(module);
const simple = { variables: [{ name: 'job', domain: [0, 1] }], constraints: [{ kind: 'prefer_value', variable: 0, value: 1, soft_weight: 1 }] };

test('trusted binary has no host imports and runs the standalone engine', () => {
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  assert.equal(maxEvaluations, 4096);
  assert.deepEqual(engine.solveProblem(simple, 2), { solution: { values: [1], score: { hard: 0, soft: 0 }, evaluated: 2 } });
});

test('invalid model, invalid request, budget, proven infeasible, and overflow are different outcomes', () => {
  const variables = [{ name: 'a', domain: [0] }, { name: 'b', domain: [0] }];
  const constraints = [{ kind: 'all_different', variables: [0, 1], hard_weight: 1 }];
  assert.throws(() => engine.solveProblem(simple, 1), { code: 'search_limit' });
  assert.throws(() => engine.solveProblem(simple, 4097), { code: 'invalid_budget' });
  assert.throws(() => engine.solveRequest('{'), { code: 'invalid_problem' });
  assert.throws(() => engine.solveRequest(''), { code: 'invalid_request' });
  assert.throws(() => engine.solveRequest('€'.repeat(maxRequestBytes / 2)), { code: 'invalid_request' });
  assert.throws(() => engine.solveProblem({ variables: [{ name: 'x', domain: [] }], constraints: [] }), { code: 'invalid_problem' });
  assert.deepEqual(engine.solveProblem({ variables, constraints }), { solution: null });
  assert.throws(() => engine.solveRequest('{"variables":[{"name":"x","domain":[0]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":9223372036854775807},{"kind":"prefer_value","variable":0,"value":1,"soft_weight":9223372036854775807}]}'), { code: 'engine_error' });
});

test('raw native JSON preserves 64-bit precision, object helper refuses implicit rounding', () => {
  const raw = '{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}';
  assert.match(engine.solveRequest(raw), /"values":\[9223372036854775807\]/);
  assert.throws(() => engine.solveProblem({ variables: [{ name: 'x', domain: [Number.MAX_SAFE_INTEGER + 1] }], constraints: [] }), { code: 'unsafe_number' });
});

test('high row count on a fixed-size search works without claiming arbitrary row capacity', () => {
  const vars = Array.from({ length: 200 }, (_, i) => ({ name: `row-${i}-${'a'.repeat(70)}`, domain: [1] }));
  const request = JSON.stringify({ variables: vars, constraints: [] });
  assert.ok(request.length > 8192);
  assert.equal(engine.solveProblem({ variables: vars, constraints: [] }, 1).solution.values.length, 200);
  assert.throws(() => engine.solveProblem({ variables: vars.map(row => ({ ...row, domain: [0, 1] })), constraints: [] }), { code: 'search_limit' });
});

test('repeated calls do not leak owned allocations or state', () => {
  for (let i = 0; i < 300; i++) {
    if (i % 2) assert.throws(() => engine.solveProblem(simple, 1), { code: 'search_limit' });
    else assert.equal(engine.solveProblem(simple, 2).solution.values[0], 1);
  }
});

const fixtures = process.env.ORCHESTRATA_FIXTURES_DIR;
if (fixtures) {
  for (const name of readdirSync(fixtures, { withFileTypes: true }).filter(item => item.isDirectory() && item.name !== 'retail-staffing-what-if').map(item => item.name)) {
    test(`published fixture matches native expected result: ${name}`, () => {
      const request = readFileSync(resolve(fixtures, name, 'problem.json'), 'utf8');
      const expected = JSON.parse(readFileSync(resolve(fixtures, name, 'expected.json'), 'utf8'));
      assert.deepEqual(JSON.parse(engine.solveRequest(request)), { solution: expected });
    });
  }
}

test('matching reassigns a flexible job to cover a constrained job', () => {
  assert.deepEqual(
    engine.matchCapacity({ worker_capacities: [1, 1], eligible_workers_by_job: [[0, 1], [0], []] }),
    { workers_by_job: [1, 0, null], assigned: 2 },
  );
  const malformed = [
    [[1], [[0]]],
    { worker_capacities: [1], eligible_workers_by_job: [[0, 0]] },
    { worker_capacities: [1], eligible_workers_by_job: [[1]] },
    { worker_capacities: [1], eligible_workers_by_job: [[0]], privateField: true },
    { worker_capacities: [201], eligible_workers_by_job: [[0]] },
    { worker_capacities: [200], eligible_workers_by_job: Array.from({length:201}, () => [0]) },
  ];
  for (const problem of malformed) {
    assert.throws(() => engine.matchCapacity(problem), { code: 'invalid_capacity_problem' });
  }
});

test('actual 200×200 broad-eligibility model fits the capacity byte limit', () => {
  const all = Array.from({ length: 200 }, (_, i) => i);
  const input = { worker_capacities: all.map(() => 1), eligible_workers_by_job: all.map(() => all) };
  assert.ok(JSON.stringify(input).length > 128 * 1024);
  const result = engine.matchCapacity(input);
  assert.equal(result.assigned, 200); // 200 jobs is also the independent upper bound.
  assert.equal(new Set(result.workers_by_job).size, 200);
  assert.deepEqual([...result.workers_by_job].sort((a, b) => a - b), all);
  input.worker_capacities = all.map((n) => Number(n < 180));
  const constrained = engine.matchCapacity(input);
  const workers = constrained.workers_by_job.filter(i => i !== null);
  assert.equal(constrained.assigned, 180); // Capacity upper bound, met exactly.
  assert.equal(workers.length, 180);
  assert.deepEqual([...workers].sort((a, b) => a - b), all.slice(0, 180));
});

test('WASM matching and independent brute force agree on every three-job eligibility graph', () => {
  const work = [0, 1, 2];
  function brute(eligibility, remaining, job = 0) {
    if (job === eligibility.length) return 0;
    let best = brute(eligibility, remaining, job + 1);
    for (const w of eligibility[job]) {
      if (remaining[w] === 0) continue;
      const capacity = [...remaining];
      capacity[w]--;
      best = Math.max(best, 1 + brute(eligibility, capacity, job + 1));
    }
    return best;
  }
  for (let m = 0; m < 2 ** 9; m++) {
    const eligibility = work.map(j => work.filter(w => m & 1 << (j * 3 + w)));
    for (const worker_capacities of [[1, 1, 1], [0, 1, 2], [2, 2, 0], [0, 0, 0]]) {
      const result = engine.matchCapacity({ worker_capacities, eligible_workers_by_job: eligibility });
      assert.equal(result.assigned, brute(eligibility, worker_capacities));
      const remaining = [...worker_capacities];
      let covered = 0;
      assert.equal(result.workers_by_job.length, eligibility.length);
      result.workers_by_job.forEach((worker, job) => {
        if (worker === null) return;
        assert.ok(eligibility[job].includes(worker));
        assert.ok(remaining[worker]-- > 0);
        covered++;
      });
      assert.equal(covered, result.assigned);
    }
  }
});
