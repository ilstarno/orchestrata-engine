<?php

declare(strict_types=1);

namespace Orchestrata;

final class ScheduledInterval implements \JsonSerializable
{
    public function __construct(
        public readonly int $start,
        public readonly int $duration,
    ) {
        if ($start < 0) {
            throw new \InvalidArgumentException('start must be nonnegative.');
        }
        if ($duration < 0) {
            throw new \InvalidArgumentException('duration must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'start' => $this->start,
            'duration' => $this->duration,
        ];
    }
}
