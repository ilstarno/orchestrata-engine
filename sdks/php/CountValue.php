<?php

declare(strict_types=1);

namespace Orchestrata;

final class CountValue implements Constraint
{
    public function __construct(
        public readonly array $variables,
        public readonly int $value,
        public readonly int $min,
        public readonly int $max,
        public readonly int $weight,
        public readonly CountValueTier $tier,
    ) {
        if (!array_is_list($variables)) {
            throw new \InvalidArgumentException('variables must be a list.');
        }
        foreach ($variables as $item) {
            if (!(is_int($item) && $item >= 0)) {
                throw new \InvalidArgumentException('Invalid element in variables.');
            }
        }
        if ($min < 0) {
            throw new \InvalidArgumentException('min must be nonnegative.');
        }
        if ($max < 0) {
            throw new \InvalidArgumentException('max must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'count_value',
            'variables' => $this->variables,
            'value' => $this->value,
            'min' => $this->min,
            'max' => $this->max,
            'weight' => $this->weight,
            'tier' => $this->tier->value,
        ];
    }
}
