<?php

declare(strict_types=1);

namespace Orchestrata;

final class ResourceLoad implements \JsonSerializable
{
    public function __construct(
        public readonly int $variable,
        public readonly int $load,
    ) {
        if ($variable < 0) {
            throw new \InvalidArgumentException('variable must be nonnegative.');
        }
        if ($load < 0) {
            throw new \InvalidArgumentException('load must be nonnegative.');
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'variable' => $this->variable,
            'load' => $this->load,
        ];
    }
}
