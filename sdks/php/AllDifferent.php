<?php

declare(strict_types=1);

namespace Orchestrata;

final class AllDifferent implements Constraint
{
    public function __construct(
        public readonly array $variables,
        public readonly int $hardWeight,
    ) {
        if (!array_is_list($variables)) {
            throw new \InvalidArgumentException('variables must be a list.');
        }
        foreach ($variables as $item) {
            if (!(is_int($item) && $item >= 0)) {
                throw new \InvalidArgumentException('Invalid element in variables.');
            }
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'all_different',
            'variables' => $this->variables,
            'hard_weight' => $this->hardWeight,
        ];
    }
}
