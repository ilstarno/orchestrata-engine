<?php

declare(strict_types=1);

require __DIR__ . '/../../sdks/php/SolveException.php';
require __DIR__ . '/../../sdks/php/Solver.php';

$solver = new Orchestrata\Solver($argv[1]);
$problem = '{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}';
if (!str_contains($solver->solveJson($problem, 1), '"values":[9223372036854775807]')) {
    throw new RuntimeException('The solver lost integer precision.');
}

try {
    $solver->solveJson($problem, 0);
} catch (Orchestrata\SolveException $exception) {
    if ($exception->status === 2) {
        exit(0);
    }

    throw $exception;
}
throw new RuntimeException('Expected a search limit error.');
