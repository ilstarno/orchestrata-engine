<?php

declare(strict_types=1);

namespace Orchestrata;

final class NoOverlap implements Constraint
{
    public function __construct(
        public readonly array $intervals,
    ) {
        if (!array_is_list($intervals)) {
            throw new \InvalidArgumentException('intervals must be a list.');
        }
        foreach ($intervals as $item) {
            if (!($item instanceof ScheduledInterval)) {
                throw new \InvalidArgumentException('Invalid element in intervals.');
            }
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'kind' => 'no_overlap',
            'intervals' => $this->intervals,
        ];
    }
}
