<?php

declare(strict_types=1);

namespace Orchestrata;

final class Precedence implements Constraint
{
    public function __construct(
        public readonly int $before,
        public readonly int $after,
        public readonly int $minGap,
    ) {
        if ($before < 0) {
            throw new \InvalidArgumentException('before must be nonnegative.');
        }
        if ($after < 0) {
            throw new \InvalidArgumentException('after must be nonnegative.');
        }
        if ($minGap < 0) {
            throw new \InvalidArgumentException('minGap must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'precedence',
            'before' => $this->before,
            'after' => $this->after,
            'min_gap' => $this->minGap,
        ];
    }
}
