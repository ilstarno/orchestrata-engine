<?php

declare(strict_types=1);

namespace Orchestrata;

final class ResourceLimit implements \JsonSerializable
{
    public function __construct(
        public readonly int $resource,
        public readonly int $capacity,
    ) {
        if ($capacity < 0) {
            throw new \InvalidArgumentException('capacity must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'resource' => $this->resource,
            'capacity' => $this->capacity,
        ];
    }
}
