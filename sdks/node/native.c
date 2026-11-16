#include <node_api.h>
#include "orchestrata.h"

static napi_value throw_error(napi_env env, const char *message) {
    napi_throw_error(env, NULL, message);
    return NULL;
}

static napi_value throw_status(napi_env env, int32_t status) {
    const char *message;
    switch (status) {
        case ORCHESTRATA_INVALID_PROBLEM: message = "Invalid problem"; break;
        case ORCHESTRATA_SEARCH_LIMIT: message = "Search limit exceeded"; break;
        case ORCHESTRATA_INTERNAL_ERROR: message = "Solver execution failed"; break;
        case ORCHESTRATA_INVALID_ARGUMENT: message = "Invalid native argument"; break;
        default: message = "Unknown native status"; break;
    }

    napi_value text, error, code;
    if (napi_create_string_utf8(env, message, NAPI_AUTO_LENGTH, &text) != napi_ok ||
        napi_create_error(env, NULL, text, &error) != napi_ok ||
        napi_create_int32(env, status, &code) != napi_ok ||
        napi_set_named_property(env, error, "status", code) != napi_ok) {
        return throw_error(env, "Could not report solver status");
    }

    napi_throw(env, error);
    return NULL;
}

static napi_value solve_json(napi_env env, napi_callback_info info) {
    size_t argc = 2, request_length;
    napi_value argv[2], response;
    void *request;
    uint64_t limit;
    bool lossless;
    char *result = NULL;

    if (napi_get_cb_info(env, info, &argc, argv, NULL, NULL) != napi_ok || argc != 2 ||
        napi_get_buffer_info(env, argv[0], &request, &request_length) != napi_ok ||
        napi_get_value_bigint_uint64(env, argv[1], &limit, &lossless) != napi_ok || !lossless) {
        napi_throw_type_error(env, NULL, "Expected a Buffer and an unsigned 64-bit bigint");
        return NULL;
    }

    int32_t status = orchestrata_solve_json_v1(request, request_length, limit, &result);
    if (status != ORCHESTRATA_OK || result == NULL) {
        orchestrata_free_json_v1(result);
        return status == ORCHESTRATA_OK
            ? throw_error(env, "Native solver returned no response")
            : throw_status(env, status);
    }

    napi_status converted = napi_create_string_utf8(env, result, NAPI_AUTO_LENGTH, &response);
    orchestrata_free_json_v1(result);
    if (converted != napi_ok) {
        return throw_error(env, "Could not copy solver response");
    }
    return response;
}

static napi_value init(napi_env env, napi_value exports) {
    if (orchestrata_abi_version() != ORCHESTRATA_ABI_VERSION) {
        return throw_error(env, "Unsupported Orchestrata ABI version");
    }

    napi_property_descriptor method = {
        "solveJson", NULL, solve_json, NULL, NULL, NULL, napi_default, NULL
    };
    if (napi_define_properties(env, exports, 1, &method) != napi_ok) {
        return throw_error(env, "Could not initialize solver binding");
    }
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, init)
