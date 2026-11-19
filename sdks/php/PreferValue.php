<?php

declare(strict_types=1);

namespace Orchestrata;

final class PreferValue implements Constraint
{
    public function __construct(
        public readonly int $variable,
        public readonly int $value,
        public readonly int $softWeight,
    ) {
        if ($variable < 0) {
            throw new \InvalidArgumentException('variable must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'prefer_value',
            'variable' => $this->variable,
            'value' => $this->value,
            'soft_weight' => $this->softWeight,
        ];
    }
}
