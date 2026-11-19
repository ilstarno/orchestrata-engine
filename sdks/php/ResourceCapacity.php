<?php

declare(strict_types=1);

namespace Orchestrata;

final class ResourceCapacity implements Constraint
{
    public function __construct(
        public readonly array $items,
        public readonly array $limits,
    ) {
        if (!array_is_list($items)) {
            throw new \InvalidArgumentException('items must be a list.');
        }
        foreach ($items as $item) {
            if (!($item instanceof ResourceLoad)) {
                throw new \InvalidArgumentException('Invalid element in items.');
            }
        }
        if (!array_is_list($limits)) {
            throw new \InvalidArgumentException('limits must be a list.');
        }
        foreach ($limits as $item) {
            if (!($item instanceof ResourceLimit)) {
                throw new \InvalidArgumentException('Invalid element in limits.');
            }
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'resource_capacity',
            'items' => $this->items,
            'limits' => $this->limits,
        ];
    }
}
