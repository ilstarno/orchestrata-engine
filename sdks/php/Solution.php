<?php

declare(strict_types=1);

namespace Orchestrata;

final class Solution
{
    private function __construct(
        public readonly array $values,
        public readonly Score $score,
        public readonly int|string $evaluated,
    ) {
    }

    public static function fromAbiResult(string $json): ?self
    {
        if (strlen($json) >= 2097152) {
            throw new \UnexpectedValueException('Native result exceeds the ABI v1 limit.');
        }
        $document = json_decode($json, false, 512, JSON_THROW_ON_ERROR | JSON_BIGINT_AS_STRING);
        if (!$document instanceof \stdClass || array_keys(get_object_vars($document)) !== ['solution']) {
            throw new \UnexpectedValueException('Expected the ABI v1 solution envelope.');
        }
        $decoded = $document->solution;
        if ($decoded === null) {
            return null;
        }
        if (!$decoded instanceof \stdClass || count(get_object_vars($decoded)) !== 3) {
            throw new \UnexpectedValueException('Invalid native solution.');
        }
        $values = $decoded->values ?? null;
        $score = $decoded->score ?? null;
        $evaluated = $decoded->evaluated ?? null;
        if (!is_array($values)
            || !$score instanceof \stdClass || count(get_object_vars($score)) !== 2
            || ($score->hard ?? null) !== 0 || !is_int($score->soft ?? null) || $score->soft > 0
            || !((is_int($evaluated) && $evaluated > 0) || self::isWideCount($evaluated))) {
            throw new \UnexpectedValueException('Invalid native solution.');
        }
        foreach ($values as $value) {
            if (!is_int($value)) {
                throw new \UnexpectedValueException('Invalid native assignment value.');
            }
        }
        return new self($values, new Score($score->hard, $score->soft), $evaluated);
    }

    private static function isWideCount(mixed $value): bool
    {
        return is_string($value) && preg_match('/^[1-9][0-9]{0,19}$/D', $value) === 1
            && (strlen($value) > 19 || (strlen($value) === 19 && strcmp($value, '9223372036854775807') > 0))
            && (strlen($value) < 20 || strcmp($value, '18446744073709551615') <= 0);
    }
}
