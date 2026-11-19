<?php

declare(strict_types=1);

namespace Orchestrata;

enum CountValueTier: string
{
    case Hard = 'hard';
    case Soft = 'soft';
}
