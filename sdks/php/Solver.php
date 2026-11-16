<?php

declare(strict_types=1);

namespace Orchestrata;

final class Solver
{
    private const DECLARATIONS = <<<'C'
        uint32_t orchestrata_abi_version(void);
        int32_t orchestrata_solve_json_v1(
            const uint8_t *request, size_t request_length,
            uint64_t max_evaluations, char **result);
        void orchestrata_free_json_v1(char *result);
        C;

    private \FFI $library;

    public function __construct(string $library)
    {
        $path = realpath($library);
        if ($path === false) {
            throw new \InvalidArgumentException('Native library path does not exist.');
        }
        $this->library = \FFI::cdef(self::DECLARATIONS, $path);
        $abiVersion = $this->library->orchestrata_abi_version();
        if ($abiVersion !== 1) {
            throw new \RuntimeException("Unsupported Orchestrata ABI version: {$abiVersion}.");
        }
    }

    /** Returns the raw JSON result, without converting its 64-bit integers. */
    public function solveJson(string $problem, int $maxEvaluations): string
    {
        if ($maxEvaluations < 0) {
            throw new \InvalidArgumentException('The evaluation limit must be nonnegative.');
        }
        $length = strlen($problem);
        if ($length === 0 || $length > 1048576) {
            throw new SolveException(4);
        }

        $request = $this->library->new("uint8_t[{$length}]");
        \FFI::memcpy($request, $problem, $length);
        $result = $this->library->new('char *');
        $status = $this->library->orchestrata_solve_json_v1(
            $request,
            $length,
            $maxEvaluations,
            \FFI::addr($result),
        );
        try {
            if ($status !== 0) {
                throw new SolveException($status);
            }
            if (\FFI::isNull($result)) {
                throw new \RuntimeException('Native solver returned no response.');
            }

            return \FFI::string($result);
        } finally {
            $this->library->orchestrata_free_json_v1($result);
        }
    }
}
