<?php

declare(strict_types=1);

namespace Orchestrata;

final class Problem implements \JsonSerializable
{
    public function __construct(
        public readonly array $variables,
        public readonly array $constraints,
    ) {
        if (!array_is_list($variables)) {
            throw new \InvalidArgumentException('variables must be a list.');
        }
        foreach ($variables as $item) {
            if (!($item instanceof Variable)) {
                throw new \InvalidArgumentException('Invalid element in variables.');
            }
        }
        if (!array_is_list($constraints)) {
            throw new \InvalidArgumentException('constraints must be a list.');
        }
        foreach ($constraints as $item) {
            if (!($item instanceof Constraint)) {
                throw new \InvalidArgumentException('Invalid element in constraints.');
            }
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'variables' => $this->variables,
            'constraints' => $this->constraints,
        ];
    }
}
