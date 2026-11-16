#ifndef ORCHESTRATA_H
#define ORCHESTRATA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ORCHESTRATA_ABI_VERSION UINT32_C(1)
#define ORCHESTRATA_MAX_REQUEST_BYTES ((size_t)1048576)
#define ORCHESTRATA_MAX_RESULT_BYTES ((size_t)2097152)
#define ORCHESTRATA_OK INT32_C(0)
#define ORCHESTRATA_INVALID_PROBLEM INT32_C(1)
#define ORCHESTRATA_SEARCH_LIMIT INT32_C(2)
#define ORCHESTRATA_INTERNAL_ERROR INT32_C(3)
#define ORCHESTRATA_INVALID_ARGUMENT INT32_C(4)

uint32_t orchestrata_abi_version(void);

/*
 * Reads exactly request_length UTF-8 JSON bytes without retaining a pointer.
 * max_evaluations is a preflight cap; no partial result is returned.
 * result must be writable, aligned, and disjoint from request.
 * On ORCHESTRATA_OK, *result is nonnull and points to a UTF-8 JSON C string
 * of the form {"solution":{...}} or {"solution":null} when proven infeasible.
 * Its size, including the NUL, is at most ORCHESTRATA_MAX_RESULT_BYTES.
 * On error a valid *result is set to NULL; no buffer is allocated.
 * Input is limited to ORCHESTRATA_MAX_REQUEST_BYTES.
 */
int32_t orchestrata_solve_json_v1(
    const uint8_t *request,
    size_t request_length,
    uint64_t max_evaluations,
    char **result
);

/* NULL is allowed. Release each result exactly once; never modify its bytes. */
void orchestrata_free_json_v1(char *result);

#ifdef __cplusplus
}
#endif

#endif
