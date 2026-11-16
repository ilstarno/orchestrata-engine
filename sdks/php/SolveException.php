<?php

declare(strict_types=1);

namespace Orchestrata;

final class SolveException extends \RuntimeException
{
    public function __construct(public readonly int $status)
    {
        parent::__construct(match ($status) {
            1 => 'Invalid problem.',
            2 => 'Search limit exceeded.',
            3 => 'Solver execution failed.',
            4 => 'Invalid native argument.',
            default => "Unknown native status: {$status}.",
        }, $status);
    }
}
