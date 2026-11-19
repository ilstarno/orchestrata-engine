<?php

declare(strict_types=1);

require $argv[1];

use Orchestrata\{AllDifferent, CountValue, CountValueTier, NoOverlap, NotEqualOffset,
    Precedence, PreferValue, Problem, ResourceCapacity, ResourceLimit, ResourceLoad,
    ScheduledInterval, Solution, SolveException, Solver, Variable};

function expectStatus(int $status, callable $call): void
{
    try {
        $call();
    } catch (SolveException $error) {
        if ($error->status !== $status) {
            throw $error;
        }
        return;
    }
    throw new RuntimeException("Expected native status {$status}.");
}

$solver = new Solver($argv[2]);
$variables = [new Variable('α', [0]), new Variable('β', [10]), new Variable('γ', [2])];
$rules = [
    new AllDifferent([0, 0], 1),
    new NotEqualOffset(0, 1, -10, 1),
    new CountValue([2], 3, 1, 1, 1, CountValueTier::Hard),
    new Precedence(0, 1, 11),
    new NoOverlap([new ScheduledInterval(0, 11), new ScheduledInterval(1, 1)]),
    new ResourceCapacity([new ResourceLoad(2, PHP_INT_MAX)], [new ResourceLimit(2, PHP_INT_MAX - 1)]),
];
foreach ($rules as $rule) {
    if ($solver->solve(new Problem($variables, [$rule]), 1) !== null) {
        throw new RuntimeException('Expected infeasibility for ' . $rule::class);
    }
}
foreach ([new PreferValue(1, 11, 3), new CountValue([2], 3, 1, 1, 3, CountValueTier::Soft)] as $rule) {
    $result = $solver->solve(new Problem($variables, [$rule]), 1);
    if ($result === null || $result->score->soft !== -3 || $result->values !== [0, 10, 2]) {
        throw new RuntimeException('Incorrect soft cost.');
    }
}
$exact = new Problem([new Variable('min', [PHP_INT_MIN]), new Variable('max', [PHP_INT_MAX])], []);
$result = $solver->solve($exact, 1);
if ($result === null || $result->values !== [PHP_INT_MIN, PHP_INT_MAX] || $result->evaluated !== 1) {
    throw new RuntimeException('Integer precision changed.');
}
expectStatus(2, fn () => $solver->solve($exact, 0));
expectStatus(1, fn () => $solver->solve(new Problem($variables, [new Precedence(0, 0, 0)]), 1));
expectStatus(4, fn () => $solver->solveJson('', 1));
expectStatus(4, fn () => $solver->solveJson(str_repeat(' ', 1048577), 1));
expectStatus(3, fn () => $solver->solve(new Problem($variables, [
    new PreferValue(0, 1, PHP_INT_MAX), new PreferValue(0, 1, 2),
]), 1));
$wide = Solution::fromAbiResult('{"solution":{"evaluated":18446744073709551615,"values":[0],"score":{"soft":0,"hard":0}}}');
if ($wide === null || $wide->evaluated !== '18446744073709551615') {
    throw new RuntimeException('Unsigned result precision changed.');
}
foreach (['{}', '{"solution":false}', '{"solution":{"values":[1.1],"score":{"hard":0,"soft":0},"evaluated":1}}'] as $bad) {
    try {
        Solution::fromAbiResult($bad);
    } catch (UnexpectedValueException) {
        continue;
    }
    throw new RuntimeException('Accepted an invalid result.');
}
echo "Seven constraint variants, infeasibility, errors and exact precision passed.\n";
