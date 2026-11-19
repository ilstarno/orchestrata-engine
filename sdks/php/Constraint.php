<?php

declare(strict_types=1);

namespace Orchestrata;

interface Constraint extends \JsonSerializable
{
    public function jsonSerialize(): array;
}
