<?php

declare(strict_types=1);

namespace Orchestrata;

final class Score
{
    public function __construct(public readonly int $hard, public readonly int $soft)
    {
    }
}
