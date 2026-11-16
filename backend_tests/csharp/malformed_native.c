#include "orchestrata.h"

#include <stdlib.h>
#include <string.h>

/* Test-only native library: returns success with intentionally invalid V1 JSON. */
uint32_t orchestrata_abi_version(void) {
    return ORCHESTRATA_ABI_VERSION;
}

int32_t orchestrata_solve_json_v1(
    const uint8_t *request, size_t request_length, uint64_t max_evaluations, char **result
) {
    (void)request;
    (void)request_length;
    const char *json;
    switch (max_evaluations) {
        case 1: json = "{\"solution\":null,\"solution\":null}"; break;
        case 2: json = "{\"solution\":{\"values\":[0],\"score\":{\"hard\":-1,\"hard\":0,\"soft\":0},\"evaluated\":1}}"; break;
        case 3: json = "{\"solution\":{\"values\":[9],\"score\":{\"hard\":0,\"soft\":0},\"evaluated\":1}}"; break;
        case 4: json = "{\"solution\":{\"values\":[0.0],\"score\":{\"hard\":0,\"soft\":0},\"evaluated\":1}}"; break;
        case 6: *result = NULL; return ORCHESTRATA_OK;
        default: json = "{\"solution\":{\"values\":[0],\"score\":{\"hard\":0,\"soft\":0},\"evaluated\":6}}"; break;
    }
    size_t bytes = strlen(json) + 1;
    *result = malloc(bytes);
    if (*result == NULL)
        return ORCHESTRATA_INTERNAL_ERROR;
    memcpy(*result, json, bytes);
    return ORCHESTRATA_OK;
}

void orchestrata_free_json_v1(char *result) {
    free(result);
}
