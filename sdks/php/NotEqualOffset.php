<?php

declare(strict_types=1);

namespace Orchestrata;

final class NotEqualOffset implements Constraint
{
    public function __construct(
        public readonly int $left,
        public readonly int $right,
        public readonly int $offset,
        public readonly int $hardWeight,
    ) {
        if ($left < 0) {
            throw new \InvalidArgumentException('left must be nonnegative.');
        }
        if ($right < 0) {
            throw new \InvalidArgumentException('right must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'not_equal_offset',
            'left' => $this->left,
            'right' => $this->right,
            'offset' => $this->offset,
            'hard_weight' => $this->hardWeight,
        ];
    }
}
