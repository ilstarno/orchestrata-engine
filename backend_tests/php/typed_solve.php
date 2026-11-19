<?php

declare(strict_types=1);

spl_autoload_register(static function (string $class): void {
    $prefix = 'Orchestrata\\';
    if (str_starts_with($class, $prefix)) {
        require __DIR__ . '/../../sdks/php/' . substr($class, strlen($prefix)) . '.php';
    }
});

use Orchestrata\AllDifferent;
use Orchestrata\PreferValue;
use Orchestrata\Problem;
use Orchestrata\Solution;
use Orchestrata\Solver;
use Orchestrata\Variable;

$problem = new Problem(
    [new Variable('x', [1, 2]), new Variable('y', [1, 2])],
    [new AllDifferent([0, 1], 5), new PreferValue(0, 2, 3)],
);
$document = json_decode(json_encode($problem, JSON_THROW_ON_ERROR), true, 512, JSON_THROW_ON_ERROR);
if ($document['constraints'][0] !== ['kind' => 'all_different', 'variables' => [0, 1], 'hard_weight' => 5]) {
    throw new RuntimeException('Constraint differs from Rust wire format.');
}

$wide = Solution::fromAbiResult('{"solution":{"values":[9223372036854775807],"score":{"hard":0,"soft":0},"evaluated":18446744073709551615}}');
if ($wide === null || $wide->values !== [PHP_INT_MAX] || $wide->evaluated !== '18446744073709551615') {
    throw new RuntimeException('Incorrect numeric precision.');
}

if (Solution::fromAbiResult('{"solution":null}') !== null) {
    throw new RuntimeException('Expected an explicitly infeasible result.');
}
foreach (['{}', '{"values":[],"score":{"hard":0,"soft":0},"evaluated":1}'] as $invalid) {
    try {
        Solution::fromAbiResult($invalid);
        throw new RuntimeException('Accepted a missing solution envelope.');
    } catch (UnexpectedValueException) {
    }
}

if (isset($argv[1])) {
    $solver = new Solver($argv[1]);
    $result = $solver->solve($problem, 20);
    if ($result === null || $result->values !== [2, 1] || $result->score->hard !== 0 || $result->score->soft !== 0) {
        throw new RuntimeException('Unexpected native solution.');
    }
    $infeasible = new Problem(
        [new Variable('x', [1]), new Variable('y', [1])],
        [new AllDifferent([0, 1], 5)],
    );
    if ($solver->solve($infeasible, 20) !== null) {
        throw new RuntimeException('Native solver did not report infeasibility.');
    }
}
