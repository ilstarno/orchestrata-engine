#include "orchestrata.h"

#include <assert.h>
#include <string.h>

int main(void) {
    const char *input = "{\"variables\":[],\"constraints\":[]}";
    const char *expected = "{\"solution\":{\"values\":[],\"score\":{\"hard\":0,\"soft\":0},\"evaluated\":1}}";
    char *result = NULL;

    assert(orchestrata_abi_version() == ORCHESTRATA_ABI_VERSION);
    assert(orchestrata_solve_json_v1((const uint8_t *)input, strlen(input), 1, &result) == ORCHESTRATA_OK);
    assert(result != NULL);
    assert(strcmp(result, expected) == 0);
    orchestrata_free_json_v1(result);

    result = NULL;
    assert(orchestrata_solve_json_v1((const uint8_t *)input, strlen(input), 0, &result) == ORCHESTRATA_SEARCH_LIMIT);
    assert(result == NULL);
    orchestrata_free_json_v1(result);

    const char *extra = "{\"variables\":[],\"constraints\":[],\"extra\":true}";
    assert(orchestrata_solve_json_strict_v1((const uint8_t *)extra, strlen(extra), 0, &result) == ORCHESTRATA_UNKNOWN_FIELD);
    assert(result == NULL);

    assert(orchestrata_solve_json_strict_v1((const uint8_t *)input, strlen(input), 1, &result) == ORCHESTRATA_OK);
    assert(result != NULL);
    assert(strcmp(result, expected) == 0);
    orchestrata_free_json_v1(result);
}
