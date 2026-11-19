<?php

declare(strict_types=1);

namespace Orchestrata;

final class Variable implements \JsonSerializable
{
    public function __construct(
        public readonly string $name,
        public readonly array $domain,
    ) {
        if (!array_is_list($domain)) {
            throw new \InvalidArgumentException('domain must be a list.');
        }
        foreach ($domain as $item) {
            if (!(is_int($item))) {
                throw new \InvalidArgumentException('Invalid element in domain.');
            }
        }
    }

    public function jsonSerialize(): array
    {
        return [
            'name' => $this->name,
            'domain' => $this->domain,
        ];
    }
}
